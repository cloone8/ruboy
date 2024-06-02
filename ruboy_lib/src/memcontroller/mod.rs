use std::{error::Error, fmt::Display};

use thiserror::Error;

use crate::{
    boot,
    extern_traits::{GBAllocator, GBRam, RomReader},
    isa::decoder::DecoderReadable,
    rom::{
        self,
        controller::{RomController, RomControllerInitErr},
    },
};

const WORKRAM_START: u16 = 0xC000;
const WORKRAM_END: u16 = 0xE000;
const WORKRAM_SIZE: u16 = WORKRAM_END - WORKRAM_START;

const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0xA000;
const VRAM_SIZE: u16 = VRAM_END - VRAM_START;

const HRAM_START: u16 = 0xFF80;
const HRAM_END: u16 = 0xFFFF;
const HRAM_SIZE: u16 = HRAM_END - HRAM_START;

#[derive(Debug, Clone, Copy, Default)]
struct Interrupts {
    raw: u8,
}

impl Interrupts {
    #[inline]
    const fn get(self, mask: u8) -> bool {
        (self.raw & mask) != 0
    }

    #[inline]
    fn set(&mut self, mask: u8, val: bool) {
        if val {
            self.raw |= mask
        } else {
            self.raw &= !mask
        }
    }

    #[inline]
    pub fn vblank(self) -> bool {
        self.get(0b1)
    }

    #[inline]
    pub fn lcd(self) -> bool {
        self.get(0b01)
    }

    #[inline]
    pub fn timer(self) -> bool {
        self.get(0b001)
    }

    #[inline]
    pub fn serial(self) -> bool {
        self.get(0b0001)
    }

    #[inline]
    pub fn joypad(self) -> bool {
        self.get(0b00001)
    }

    #[inline]
    pub fn set_vblank(&mut self, val: bool) {
        self.set(0b1, val)
    }

    #[inline]
    pub fn set_lcd(&mut self, val: bool) {
        self.set(0b01, val)
    }

    #[inline]
    pub fn set_timer(&mut self, val: bool) {
        self.set(0b001, val)
    }

    #[inline]
    pub fn set_serial(&mut self, val: bool) {
        self.set(0b0001, val)
    }

    #[inline]
    pub fn set_joypad(&mut self, val: bool) {
        self.set(0b00001, val)
    }
}

impl From<u8> for Interrupts {
    fn from(value: u8) -> Self {
        Interrupts { raw: value }
    }
}

impl From<Interrupts> for u8 {
    fn from(value: Interrupts) -> Self {
        value.raw
    }
}

pub struct MemController<A: GBAllocator, R: RomReader> {
    boot_rom_enabled: bool,
    ram: A::Mem<u8, { WORKRAM_SIZE as usize }>,
    vram: A::Mem<u8, { VRAM_SIZE as usize }>,
    hram: A::Mem<u8, { HRAM_SIZE as usize }>,
    interrupts_enabled: Interrupts,
    interrupts_requested: Interrupts,
    rom: RomController<A, R>,
}

#[derive(Debug, Clone, Copy)]
enum MemRegion {
    BootRom,
    Cartridge,
    VRam,
    WorkRam,
    EchoRam,
    ObjectAttrMem,
    Prohibited,
    IORegs,
    HighRam,
    InterruptEnableReg,
}

impl Display for MemRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MemRegion::BootRom => "Boot ROM",
            MemRegion::Cartridge => "Cartridge",
            MemRegion::WorkRam => "Working RAM",
            MemRegion::VRam => "VRAM",
            MemRegion::IORegs => "I/O Registers",
            MemRegion::HighRam => "High RAM",
            MemRegion::InterruptEnableReg => "Interrupt Enable Register",
            MemRegion::EchoRam => "Echo RAM",
            MemRegion::ObjectAttrMem => "Object Attribute Memory",
            MemRegion::Prohibited => "Prohibited",
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, Error)]
pub enum ReadErrType {
    #[error("Error during ROM reading: {0}")]
    Rom(#[from] rom::controller::ReadError),
}

#[derive(Debug)]
pub struct ReadError {
    addr: u16,
    region: MemRegion,
    err: ReadErrType,
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.err)
    }
}

impl Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error during memory read at 0x{:x} in region {}: {}",
            self.addr, self.region, self.err
        )
    }
}

#[derive(Debug)]
pub struct WriteError {
    addr: u16,
    region: MemRegion,
    err: WriteErrType,
}

impl Error for WriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.err)
    }
}

impl Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error during memory read at 0x{:x} in region {}: {}",
            self.addr, self.region, self.err
        )
    }
}

#[derive(Debug, Error)]
pub enum WriteErrType {
    #[error("Write to read-only memory")]
    ReadOnly,

    #[error("Error during ROM writing: {0}")]
    Rom(#[from] rom::controller::WriteError),
}

macro_rules! unimplemented_read {
    ($region:expr) => {
        todo!("Attempted read at unimplemented region {}", $region)
    };
}

macro_rules! unimplemented_write {
    ($region:expr) => {
        todo!("Attempted write at unimplemented region {}", $region)
    };
}

#[derive(Debug, Error)]
pub enum MemControllerInitErr<R: RomReader> {
    #[error("Could not initialize ROM controller: {0}")]
    Rom(#[from] RomControllerInitErr<R>),
}

impl<A: GBAllocator, R: RomReader> MemController<A, R> {
    pub fn new(rom: R) -> Result<Self, MemControllerInitErr<R>> {
        log::debug!("Initializing memory controller");

        Ok(MemController {
            boot_rom_enabled: cfg!(feature = "boot_img_enabled"),
            ram: A::empty(),
            vram: A::empty(),
            hram: A::empty(),
            interrupts_enabled: Interrupts::default(),
            interrupts_requested: Interrupts::default(),
            rom: RomController::new(rom)?,
        })
    }

