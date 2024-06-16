use crate::{memcontroller::MemController, GBAllocator, GbColorID, GbMonoColor, RomReader};

#[derive(Debug, Clone, Copy)]
pub enum PaletteID {
    Zero,
    One,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Palette {
    val: u8,
}

impl Palette {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn make_color(self, id: GbColorID) -> GbMonoColor {
        match id {
            GbColorID::ID0 => bits_to_color(self.val & 0b11),
            GbColorID::ID1 => bits_to_color((self.val & 0b1100) >> 2),
            GbColorID::ID2 => bits_to_color((self.val & 0b110000) >> 4),
            GbColorID::ID3 => bits_to_color((self.val & 0b11000000) >> 6),
        }
    }

    pub fn load_bg(mem: &MemController<impl GBAllocator, impl RomReader>) -> Palette {
        mem.io_registers.bg_palette
    }

    pub fn load_obj(
        id: PaletteID,
        mem: &MemController<impl GBAllocator, impl RomReader>,
    ) -> Palette {
        match id {
            PaletteID::Zero => mem.io_registers.obj0_palette,
            PaletteID::One => mem.io_registers.obj1_palette,
        }
    }
}

impl From<u8> for Palette {
    fn from(value: u8) -> Self {
        Self { val: value }
    }
}

impl From<Palette> for u8 {
    fn from(value: Palette) -> Self {
        value.val
    }
}

const fn bits_to_color(bits: u8) -> GbMonoColor {
    debug_assert!((!0b11_u8) & bits == 0); // Colors must be expressed with 2-bit values

    match bits {
        0 => GbMonoColor::White,
        1 => GbMonoColor::LightGray,
        2 => GbMonoColor::DarkGray,
        3 => GbMonoColor::Black,
        _ => panic!("Invalid color bits!"),
    }
}
