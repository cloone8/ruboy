use thiserror::Error;

use super::{
    meta::{RomMeta, RomMetaParseError},
    RomReadErr, RomReader,
};

pub struct RomController<R: RomReader> {
    meta: RomMeta,
    reader: R,
}

#[derive(Debug, Clone, Error)]
pub enum RomControllerInitErr<R: RomReader> {
    #[error("Error reading ROM file: {0}")]
    Read(#[from] RomReadErr<R::Err>),

    #[error("Error parsing ROM file: {0}")]
    Parse(#[from] RomMetaParseError),
}

impl<R: RomReader> RomController<R> {
    pub fn new(mut rom: R) -> Result<Self, RomControllerInitErr<R>> {
        let header_bytes: [u8; RomMeta::HEADER_LENGTH] = rom.read(RomMeta::OFFSET_HEADER_START)?;
        let meta = RomMeta::parse(&header_bytes)?;

        Ok(Self { meta, reader: rom })
    }
}
