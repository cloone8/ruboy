use allocator::GBAllocator;
use thiserror::Error;

use crate::{boot, isa::decoder::DecoderReadable};
pub mod allocator;

const WORKRAM_SIZE: usize = 0xDFFF - 0xC000;

pub struct MemController<T: GBAllocator> {
    boot_rom_enabled: bool,
    ram: T::Mem<WORKRAM_SIZE>,
}

#[derive(Debug, Clone, Copy)]
enum MemRegion {
    BootRom,
    Cartridge,
    WorkRam,
    VRam,
    IORegs,
    HighRam,
}

#[derive(Debug, Clone, Copy, Error)]
pub enum WriteError {
    #[error("Attempt to write to read-only memory at {:x}", 0)]
    ReadOnly(u16),
}

macro_rules! unimplemented_read {
    ($region:expr) => {{
        log::debug!(
            "Attempted read at unimplemented region {:?}, returning 0x0",
            $region
        );
        0
    }};
}

macro_rules! unimplemented_write {
    ($region:expr) => {{
        log::debug!(
            "Attempted write at unimplemented region {:?}, writing nothing",
            $region
        );

        Ok(())
    }};
}

impl<T: GBAllocator> MemController<T> {
    pub fn new() -> Self {
        MemController {
            boot_rom_enabled: cfg!(feature = "boot_img_enabled"),
            ram: T::allocate::<WORKRAM_SIZE>(),
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
            0xC000..=0xDFFF => MemRegion::WorkRam,
            0xFF00..=0xFF7F => MemRegion::IORegs,
            0xFF80..=0xFFFE => MemRegion::HighRam,
            _ => panic!("Unknown memory region at 0x{:x}", addr),
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match self.map_to_region(addr) {
            MemRegion::BootRom => boot::IMAGE[addr as usize],
            MemRegion::Cartridge => unimplemented_read!(MemRegion::Cartridge),
            MemRegion::WorkRam => T::read(&self.ram, addr - 0xC000),
            MemRegion::VRam => unimplemented_read!(MemRegion::VRam),
            MemRegion::IORegs => self.io_read(addr),
            MemRegion::HighRam => unimplemented_read!(MemRegion::HighRam),
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read8(addr), self.read8(addr + 1)])
    }

    pub fn write8(&mut self, addr: u16, value: u8) -> Result<(), WriteError> {
        match self.map_to_region(addr) {
            MemRegion::BootRom => Err(WriteError::ReadOnly(addr)),
            MemRegion::Cartridge => unimplemented_write!(MemRegion::Cartridge),
            MemRegion::WorkRam => {
                T::write(&mut self.ram, addr - 0xC000, value);
                Ok(())
            }
            MemRegion::VRam => unimplemented_write!(MemRegion::VRam),
            MemRegion::IORegs => self.io_write(addr, value),
            MemRegion::HighRam => unimplemented_write!(MemRegion::HighRam),
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
                log::debug!("I/O register not implemented for reading: 0x{:x}", addr);
                0
            }
        }
    }
}

impl<T: GBAllocator> DecoderReadable for MemController<T> {
    fn read_at(&self, idx: usize) -> Option<u8> {
        match u16::try_from(idx) {
            Ok(addr) => Some(self.read8(addr)),
            Err(_) => None,
        }
    }
}
