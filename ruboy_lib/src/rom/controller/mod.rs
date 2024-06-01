use nonbanking::NonBankingController;
use thiserror::Error;

use super::{
    meta::{RomMeta, RomMetaParseError},
    RomReadErr, RomReader,
};

mod nonbanking;

#[derive(Debug, Error)]
pub enum MbcReadErr {}

#[derive(Debug, Error)]
pub enum MbcWriteErr {}

trait Mbc<R: RomReader> {
    fn read(&self, addr: u16) -> Result<u8, MbcReadErr>;
    fn write(&mut self, addr: u16, val: u8) -> Result<(), MbcWriteErr>;
}

#[derive(Debug)]
enum MemBankController<R: RomReader> {
    None(NonBankingController<R>),
}

impl<R: RomReader> Mbc<R> for MemBankController<R> {
    fn read(&self, addr: u16) -> Result<u8, MbcReadErr> {
        match self {
            MemBankController::None(c) => c.read(addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> Result<(), MbcWriteErr> {
        match self {
            MemBankController::None(c) => c.write(addr, val),
        }
    }
}

#[derive(Debug)]
pub struct RomController<R: RomReader> {
    meta: RomMeta,
    controller: MemBankController<R>,
}

#[derive(Debug, Clone, Error)]
pub enum RomControllerInitErr<R: RomReader> {
    #[error("Error reading ROM file: {0}")]
    Read(#[from] RomReadErr<R::Err>),

    #[error("Error parsing ROM file: {0}")]
    Parse(#[from] RomMetaParseError),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("MBC returned error during read: {0}")]
    Mbc(#[from] MbcReadErr),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("MBC returned error during write: {0}")]
    Mbc(#[from] MbcWriteErr),
}

impl<R: RomReader> RomController<R> {
    pub fn new(mut rom: R) -> Result<Self, RomControllerInitErr<R>> {
        let header_bytes: [u8; RomMeta::HEADER_LENGTH] = rom.read(RomMeta::OFFSET_HEADER_START)?;
        let meta = RomMeta::parse(&header_bytes)?;

        let controller = match meta.cartridge_hardware().mapper() {
            Some(mapper) => todo!("MBC not yet implemented: {}", mapper),
            None => MemBankController::None(NonBankingController::new(rom)),
        };

        Ok(Self { meta, controller })
    }

    pub fn read(&self, addr: u16) -> Result<u8, ReadError> {
        self.controller.read(addr).map_err(ReadError::from)
    }

    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), WriteError> {
        self.controller.write(addr, value).map_err(WriteError::from)
    }
}
