use core::mem::size_of;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Tile([u8; 16]);

impl Tile {
    pub const X_SIZE: usize = 8;
    pub const Y_SIZE: usize = 8;

    #[inline]
    pub const fn get_lower_for_row(self, row: u8) -> u8 {
        debug_assert!((row as usize) < Self::Y_SIZE);

        self.0[(row * 2) as usize]
    }

    #[inline]
    pub const fn get_upper_for_row(self, row: u8) -> u8 {
        debug_assert!((row as usize) < Self::Y_SIZE);

        self.0[((row * 2) + 1) as usize]
    }
}

impl From<[u8; size_of::<Tile>()]> for Tile {
    fn from(value: [u8; size_of::<Tile>()]) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn tile_size() {
        assert_eq!(16, size_of::<Tile>());
    }
}
