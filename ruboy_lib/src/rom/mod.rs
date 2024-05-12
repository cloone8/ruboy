use std::io::Read;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RomMeta {
    title: String,
    manufacturer: Manufacturer,
    cgb_flag: CgbFlag,
    licensee: Licensee,
    sgb_flag: bool,
    cartridge_hardware: CartridgeHardware,
    rom_size: RomSize,
    ram_size: RamSize,
    game_version: u8,
    header_checksum: u8,
    checksum_valid: bool,
    global_checksum: u16,
}

impl RomMeta {
    pub const OFFSET_HEADER_START: u16 = 0x100;
    pub const OFFSET_LOGO: u16 = 0x104;
    pub const OFFSET_TITLE: u16 = 0x134;
    pub const OFFSET_MANUFACTURER: u16 = 0x13f;
    pub const OFFSET_CGB_FLAG: u16 = 0x143;
    pub const OFFSET_NEW_LICENSEE_CODE: u16 = 0x144;
    pub const OFFSET_SGB_FLAG: u16 = 0x146;
    pub const OFFSET_CARTRIDGE_TYPE: u16 = 0x147;
    pub const OFFSET_ROM_SIZE: u16 = 0x148;
    pub const OFFSET_RAM_SIZE: u16 = 0x149;
    pub const OFFSET_DESTINATION_CODE: u16 = 0x14a;
    pub const OFFSET_OLD_LICENSEE_CODE: u16 = 0x14b;
    pub const OFFSET_ROM_VERSION: u16 = 0x14c;
    pub const OFFSET_HEADER_CHECKSUM: u16 = 0x14d;
    pub const OFFSET_GLOBAL_CHECKSUM: u16 = 0x14e;
    pub const OFFSET_HEADER_END: u16 = 0x14f;

    pub const HEADER_LENGTH: u16 = Self::OFFSET_HEADER_END - Self::OFFSET_HEADER_START;

    pub fn parse(header_bytes: &[u8]) -> Result<Self, HeaderParseError> {
        if header_bytes.len() < Self::HEADER_LENGTH as usize {
            return Err(HeaderParseError::TooShort(header_bytes.len()));
        };

        let meta = Self {
            title: todo!(),
            manufacturer: todo!(),
            cgb_flag: todo!(),
            licensee: todo!(),
            sgb_flag: todo!(),
            cartridge_hardware: todo!(),
            rom_size: todo!(),
            ram_size: todo!(),
            game_version: todo!(),
            header_checksum: todo!(),
            checksum_valid: todo!(),
            global_checksum: todo!(),
        };

        Ok(meta)
    }
}

#[derive(Debug, Error)]
pub enum HeaderParseError {
    #[error("Too few input bytes: {0}, wanted {:x}", RomMeta::HEADER_LENGTH)]
    TooShort(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Manufacturer {
    raw: [u8; 4],
}

#[derive(Debug, Clone, Copy)]
pub enum Licensee {
    Old { raw: u8 },
    New { raw: [u8; 2] },
}

#[derive(Debug, Clone, Copy)]
pub enum CgbFlag {
    NoCgb,
    CgbBackwards,
    CgbOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct CartridgeHardware {
    raw: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct RomSize {
    raw: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct RamSize {
    raw: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Destination {
    Japan = 0,
    Elsewhere = 1,
}
