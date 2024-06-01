use thiserror::Error;

use crate::rom::RomReader;

use super::{Mbc, MbcReadErr, MbcWriteErr};

#[derive(Debug)]
pub struct NonBankingController<R: RomReader> {
    reader: R,
}

impl<R: RomReader> NonBankingController<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

#[derive(Debug, Error)]
pub enum ReadError {}

#[derive(Debug, Error)]
pub enum WriteError {}

impl<R: RomReader> Mbc<R> for NonBankingController<R> {
    fn read(&self, addr: u16) -> Result<u8, MbcReadErr> {
        todo!()
    }

    fn write(&mut self, addr: u16, val: u8) -> Result<(), MbcWriteErr> {
        todo!()
    }
}
