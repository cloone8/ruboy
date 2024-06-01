use std::{
    error::Error,
    io::{Read, Seek},
};

use thiserror::Error;

pub(crate) mod controller;
pub mod meta;

#[derive(Debug, Clone, Copy, Error)]
pub enum RomReadErr<T: Error> {
    #[error("Error returned by reader: {0}")]
    ReaderErr(#[from] T),
}

pub trait RomReader {
    type Err: Error + 'static;
    fn read<const N: usize>(&mut self, addr: usize) -> Result<[u8; N], RomReadErr<Self::Err>>;
}

impl<T> RomReader for T
where
    T: Read + Seek,
{
    type Err = std::io::Error;

    fn read<const N: usize>(&mut self, addr: usize) -> Result<[u8; N], RomReadErr<Self::Err>> {
        let mut buf = [0u8; N];

        let cur_pos = self.stream_position()?;

        if usize::try_from(cur_pos).unwrap() != addr {
            self.seek(std::io::SeekFrom::Start(u64::try_from(addr).unwrap()))?;
        };

        match self.read_exact(&mut buf) {
            Ok(_) => Ok(buf),
            Err(e) => Err(e.into()),
        }
    }
}
