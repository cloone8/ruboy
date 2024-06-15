use std::{error::Error, fmt::Display};

use interrupts::Interrupts;
use io::{IoReadErr, IoRegs, IoWriteErr};
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

pub mod interrupts;
pub mod io;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0xA000;
pub const VRAM_SIZE: u16 = VRAM_END - VRAM_START;

pub const WORKRAM_START: u16 = 0xC000;
pub const WORKRAM_END: u16 = 0xE000;
pub const WORKRAM_SIZE: u16 = WORKRAM_END - WORKRAM_START;

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFEA0;
pub const OAM_SIZE: u16 = OAM_END - OAM_START;

pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFF;
pub const HRAM_SIZE: u16 = HRAM_END - HRAM_START;

pub struct MemController<A: GBAllocator, R: RomReader> {
    rom: RomController<A, R>,
    vram: A::Mem<u8, { VRAM_SIZE as usize }>,
    ram: A::Mem<u8, { WORKRAM_SIZE as usize }>,
    oam: A::Mem<u8, { OAM_SIZE as usize }>,
    hram: A::Mem<u8, { HRAM_SIZE as usize }>,
    interrupts_enabled: Interrupts,

    /// If true, CPU can access VRAM
    pub vram_open: bool,

    /// If true, CPU can access OAM mem
    pub oam_open: bool,

    pub io_registers: IoRegs,
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

    #[error("Error during I/O register reading: {0}")]
    IORegs(#[from] IoReadErr),
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

    #[error("Error during I/O register writing: {0}")]
    IORegs(#[from] IoWriteErr),
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
            rom: RomController::new(rom)?,
            vram: A::empty(),
            ram: A::empty(),
            oam: A::empty(),
            hram: A::empty(),
            io_registers: IoRegs::new(),
            interrupts_enabled: Interrupts::default(),
            vram_open: true,
            oam_open: true,
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
                if self.io_registers.boot_rom_enabled {
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

    pub fn read_range<const N: usize>(&self, addr: u16) -> Result<[u8; N], ReadError> {
        let mut buf = [0u8; N];

        for i in 0u16..(N as u16) {
            buf[i as usize] = self.read8(addr + i)?;
        }

        Ok(buf)
    }

    pub fn read8(&self, addr: u16) -> Result<u8, ReadError> {
        match self.map_to_region(addr) {
            MemRegion::BootRom => Ok(boot::IMAGE[addr as usize]),
            MemRegion::Cartridge => self.rom.read(addr).map_err(|e| self.r_err(addr, e)),
            MemRegion::VRam => {
                let res = self.vram.read(addr - VRAM_START);
                // log::info!("Reading from VRAM @ 0x{:x}: 0x{:x}", addr, res);
                Ok(res)
            }
            MemRegion::WorkRam => Ok(self.ram.read(addr - WORKRAM_START)),
            MemRegion::EchoRam => unimplemented_read!(MemRegion::EchoRam),
            MemRegion::ObjectAttrMem => Ok(self.oam.read(addr - OAM_START)),
            MemRegion::Prohibited => unimplemented_read!(MemRegion::Prohibited),
            MemRegion::IORegs => self
                .io_registers
                .read(addr)
                .map_err(|e| self.r_err(addr, e)),
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
                // log::info!("Writing into VRAM @ 0x{:x}: 0x{:x}", addr, value);
                self.vram.write(addr - VRAM_START, value);
                Ok(())
            }
            MemRegion::WorkRam => {
                self.ram.write(addr - WORKRAM_START, value);
                Ok(())
            }
            MemRegion::EchoRam => unimplemented_write!(MemRegion::EchoRam),
            MemRegion::ObjectAttrMem => {
                self.oam.write(addr - OAM_START, value);
                Ok(())
            }
            MemRegion::Prohibited => unimplemented_write!(MemRegion::Prohibited),
            MemRegion::IORegs => self
                .io_registers
                .write(addr, value)
                .map_err(|e| self.w_err(addr, e)),
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
