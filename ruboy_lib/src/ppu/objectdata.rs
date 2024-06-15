use core::mem::size_of;

#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct ObjectData([u8; 4]);

impl ObjectData {
    pub const fn y_pos(self) -> u8 {
        self.0[0]
    }

    pub const fn offset_ypos(self) -> i16 {
        ((self.y_pos() as u16) as i16) - 8
    }

    pub const fn x_pos(self) -> u8 {
        self.0[1]
    }

    pub const fn offset_xpos(self) -> i16 {
        ((self.x_pos() as u16) as i16) - 8
    }

    pub const fn tilenum(self) -> u8 {
        self.0[2]
    }

    pub const fn flags(self) -> ObjDataFlags {
        ObjDataFlags::from_byte(self.0[3])
    }
}

impl From<[u8; size_of::<ObjectData>()]> for ObjectData {
    fn from(value: [u8; size_of::<ObjectData>()]) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct ObjDataFlags(u8);

impl ObjDataFlags {
    pub const fn from_byte(val: u8) -> ObjDataFlags {
        Self(val)
    }
}

impl From<u8> for ObjDataFlags {
    fn from(value: u8) -> Self {
        Self::from_byte(value)
    }
}

impl From<ObjDataFlags> for u8 {
    fn from(value: ObjDataFlags) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn object_data_size() {
        assert_eq!(4, size_of::<ObjectData>());
    }
}
