use mbc1::Mbc1;
use nonbanking::NonBankingController;
use thiserror::Error;

use crate::extern_traits::GBAllocator;
use crate::rom::meta::CartridgeMapper;

use super::meta::{RomMeta, RomMetaParseError};
use crate::extern_traits::RomReader;

mod mbc1;
mod nonbanking;

trait Mbc {
    fn read(&self, addr: u16) -> Result<u8, ReadError>;
    fn write(&mut self, addr: u16, val: u8) -> Result<(), WriteError>;
}

#[derive(Debug)]
#[allow(unused_associated_type_bounds)]
pub enum RomController<A: GBAllocator, R: RomReader> {
    None(NonBankingController<A>),
    Mbc1(Mbc1<A, R>),
}

impl<A: GBAllocator, R: RomReader> RomController<A, R> {
    pub fn new(mut rom: R) -> Result<Self, RomControllerInitErr<R>> {
        log::debug!("Initializing ROM controller");

        let header_bytes: [u8; RomMeta::HEADER_LENGTH] = rom
            .read(RomMeta::OFFSET_HEADER_START)
            .map_err(|e| RomControllerInitErr::Read(e))?;

        let meta = RomMeta::parse(&header_bytes)?;

        log::debug!("Resolving ROM mapper type");

        let controller = match meta.cartridge_hardware().mapper() {
            Some(mapper) => match mapper {
                CartridgeMapper::MBC1 => RomController::Mbc1(
                    Mbc1::new(meta, rom).map_err(|e| RomControllerInitErr::Read(e))?,
                ),
                _ => todo!("ROM controller not yet implemented: {}", mapper),
            },
            None => RomController::None(
                NonBankingController::new(meta, rom).map_err(|e| RomControllerInitErr::Read(e))?,
            ),
        };

        Ok(controller)
    }

    pub fn read(&self, addr: u16) -> Result<u8, ReadError> {
        let result = match self {
            RomController::None(c) => c.read(addr)?,
            RomController::Mbc1(mbc) => mbc.read(addr)?,
        };

        Ok(result)
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<(), WriteError> {
        match self {
            RomController::None(c) => c.write(addr, val)?,
            RomController::Mbc1(mbc) => mbc.write(addr, val)?,
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Error)]
pub enum RomControllerInitErr<R: RomReader> {
    #[error("Error reading ROM file: {0}")]
    Read(#[source] R::Err),

    #[error("Error parsing ROM file: {0}")]
    Parse(#[from] RomMetaParseError),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("RAM address {addr} out of reach for this cartridge (max {max})")]
    NotEnoughRam { addr: u16, max: u16 },

    #[error("Error with RomReader: {}", 0)]
    Reader(Box<dyn std::error::Error>),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("RAM address {addr} out of reach for this cartridge (max {max})")]
    NotEnoughRam { addr: u16, max: u16 },

    #[error("Address is read only: 0x{:x}", .0)]
    ReadOnly(u16),

    #[error("Error with RomReader: {}", 0)]
    Reader(Box<dyn std::error::Error>),
}

/// Converts a bank index to an address within the ROM
const fn bank_num_to_addr(num: usize) -> usize {
    0x4000 * num
}
