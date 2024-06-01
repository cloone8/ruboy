use std::{
    error::Error,
    io::{Read, Seek},
};

pub(crate) mod controller;
pub mod meta;

pub trait RomReader {
    type Err: Error + 'static;
    fn read_into(&mut self, buf: &mut [u8], addr: usize) -> Result<(), Self::Err>;
    fn read<const N: usize>(&mut self, addr: usize) -> Result<[u8; N], Self::Err> {
        let mut buf = [0u8; N];

        self.read_into(&mut buf, addr)?;

        Ok(buf)
    }
}

impl<T> RomReader for T
where
    T: Read + Seek,
{
    type Err = std::io::Error;

    fn read_into(&mut self, buf: &mut [u8], addr: usize) -> Result<(), Self::Err> {
        let cur_pos = self.stream_position()?;

        if usize::try_from(cur_pos).unwrap() != addr {
            self.seek(std::io::SeekFrom::Start(u64::try_from(addr).unwrap()))?;
        };

        match self.read_exact(buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
