use std::{fmt::Display, io::Read, num::Wrapping};

use thiserror::Error;

use crate::rom::licensee;

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
    destination: Destination,
    game_version: u8,
    header_checksum: u8,
    header_checksum_valid: bool,
    global_checksum: u16,
}

impl RomMeta {
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn manufacturer(&self) -> Manufacturer {
        self.manufacturer
    }

    pub fn cgb_support(&self) -> CgbFlag {
        self.cgb_flag
    }

    pub fn licensee(&self) -> Licensee {
        self.licensee
    }

    pub fn sgb_support(&self) -> bool {
        self.sgb_flag
    }

    pub fn cartridge_hardware(&self) -> CartridgeHardware {
        self.cartridge_hardware
    }

    pub fn rom_size(&self) -> RomSize {
        self.rom_size
    }

    pub fn ram_size(&self) -> RamSize {
        self.ram_size
    }

    pub fn destination(&self) -> Destination {
        self.destination
    }

    pub fn game_version(&self) -> u8 {
        self.game_version
    }

    pub fn header_checksum(&self) -> u8 {
        self.header_checksum
    }

    pub fn header_checksum_valid(&self) -> bool {
        self.header_checksum_valid
    }

    pub fn global_checksum(&self) -> u16 {
        self.global_checksum
    }
}

fn get_last_nonnull_idx(bytes: &[u8]) -> usize {
    for (idx, byte) in bytes.iter().enumerate().rev() {
        if *byte != 0 {
            return idx;
        }
    }

    0
}

impl RomMeta {
    pub const OFFSET_HEADER_START: usize = 0x100;

    pub const OFFSET_LOGO: usize = 0x104;
    pub const OFFSET_LOGO_START: usize = 0x104 - Self::OFFSET_HEADER_START;

    pub const OFFSET_TITLE: usize = 0x134;
    pub const OFFSET_TITLE_START: usize = 0x134 - Self::OFFSET_HEADER_START;

    pub const OFFSET_MANUFACTURER: usize = 0x13f;
    pub const OFFSET_MANUFACTURER_START: usize = 0x13f - Self::OFFSET_HEADER_START;

    pub const OFFSET_CGB_FLAG: usize = 0x143;
    pub const OFFSET_CGB_FLAG_START: usize = 0x143 - Self::OFFSET_HEADER_START;

    pub const OFFSET_NEW_LICENSEE_CODE: usize = 0x144;
    pub const OFFSET_NEW_LICENSEE_CODE_START: usize = 0x144 - Self::OFFSET_HEADER_START;

    pub const OFFSET_SGB_FLAG: usize = 0x146;
    pub const OFFSET_SGB_FLAG_START: usize = 0x146 - Self::OFFSET_HEADER_START;

    pub const OFFSET_CARTRIDGE_TYPE: usize = 0x147;
    pub const OFFSET_CARTRIDGE_TYPE_START: usize = 0x147 - Self::OFFSET_HEADER_START;

    pub const OFFSET_ROM_SIZE: usize = 0x148;
    pub const OFFSET_ROM_SIZE_START: usize = 0x148 - Self::OFFSET_HEADER_START;

    pub const OFFSET_RAM_SIZE: usize = 0x149;
    pub const OFFSET_RAM_SIZE_START: usize = 0x149 - Self::OFFSET_HEADER_START;

    pub const OFFSET_DESTINATION_CODE: usize = 0x14a;
    pub const OFFSET_DESTINATION_CODE_START: usize = 0x14a - Self::OFFSET_HEADER_START;

    pub const OFFSET_OLD_LICENSEE_CODE: usize = 0x14b;
    pub const OFFSET_OLD_LICENSEE_CODE_START: usize = 0x14b - Self::OFFSET_HEADER_START;

    pub const OFFSET_ROM_VERSION: usize = 0x14c;
    pub const OFFSET_ROM_VERSION_START: usize = 0x14c - Self::OFFSET_HEADER_START;

    pub const OFFSET_HEADER_CHECKSUM: usize = 0x14d;
    pub const OFFSET_HEADER_CHECKSUM_START: usize = 0x14d - Self::OFFSET_HEADER_START;

    pub const OFFSET_GLOBAL_CHECKSUM: usize = 0x14e;
    pub const OFFSET_GLOBAL_CHECKSUM_START: usize = 0x14e - Self::OFFSET_HEADER_START;

