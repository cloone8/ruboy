use crate::{
    allocator::{GBAllocator, GBRam},
    rom::{meta::RomMeta, RomReader},
};

use super::{Mbc, ReadError, WriteError};

#[derive(Debug)]
pub struct NonBankingController<A: GBAllocator> {
    meta: RomMeta,
    rom_content: A::Mem<0x8000>,
    ram_content: A::Mem<0x2000>,
}

impl<A: GBAllocator> NonBankingController<A> {
    pub fn new<R: RomReader>(meta: RomMeta, mut reader: R) -> Result<Self, R::Err> {
        let mut new = Self {
            meta,
            rom_content: A::allocate(),
            ram_content: A::allocate(),
        };

        reader.read_into(new.rom_content.raw_mut(), 0)?;

        Ok(new)
    }
}

impl<A: GBAllocator> Mbc for NonBankingController<A> {
    fn read(&self, addr: u16) -> Result<u8, ReadError> {
        match addr {
            0x0000..=0x7FFF => Ok(self.rom_content.read(addr)),
            0xA000..=0xBFFF => {
                let ram_index = addr - 0xA000;
                let ram_size = self.meta.ram_size().in_bytes();

                if (ram_index as usize) < ram_size {
                    Ok(self.ram_content.read(ram_index))
                } else {
                    Err(ReadError::NotEnoughRam {
                        addr: ram_index,
                        max: ram_size as u16,
                    })
                }
            }
            _ => panic!("Address not a ROM address"),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> Result<(), WriteError> {
        match addr {
            0x0000..=0x7FFF => Err(WriteError::ReadOnly(addr)),
            0xA000..=0xBFFF => {
                let ram_index = addr - 0xA000;
                let ram_size = self.meta.ram_size().in_bytes();

                if (ram_index as usize) < ram_size {
                    self.ram_content.write(ram_index, val);
                    Ok(())
                } else {
                    Err(WriteError::NotEnoughRam {
                        addr: ram_index,
                        max: ram_size as u16,
                    })
                }
            }
            _ => panic!("Address not a ROM address"),
        }
    }
}
