use core::fmt::Debug;

use std::{
    error::Error,
    io::{Read, Seek},
};

use crate::ppu::palette::Palette;

/// Trait representing something that can read a ROM.
/// Used internally by the Ruboy ROM memory-bank-controllers to read the data
/// for each bank into memory dynamically.
pub trait RomReader: Debug {
    /// The error that can be returned by this reader.
    type Err: Error + 'static;

    /// Given a buffer, this function should fill this buffer _completely_ with
    /// data from the ROM on disk, starting at offset "addr"
    ///
    /// # Arguments
    ///
    /// * `buf` The buffer to fill
    /// * `addr` The offset of the ROM on disk to start reading from
    fn read_into(&mut self, buf: &mut [u8], addr: usize) -> Result<(), Self::Err>;

    /// Given a constant size N, this function should return a byte-array of size N filled with
    /// bytes taken from the ROM on disk, starting at offset "addr"
    ///
    /// # Arguments
    /// * `N` The amount of data to read, and the size of the returned array
    /// * `addr` The offset of the ROM on disk to start reading from
    fn read<const N: usize>(&mut self, addr: usize) -> Result<[u8; N], Self::Err> {
        let mut buf = [0u8; N];

        self.read_into(&mut buf, addr)?;

        Ok(buf)
    }
}

impl<T> RomReader for T
where
    T: Read + Seek + Debug,
{
    type Err = std::io::Error;

    fn read_into(&mut self, buf: &mut [u8], addr: usize) -> Result<(), Self::Err> {
        let cur_pos = self.stream_position()?;

        if usize::try_from(cur_pos).unwrap() != addr {
            self.seek(std::io::SeekFrom::Start(u64::try_from(addr).unwrap()))?;
        };

        match self.read_exact(buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// Trait representing something that can allocate memory for [crate::Ruboy]
/// Usually not required to implement directly, but can be useful if a custom memory
/// allocator is used.
///
/// See the two provided implementations: [InlineAllocator] and [BoxAllocator]
pub trait GBAllocator: Debug {
    /// The type of the memory created by this allocator. For example [T; N] or Box<[T; N]>
    type Mem<T: Copy + Debug, const N: usize>: GBRam<T> + Debug;

    /// Return an initialized buffer of size N, filled with clones of "orig"
    ///
    /// # Arguments
    ///
    /// * `T` The type of the buffer elements
    /// * `N` The size of the buffer, in amount of elements
    /// * `orig` The object to clone for initializing each element
    fn clone_from<T: Copy + Debug, const N: usize>(orig: &T) -> Self::Mem<T, N>;

    /// Return a buffer of size N with each element initialized to its [Default]
    ///
    /// # Arguments
    ///
    /// * `T` The type of the buffer elements
    /// * `N` The siz of the buffer, in amount of elements
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

#[derive(Debug)]
pub struct InlineAllocator;

impl GBAllocator for InlineAllocator {
    type Mem<T: Copy + Debug, const N: usize> = [T; N];

    fn clone_from<T: Copy + Debug, const N: usize>(orig: &T) -> Self::Mem<T, N> {
        std::array::from_fn(|_| *orig)
    }

    fn empty<T: Default + Copy + Debug, const N: usize>() -> Self::Mem<T, N> {
        [T::default(); N]
    }
}

#[derive(Debug)]
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
pub enum GbMonoColor {
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    Black = 3,
}

impl GbMonoColor {
    pub const fn from_id(id: GbColorID, palette: Option<Palette>) -> Self {
        match palette {
            Some(_) => todo!(),
            None => match id {
                GbColorID::ID0 => Self::White,
                GbColorID::ID1 => Self::LightGray,
                GbColorID::ID2 => Self::DarkGray,
                GbColorID::ID3 => Self::Black,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GbColorID {
    ID0,
    ID1,
    ID2,
    ID3,
}

impl TryFrom<u8> for GbColorID {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        let cid = match value {
            0 => GbColorID::ID0,
            1 => GbColorID::ID1,
            2 => GbColorID::ID2,
            3 => GbColorID::ID3,
            _ => return Err(()),
        };

        Ok(cid)
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pixels: [GbMonoColor; (FRAME_X as usize) * (FRAME_Y as usize)],
}

impl Frame {
    pub fn get_raw(&self) -> &[GbMonoColor] {
        &self.pixels
    }

    pub fn get_raw_mut(&mut self) -> &mut [GbMonoColor] {
        &mut self.pixels
    }

    pub fn get_pix(&self, x: u8, y: u8) -> Option<GbMonoColor> {
        if x as usize >= FRAME_X || y as usize >= FRAME_Y {
            return None;
        }

        Some(self.pixels[(y as usize * FRAME_X) + x as usize])
    }

    pub fn set_pix(&mut self, x: u8, y: u8, val: GbMonoColor) {
        if x as usize >= FRAME_X || y as usize >= FRAME_Y {
            log::warn!(
                "Attempt to set pixel outside of framebuffer at X={} Y={}",
                x,
                y
            );
            return;
        }

        self.pixels[(y as usize * FRAME_X) + x as usize] = val;
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            pixels: [GbMonoColor::White; FRAME_X * FRAME_Y],
        }
    }
}

pub trait GBGraphicsDrawer: Debug {
    type Err: Error + 'static;
    fn output(&mut self, frame: &Frame) -> Result<(), Self::Err>;
}