    pub const OFFSET_HEADER_END: usize = 0x150;
    pub const OFFSET_HEADER_END_START: usize = 0x150 - Self::OFFSET_HEADER_START;

    pub const HEADER_LENGTH: usize = Self::OFFSET_HEADER_END - Self::OFFSET_HEADER_START;

    pub fn parse(header_bytes: &[u8]) -> Result<Self, RomMetaParseError> {
        if header_bytes.len() < Self::HEADER_LENGTH {
            return Err(RomMetaParseError::TooShort(
                header_bytes.len(),
                Self::HEADER_LENGTH,
            ));
        };

        let title_bytes = header_bytes
            .get(Self::OFFSET_TITLE_START..Self::OFFSET_TITLE_START + 16)
            .unwrap();

        // Filter null padding from end
        let last_nonnull_idx = title_bytes
            .get(0..get_last_nonnull_idx(title_bytes) + 1)
            .unwrap();

        let title = String::from_utf8_lossy(last_nonnull_idx).to_string();

        let manufacturer = Manufacturer::from_raw(
            header_bytes
                .get(Self::OFFSET_MANUFACTURER_START..Self::OFFSET_MANUFACTURER_START + 4)
                .unwrap(),
        )?;

        let cgb_flag = CgbFlag::from(*header_bytes.get(Self::OFFSET_CGB_FLAG_START).unwrap());

        let new_licensee_code: [u8; 2] = [
            *header_bytes
                .get(Self::OFFSET_NEW_LICENSEE_CODE_START)
                .unwrap(),
            *header_bytes
                .get(Self::OFFSET_NEW_LICENSEE_CODE_START + 1)
                .unwrap(),
        ];

        let licensee = Licensee::new(
            *header_bytes
                .get(Self::OFFSET_OLD_LICENSEE_CODE_START)
                .unwrap(),
            new_licensee_code,
        );

        let sgb_flag = header_bytes[Self::OFFSET_SGB_FLAG_START] == 0x3;

        let cartridge_hardware =
            CartridgeHardware::try_from(header_bytes[Self::OFFSET_CARTRIDGE_TYPE_START]).unwrap();

        let rom_size = RomSize::try_from(header_bytes[Self::OFFSET_ROM_SIZE_START]).unwrap();

        let ram_size = RamSize::try_from(header_bytes[Self::OFFSET_RAM_SIZE_START]).unwrap();

        let destination =
            Destination::try_from(header_bytes[Self::OFFSET_DESTINATION_CODE_START]).unwrap();

        let game_version = header_bytes[Self::OFFSET_ROM_VERSION_START];
        let header_checksum = header_bytes[Self::OFFSET_HEADER_CHECKSUM_START];
        let global_checksum = u16::from_be_bytes([
            header_bytes[Self::OFFSET_GLOBAL_CHECKSUM_START],
            header_bytes[Self::OFFSET_GLOBAL_CHECKSUM_START + 1],
        ]);

        let meta = Self {
            title,
            manufacturer,
            cgb_flag,
            licensee,
            sgb_flag,
            cartridge_hardware,
            rom_size,
            ram_size,
            destination,
            game_version,
            header_checksum,
            header_checksum_valid: RomMeta::verify_header_checksum(
                &header_bytes[..Self::OFFSET_HEADER_END_START],
                header_checksum,
            ),
            global_checksum,
        };

        Ok(meta)
    }