    #[inline]
    fn r_err(&self, addr: u16, err: impl Into<ReadErrType>) -> ReadError {
        ReadError {
            addr,
            region: self.map_to_region(addr),
            err: err.into(),
        }
    }

    #[inline]
    fn w_err(&self, addr: u16, err: impl Into<WriteErrType>) -> WriteError {
        WriteError {
            addr,
            region: self.map_to_region(addr),
            err: err.into(),
        }
    }

    fn map_to_region(&self, addr: u16) -> MemRegion {
        match addr {
            0x0..=0xFE => {
                if self.boot_rom_enabled {
                    MemRegion::BootRom
                } else {
                    MemRegion::Cartridge
                }
            }
            0xFF..=0x7FFF => MemRegion::Cartridge,
            0x8000..=0x9FFF => MemRegion::VRam,
            0xA000..=0xBFFF => MemRegion::Cartridge,
            0xC000..=0xDFFF => MemRegion::WorkRam,
            0xE000..=0xFDFF => MemRegion::EchoRam,
            0xFE00..=0xFE9F => MemRegion::ObjectAttrMem,
            0xFEA0..=0xFEFF => MemRegion::Prohibited,
            0xFF00..=0xFF7F => MemRegion::IORegs,
            0xFF80..=0xFFFE => MemRegion::HighRam,
            0xFFFF => MemRegion::InterruptEnableReg,
        }
    }

    pub fn read8(&self, addr: u16) -> Result<u8, ReadError> {
        match self.map_to_region(addr) {
            MemRegion::BootRom => Ok(boot::IMAGE[addr as usize]),
            MemRegion::Cartridge => self.rom.read(addr).map_err(|e| self.r_err(addr, e)),
            MemRegion::VRam => Ok(self.vram.read(addr - VRAM_START)),
            MemRegion::WorkRam => Ok(self.ram.read(addr - WORKRAM_START)),
            MemRegion::EchoRam => unimplemented_read!(MemRegion::EchoRam),
            MemRegion::ObjectAttrMem => unimplemented_read!(MemRegion::ObjectAttrMem),
            MemRegion::Prohibited => unimplemented_read!(MemRegion::Prohibited),
            MemRegion::IORegs => Ok(self.io_read(addr)),
            MemRegion::HighRam => Ok(self.hram.read(addr - HRAM_START)),
            MemRegion::InterruptEnableReg => Ok(self.interrupts_enabled.into()),
        }
    }

    pub fn read16(&self, addr: u16) -> Result<u16, ReadError> {
        Ok(u16::from_le_bytes([
            self.read8(addr)?,
            self.read8(addr + 1)?,
        ]))
    }

    pub fn write8(&mut self, addr: u16, value: u8) -> Result<(), WriteError> {
        match self.map_to_region(addr) {
            MemRegion::BootRom => Err(self.w_err(addr, WriteErrType::ReadOnly)),
            MemRegion::Cartridge => self.rom.write(addr, value).map_err(|e| self.w_err(addr, e)),
            MemRegion::VRam => {
                self.vram.write(addr - VRAM_START, value);
                Ok(())
            }
            MemRegion::WorkRam => {
                self.ram.write(addr - WORKRAM_START, value);
                Ok(())
            }
            MemRegion::EchoRam => unimplemented_write!(MemRegion::EchoRam),
            MemRegion::ObjectAttrMem => unimplemented_write!(MemRegion::ObjectAttrMem),
            MemRegion::Prohibited => unimplemented_write!(MemRegion::Prohibited),
            MemRegion::IORegs => self.io_write(addr, value),
            MemRegion::HighRam => {
                self.hram.write(addr - HRAM_START, value);
                Ok(())
            }
            MemRegion::InterruptEnableReg => {
                self.interrupts_enabled = value.into();
                Ok(())
            }
        }
    }

    pub fn write16(&mut self, addr: u16, value: u16) -> Result<(), WriteError> {
        let bytes = value.to_le_bytes();

        self.write8(addr, bytes[0])?;
        self.write8(addr + 1, bytes[1])
    }

    fn io_write(&mut self, addr: u16, val: u8) -> Result<(), WriteError> {
        match addr {
            ..=0xFEFF => panic!("Too low for I/O range"),
            0xFF50 => {
                if self.boot_rom_enabled && val != 0 {
                    log::debug!("Disabling boot ROM");
                }

                self.boot_rom_enabled = self.boot_rom_enabled && val == 0; // Disable boot-rom if non-zero is written

                Ok(())
            }
            0xFF80.. => panic!("Too high for I/O range"),
            _ => {
                log::debug!("I/O register not implemented for writing: 0x{:x}", addr);
                Ok(())
            }
        }
    }

    fn io_read(&self, addr: u16) -> u8 {
        match addr {
            ..=0xFEFF => panic!("Too low for I/O range"),
            0xFF80.. => panic!("Too high for I/O range"),
            _ => {
                log::debug!(
                    "I/O register not implemented for reading: 0x{:x}, returning 0xFF",
                    addr
                );
                0xFF
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum MemControllerDecoderErr {
    #[error("Address out of 16 bit range: {0}")]
    Addr(usize),

    #[error("Read error: {0}")]
    Read(#[from] ReadError),
}

impl<A: GBAllocator, R: RomReader> DecoderReadable for MemController<A, R> {
    type Err = MemControllerDecoderErr;
    fn read_at(&self, idx: usize) -> Result<u8, Self::Err> {
        let result = match u16::try_from(idx) {
            Ok(addr) => self.read8(addr)?,
            Err(_) => return Err(MemControllerDecoderErr::Addr(idx)),
        };

        Ok(result)
    }
}
