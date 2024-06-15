use std::marker::PhantomData;

use nonbanking::NonBankingController;
use thiserror::Error;

use crate::extern_traits::GBAllocator;

use super::meta::{RomMeta, RomMetaParseError};
use crate::extern_traits::RomReader;

mod nonbanking;

trait Mbc {
    fn read(&self, addr: u16) -> Result<u8, ReadError>;
    fn write(&mut self, addr: u16, val: u8) -> Result<(), WriteError>;
}

#[derive(Debug)]
#[allow(unused_associated_type_bounds)]
pub enum RomController<A: GBAllocator, R: RomReader> {
    None(NonBankingController<A>),
    // TODO: Remove when an actual variant that uses R is introduced
    Phantom(PhantomData<R>),
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
            Some(mapper) => todo!("MBC not yet implemented: {}", mapper),
            None => RomController::None(
                NonBankingController::new(meta, rom).map_err(|e| RomControllerInitErr::Read(e))?,
            ),
        };

        Ok(controller)
    }

    pub fn read(&self, addr: u16) -> Result<u8, ReadError> {
        let result = match self {
            RomController::None(c) => c.read(addr)?,
            RomController::Phantom(_) => todo!(),
        };

        Ok(result)
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<(), WriteError> {
        match self {
            RomController::None(c) => c.write(addr, val)?,
            RomController::Phantom(_) => todo!(),
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
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("RAM address {addr} out of reach for this cartridge (max {max})")]
    NotEnoughRam { addr: u16, max: u16 },

    #[error("Address is read only: 0x{:x}", .0)]
    ReadOnly(u16),
}