    pub fn verify_header_checksum(header_bytes: &[u8], header_checksum: u8) -> bool {
        debug_assert_eq!(Self::HEADER_LENGTH, header_bytes.len());

        let mut computed_checksum = 0u8;

        for byte in &header_bytes[Self::OFFSET_TITLE_START..Self::OFFSET_HEADER_CHECKSUM_START] {
            computed_checksum = computed_checksum.wrapping_add(!byte);
        }

        computed_checksum == header_checksum
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum RomMetaParseError {
    #[error("Too few input bytes: {0}, wanted {1}")]
    TooShort(usize, usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Manufacturer {
    raw: [u8; 4],
}

impl Display for Manufacturer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u32::from_be_bytes(self.raw))
    }
}

impl Manufacturer {
    fn from_raw(raw: &[u8]) -> Result<Self, RomMetaParseError> {
        if raw.len() < 4 {
            return Err(RomMetaParseError::TooShort(raw.len(), 4));
        };

        let raw_buf = [raw[0], raw[1], raw[2], raw[3]];

        Ok(Manufacturer { raw: raw_buf })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Licensee {
    Old { raw: u8 },
    New { raw: [u8; 2] },
}

impl Display for Licensee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Licensee::Old { raw } => match licensee::find_old(*raw) {
                Some(licensee) => write!(f, "{}", licensee.names.join("/")),
                None => write!(f, "0x{:x}", raw),
            },
            Licensee::New { raw } => match licensee::find_new(*raw) {
                Some(licensee) => write!(f, "{}", licensee.names.join("/")),
                None => write!(f, "0x{:x}", u16::from_be_bytes(*raw)),
            },
        }
    }
}

impl Licensee {
    fn new(old_code: u8, new_code: [u8; 2]) -> Self {
        if old_code != 0x33 {
            Self::Old { raw: old_code }
        } else {
            Self::New { raw: new_code }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CgbFlag {
    NoCgb,
    CgbBackwards,
    CgbOnly,
}

impl Display for CgbFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CgbFlag::NoCgb => write!(f, "No CGB support"),
            CgbFlag::CgbBackwards => write!(f, "CGB support (backwards compatible)"),
            CgbFlag::CgbOnly => write!(f, "CGB support (not backwards compatible)"),
        }
    }
}

