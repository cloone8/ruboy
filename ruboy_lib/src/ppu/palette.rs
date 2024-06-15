use crate::{GbColorID, GbMonoColor};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Palette {
    val: u8,
}

impl Palette {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn get_id(self, id: GbColorID) -> GbMonoColor {
        match id {
            GbColorID::ID0 => bits_to_color(self.val & 0b11),
            GbColorID::ID1 => bits_to_color((self.val & 0b1100) >> 2),
            GbColorID::ID2 => bits_to_color((self.val & 0b110000) >> 4),
            GbColorID::ID3 => bits_to_color((self.val & 0b11000000) >> 6),
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
