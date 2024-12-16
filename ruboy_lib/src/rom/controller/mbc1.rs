use crate::rom::controller::bank_num_to_addr;
use crate::rom::meta::RomMeta;
use crate::{GBAllocator, GBRam, RomReader};

use super::{Mbc, ReadError, WriteError};

#[derive(Debug)]
pub struct Mbc1<A: GBAllocator, R: RomReader> {
    meta: RomMeta,
    reader: R,

    /// All banks ending with 0 (00, 10, 20, etc.)
    rom_bank_x0: A::Mem<u8, 0x4000>,

    /// All other banks
    rom_bank_1x: A::Mem<u8, 0x4000>,

    ram_bank_x: A::Mem<u8, 0x2000>,

    ram_enabled: bool,

    addressing_mode: AddrMode,

    selected_bank: u8,

    secondary_bank: u8,
}

#[derive(Debug, Clone, Copy)]
enum AddrMode {
    Mode0,
    Mode1,
}

impl<A: GBAllocator, R: RomReader> Mbc1<A, R> {
    pub fn new(meta: RomMeta, mut reader: R) -> Result<Self, R::Err> {
        log::info!("Initializing MBC1 ROM mapper");

        let mut bank_0 = A::empty();
        let mut bank_1 = A::empty();

        reader.read_into(bank_0.raw_mut(), bank_num_to_addr(0))?;
        reader.read_into(bank_1.raw_mut(), bank_num_to_addr(1))?;

        let new = Self {
            meta,
            reader,
            rom_bank_x0: bank_0,
            rom_bank_1x: bank_1,
            ram_bank_x: A::empty(),
            ram_enabled: false,
            addressing_mode: AddrMode::Mode0,
            selected_bank: 0,
            secondary_bank: 0,
        };

        Ok(new)
    }

    fn switch_rom_bank(&mut self, bank: usize) -> Result<(), R::Err> {
        self.reader
            .read_into(self.rom_bank_1x.raw_mut(), bank_num_to_addr(bank))?;

        Ok(())
    }

    fn switch_ram_bank(&mut self, bank: usize) {
        //TODO: Save previous bank somewhere?
    }

    fn calc_rom_bank(&self) -> usize {
        assert!(self.selected_bank <= 0b11111, "ROM bank too high, invalid!");
        assert!(
            self.secondary_bank <= 0b11,
            "ROM secondary bank too high, invalid!"
        );

        let actual_bank = self.selected_bank + (self.secondary_bank << 5);

        (actual_bank as usize) % self.meta.rom_size().num_banks()
    }
}

impl<A: GBAllocator, R: RomReader> Mbc for Mbc1<A, R> {
    fn read(&self, addr: u16) -> Result<u8, super::ReadError> {
        match addr {
            0x0000..=0x3FFF => match self.addressing_mode {
                AddrMode::Mode0 => Ok(self.rom_bank_x0.read(addr)),
                AddrMode::Mode1 => todo!(),
            },
            0x4000..=0x7FFF => Ok(self.rom_bank_1x.read(addr - 0x4000)),
            0xA000..=0xBFFF => {
                let ram_size = self.meta.ram_size().in_bytes();
                if ram_size == 0 {
                    return Err(ReadError::NotEnoughRam { addr, max: 0 });
                }

                if self.ram_enabled {
                    let ram_addr = match self.addressing_mode {
                        AddrMode::Mode0 => addr - 0xA000,
                        AddrMode::Mode1 => todo!(),
                    };

                    Ok(self.ram_bank_x.read(ram_addr))
                } else {
                    Ok(0xFF)
                }
            }
            _ => panic!("Address not a ROM address"),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> Result<(), super::WriteError> {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = val & 0x0F == 0xA;
                Ok(())
            }
            0x2000..=0x3FFF => {
                // 5-bit register
                let mut bank_num = val & 0b11111;
                if bank_num == 0 {
                    bank_num = 1;
                }

                self.selected_bank = bank_num;
                self.switch_rom_bank(self.calc_rom_bank())
                    .map_err(|e| WriteError::Reader(Box::new(e)))?;

                Ok(())
            }
            0x4000..=0x5FFF => {
                const MB: usize = 1024 * 1024;
                let rom_size = self.meta.rom_size().in_bytes();
                let ram_size = self.meta.ram_size().in_bytes();

                if rom_size >= MB {
                    self.secondary_bank = val & 0b11;
                    self.switch_rom_bank(self.calc_rom_bank())
                        .map_err(|e| WriteError::Reader(Box::new(e)))?;
                } else if ram_size > 0 {
                    self.switch_ram_bank((val & 0b11) as usize);
                }
                // else do nothing

                Ok(())
            }
            0x6000..=0x7FFF => {
                if val & 0b1 == 0b1 {
                    self.addressing_mode = AddrMode::Mode1;
                } else {
                    self.addressing_mode = AddrMode::Mode0;
                }

                Ok(())
            }
            _ => panic!("Address not a ROM address"),
        }
    }
}