impl From<u8> for CgbFlag {
    fn from(value: u8) -> Self {
        match value {
            0x80 => Self::CgbBackwards,
            0xC0 => Self::CgbOnly,
            _ => Self::NoCgb,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CartridgeMapper {
    MBC1,
    MBC2,
    MMM01,
    MBC3,
    MBC4,
    MBC5,
    MBC6,
    MBC7,
    HuC1,
    HuC3,
}

impl Display for CartridgeMapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeMapper::MBC1 => write!(f, "MBC1"),
            CartridgeMapper::MBC2 => write!(f, "MBC2"),
            CartridgeMapper::MMM01 => write!(f, "MMM01"),
            CartridgeMapper::MBC3 => write!(f, "MBC3"),
            CartridgeMapper::MBC4 => write!(f, "MBC4"),
            CartridgeMapper::MBC5 => write!(f, "MBC5"),
            CartridgeMapper::MBC6 => write!(f, "MBC6"),
            CartridgeMapper::MBC7 => write!(f, "MBC7"),
            CartridgeMapper::HuC1 => write!(f, "HuC1"),
            CartridgeMapper::HuC3 => write!(f, "HuC3"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CartridgeHardware {
    raw: u8,
    mapper: Option<CartridgeMapper>,
    has_ram: bool,
    has_battery: bool,
    has_timer: bool,
    has_rumble: bool,
    has_sensor: bool,
    has_camera: bool,
}

impl CartridgeHardware {
    pub fn mapper(&self) -> Option<CartridgeMapper> {
        self.mapper
    }

    pub fn has_ram(&self) -> bool {
        self.has_ram
    }

    pub fn has_battery(&self) -> bool {
        self.has_battery
    }

    pub fn has_timer(&self) -> bool {
        self.has_timer
    }

    pub fn has_rumble(&self) -> bool {
        self.has_rumble
    }

    pub fn has_sensor(&self) -> bool {
        self.has_sensor
    }

    pub fn has_camera(&self) -> bool {
        self.has_camera
    }
}

impl TryFrom<u8> for CartridgeHardware {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        let mut hw = Self {
            raw: value,
            mapper: None,
            has_ram: false,
            has_battery: false,
            has_timer: false,
            has_rumble: false,
            has_sensor: false,
            has_camera: false,
        };

        match value {
            0x00 => {}
            0x01 => {
                hw.mapper = Some(CartridgeMapper::MBC1);
            }
            0x02 => {
                hw.mapper = Some(CartridgeMapper::MBC1);
                hw.has_ram = true;
            }
            0x03 => {
                hw.mapper = Some(CartridgeMapper::MBC1);
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0x05 => {
                hw.mapper = Some(CartridgeMapper::MBC2);
            }
            0x06 => {
                hw.mapper = Some(CartridgeMapper::MBC2);
                hw.has_battery = true;
            }
            0x08 => {
                hw.has_ram = true;
            }
            0x09 => {
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0x0B => {
                hw.mapper = Some(CartridgeMapper::MMM01);
            }
            0x0C => {
                hw.mapper = Some(CartridgeMapper::MMM01);
                hw.has_ram = true;
            }
            0x0D => {
                hw.mapper = Some(CartridgeMapper::MMM01);
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0x0F => {
                hw.mapper = Some(CartridgeMapper::MBC3);
                hw.has_timer = true;
                hw.has_battery = true;
            }
            0x10 => {
                hw.mapper = Some(CartridgeMapper::MBC3);
                hw.has_timer = true;
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0x11 => {
                hw.mapper = Some(CartridgeMapper::MBC3);
            }
            0x12 => {
                hw.mapper = Some(CartridgeMapper::MBC3);
                hw.has_ram = true;
            }
            0x13 => {
                hw.mapper = Some(CartridgeMapper::MBC3);
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0x19 => {
                hw.mapper = Some(CartridgeMapper::MBC5);
            }
            0x1A => {
                hw.mapper = Some(CartridgeMapper::MBC5);
                hw.has_ram = true;
            }
            0x1B => {
                hw.mapper = Some(CartridgeMapper::MBC5);
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0x1C => {
                hw.mapper = Some(CartridgeMapper::MBC5);
                hw.has_rumble = true;
            }
            0x1D => {
                hw.mapper = Some(CartridgeMapper::MBC5);
                hw.has_rumble = true;
                hw.has_ram = true;
            }
            0x1E => {
                hw.mapper = Some(CartridgeMapper::MBC5);
                hw.has_rumble = true;
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0x20 => {
                hw.mapper = Some(CartridgeMapper::MBC6);
            }
            0x22 => {
                hw.mapper = Some(CartridgeMapper::MBC7);
                hw.has_sensor = true;
                hw.has_rumble = true;
                hw.has_ram = true;
                hw.has_battery = true;
            }
            0xFC => {
                hw.has_camera = true;
            }
            0xFE => {
                hw.mapper = Some(CartridgeMapper::HuC3);
            }
            0xFF => {
                hw.mapper = Some(CartridgeMapper::HuC1);
                hw.has_ram = true;
                hw.has_battery = false;
            }
            _ => return Err(()),
        };

        Ok(hw)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RomSize {
    raw: u8,
}

impl Display for RomSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} KiB ({} banks)",
            self.in_bytes() / 1024,
            self.num_banks()
        )
    }
}

impl TryFrom<u8> for RomSize {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 8 {
            Ok(Self { raw: value })
        } else {
            Err(())
        }
    }
}

impl RomSize {
    pub const fn in_bytes(&self) -> usize {
        let base_bytes: usize = 1 << 15;

        base_bytes * (1 << self.raw)
    }

    pub const fn num_banks(&self) -> usize {
        const KB: usize = 1024;
        const BANK_SIZE: usize = 32 * KB;

        self.in_bytes() / BANK_SIZE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RamSize {
    raw: u8,
}

impl Display for RamSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} KiB ({} banks)",
            self.in_bytes() / 1024,
            self.num_banks()
        )
    }
}

impl RamSize {
    pub const fn in_bytes(&self) -> usize {
        const KB: usize = 1024;

        match self.raw {
            0x0 => 0,
            0x2 => 8 * KB,
            0x3 => 32 * KB,
            0x4 => 128 * KB,
            0x5 => 64 * KB,
            _ => panic!("Invalid RamSize value"),
        }
    }

    pub const fn num_banks(&self) -> usize {
        const KB: usize = 1024;

        self.in_bytes() / (8 * KB)
    }
}

impl TryFrom<u8> for RamSize {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0x1 || value > 0x5 {
            Err(())
        } else {
            Ok(Self { raw: value })
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Destination {
    Japan = 0,
    Elsewhere = 1,
}

impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dest = match self {
            Destination::Japan => "Japan",
            Destination::Elsewhere => "Elsewhere",
        };

        write!(f, "{}", dest)
    }
}

impl TryFrom<u8> for Destination {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Destination::Japan),
            0x1 => Ok(Destination::Elsewhere),
            _ => Err(()),
        }
    }
}
