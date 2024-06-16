const TILEMAP_X_SIZE: usize = 32;
const TILEMAP_Y_SIZE: usize = 32;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TileMap([u8; TILEMAP_X_SIZE * TILEMAP_Y_SIZE]);

/// Given an X and Y coordinate within a tilemap,
/// calculates the addres offset of those coordinates from
/// the tilemap base
#[inline]
pub fn calc_offset(x: u8, y: u8) -> u16 {
    let t_x = TILEMAP_X_SIZE as u16;
    let x_16 = x as u16;
    let y_16 = y as u16;

    (t_x * y_16) + x_16
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn tilemap_size() {
        assert_eq!(TILEMAP_X_SIZE * TILEMAP_Y_SIZE, size_of::<TileMap>());
    }
}
