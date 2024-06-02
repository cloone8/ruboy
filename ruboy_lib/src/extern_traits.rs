use core::fmt::Debug;

use std::{
    error::Error,
    io::{Read, Seek},
};

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

pub trait GBAllocator {
    type Mem<T: Copy + Debug, const N: usize>: GBRam<T> + Debug;
    fn clone_from<T: Copy + Debug, const N: usize>(orig: &T) -> Self::Mem<T, N>;
    fn empty<T: Default + Copy + Debug, const N: usize>() -> Self::Mem<T, N>;
}

pub trait GBRam<T: Copy + Debug> {
    fn read(&self, addr: u16) -> T;
    fn write(&mut self, addr: u16, val: T);

    fn raw(&self) -> &[T];
    fn raw_mut(&mut self) -> &mut [T];
    fn size(&self) -> usize;
}

impl<T: Copy + Debug, const N: usize> GBRam<T> for [T; N] {
    fn read(&self, addr: u16) -> T {
        self[addr as usize]
    }

    fn write(&mut self, addr: u16, val: T) {
        self[addr as usize] = val;
    }

    fn raw(&self) -> &[T] {
        self
    }

    fn raw_mut(&mut self) -> &mut [T] {
        self
    }

    fn size(&self) -> usize {
        N
    }
}

impl<T: Copy + Debug, R: GBRam<T>> GBRam<T> for Box<R> {
    fn read(&self, addr: u16) -> T {
        self.as_ref().read(addr)
    }

    fn write(&mut self, addr: u16, val: T) {
        self.as_mut().write(addr, val)
    }

    fn raw(&self) -> &[T] {
        self.as_ref().raw()
    }

    fn raw_mut(&mut self) -> &mut [T] {
        self.as_mut().raw_mut()
    }

    fn size(&self) -> usize {
        self.as_ref().size()
    }
}

pub struct StackAllocator;

impl GBAllocator for StackAllocator {
    type Mem<T: Copy + Debug, const N: usize> = [T; N];

    fn clone_from<T: Copy + Debug, const N: usize>(orig: &T) -> Self::Mem<T, N> {
        std::array::from_fn(|_| *orig)
    }

    fn empty<T: Default + Copy + Debug, const N: usize>() -> Self::Mem<T, N> {
        [T::default(); N]
    }
}

pub struct BoxAllocator;

impl GBAllocator for BoxAllocator {
    type Mem<T: Copy + Debug, const N: usize> = Box<[T; N]>;

    fn clone_from<T: Copy + Debug, const N: usize>(orig: &T) -> Self::Mem<T, N> {
        Box::new(std::array::from_fn(|_| *orig))
    }

    fn empty<T: Default + Copy + Debug, const N: usize>() -> Self::Mem<T, N> {
        Box::new([T::default(); N])
    }
}

pub const FRAME_X: usize = 160;
pub const FRAME_Y: usize = 144;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GbColorVal {
    ID0 = 0,
    ID1 = 1,
    ID2 = 2,
    ID3 = 3,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pixels: [GbColorVal; FRAME_X * FRAME_Y],
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            pixels: [GbColorVal::ID0; FRAME_X * FRAME_Y],
        }
    }
}

pub trait GBGraphicsDrawer {
    type Err: Error;
    fn output(&mut self, frame: &Frame) -> Result<(), Self::Err>;
}
