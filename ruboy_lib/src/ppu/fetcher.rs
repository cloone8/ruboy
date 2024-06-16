use core::mem::size_of;

use thiserror::Error;

use crate::{
    memcontroller::{MemController, ReadError},
    ppu::tilemap,
    GBAllocator, GbColorID, RomReader,
};

use super::{inlinequeue::InlineQueue, objectdata::ObjectData, palette::PaletteID, tile::Tile};

#[derive(Debug)]
pub struct PixelFetcher {
    cycles_left: u8,
    x_pos: u8,
    window_lines_drawn: u8,
    win_x_reached: bool,
    object_to_fetch: Option<ObjectData>,
    bg_fifo: InlineQueue<GbColorID, 16>,
    obj_fifo: InlineQueue<FetchedPixel, 8>,
    phase: Phase,
}

#[derive(Debug, Clone, Copy)]
struct FetchDataLowData {
    pub tile_idx: u8,
}

#[derive(Debug, Clone, Copy)]
struct FetchDataHighData {
    pub tile_idx: u8,
    pub lower: u8,
}

#[derive(Debug, Clone, Copy)]
enum Phase {
    FetchTile,
    FetchDataLow(FetchDataLowData),
    FetchDataHigh(FetchDataHighData),
    Sleep([GbColorID; 8]),
    Push([GbColorID; 8]),
}

#[derive(Debug, Clone, Copy)]
pub struct FetchedPixel {
    pub color: GbColorID,
    pub palette_id: PaletteID,
    pub prio_always: bool,
}

#[derive(Debug, Error)]
pub enum FetcherErr {
    #[error("Error during tile number fetching: {0}")]
    FetchTile(#[from] FetchTileErr),

    #[error("Error during tile data fetching: {0}")]
    FetchData(#[from] FetchDataErr),

    #[error("Error during pixel pushing: {0}")]
    Push(#[from] PushErr),
}

#[derive(Debug, Error)]
pub enum FetchTileErr {
    #[error("Error from memcontroller: {0}")]
    Read(#[from] ReadError),
}

#[derive(Debug, Error)]
pub enum FetchDataErr {
    #[error("Error from memcontroller: {0}")]
    Read(#[from] ReadError),
}

#[derive(Debug, Error)]
pub enum PushErr {}

impl PixelFetcher {
    pub fn new() -> Self {
        Self {
            cycles_left: 0,
            x_pos: 0,
            win_x_reached: false,
            window_lines_drawn: 0,
            object_to_fetch: None,
            phase: Phase::FetchTile,
            bg_fifo: InlineQueue::new(),
            obj_fifo: InlineQueue::new(),
        }
    }

    pub fn is_fetching_obj(&self) -> bool {
        self.object_to_fetch.is_some()
    }

    pub fn fetch_obj(&mut self, obj: ObjectData) {
        self.object_to_fetch = Some(obj);
        self.cycles_left = 0;
        self.phase = Phase::FetchTile;
    }

    pub fn vblank_reset(&mut self) {
        self.window_lines_drawn = 0;
        self.hblank_reset();
    }

    pub fn hblank_reset(&mut self) {
        self.x_pos = 0;
        self.bg_fifo.clear();
        self.obj_fifo.clear();
    }

    fn fetch_obj_tile(&mut self) -> Result<(), FetchTileErr> {
        let obj = self.object_to_fetch.unwrap();

        self.phase = Phase::FetchDataLow(FetchDataLowData {
            tile_idx: obj.tilenum(),
        });

        Ok(())
    }

    fn fetch_win_tile(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), FetchTileErr> {
        let tilemap_base: u16 = if mem.io_registers.lcd_control.window_tilemap_area() {
            0x9C00
        } else {
            0x9800
        };

        let (x, y) = todo!();

        debug_assert!(x <= 31, "tile X wrong size: {}", x);

        let tile_offset = tilemap::calc_offset(x, y) & 0x3FF; // AND mask make sure that it stays within the bounds of the tilemap

        let tile_addr = tilemap_base + tile_offset;
        let tile_idx = mem.read8(tile_addr)?;

        self.phase = Phase::FetchDataLow(FetchDataLowData { tile_idx });
        Ok(())
    }

    fn fetch_bg_tile(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), FetchTileErr> {
        let tilemap_base: u16 = if mem.io_registers.lcd_control.bg_tilemap_area() {
            0x9C00
        } else {
            0x9800
        };

        let (x, y) = (
            ((mem.io_registers.scx / 8) + self.x_pos) & 0x1F,
            ((mem.io_registers.scy + mem.io_registers.lcd_y) / 8),
        );

        debug_assert!(x <= 31, "tile X wrong size: {}", x);

        let tile_offset = tilemap::calc_offset(x, y) & 0x3FF; // AND mask make sure that it stays within the bounds of the tilemap

        let tile_addr = tilemap_base + tile_offset;
        let tile_idx = mem.read8(tile_addr)?;

        self.phase = Phase::FetchDataLow(FetchDataLowData { tile_idx });
        Ok(())
    }

    fn fetch_tile(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        fetching_window: bool,
    ) -> Result<(), FetchTileErr> {
        if self.is_fetching_obj() {
            self.fetch_obj_tile()
        } else if fetching_window {
            self.fetch_win_tile(mem)
        } else {
            self.fetch_bg_tile(mem)
        }
    }

    fn fetch_data_low(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), FetchDataErr> {
        let is_obj = self.is_fetching_obj();
        let data = match &mut self.phase {
            Phase::FetchDataLow(data) => data,
            _ => panic!("Invalid mode for fetch_data_low!"),
        };

        let tile = get_tile_by_idx(is_obj, data.tile_idx, mem)?;
        let tile_line = (mem.io_registers.lcd_y + mem.io_registers.scy) % (Tile::Y_SIZE as u8);

        let pix_lower = tile.get_lower_for_row(tile_line);

        self.phase = Phase::FetchDataHigh(FetchDataHighData {
            tile_idx: data.tile_idx,
            lower: pix_lower,
        });

        Ok(())
    }

    fn fetch_data_high(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), FetchDataErr> {
        let is_obj = self.is_fetching_obj();
        let data = match &mut self.phase {
            Phase::FetchDataHigh(data) => data,
            _ => panic!("Invalid mode for fetch_data_high!"),
        };

        let tile = get_tile_by_idx(is_obj, data.tile_idx, mem)?;
        let tile_line = (mem.io_registers.lcd_y + mem.io_registers.scy) % (Tile::Y_SIZE as u8);

        let pix_lower = data.lower;
        let pix_upper = tile.get_upper_for_row(tile_line);

        let pix_ids: [GbColorID; 8] =
            std::array::from_fn(|i| combine_pixdata(pix_lower, pix_upper, i));

        self.phase = Phase::Sleep(pix_ids);

        Ok(())
    }

    fn push(&mut self) -> Result<(), PushErr> {
        if self.bg_fifo.space_remaining() < 8 {
            return Ok(());
        }

        let mut pixels = match self.phase {
            Phase::Push(pixels) => pixels,
            _ => panic!("Invalid mode for push!"),
        };

        pixels.reverse();

        if self.is_fetching_obj() {
            let obj = self.object_to_fetch.take().unwrap();
            let occupied_slots = self.obj_fifo.len();

            if obj.flags().x_flip() {
                pixels.reverse();
            }

            // TODO: Obj y-flip

            pixels.into_iter().skip(occupied_slots).for_each(|pix| {
                self.obj_fifo
                    .push(FetchedPixel {
                        color: pix,
                        palette_id: obj.flags().palette(),
                        prio_always: obj.flags().prio_always(),
                    })
                    .unwrap()
            });
        } else {
            self.bg_fifo.push_n(pixels).unwrap();
            self.x_pos += 1;
        }

        self.phase = Phase::FetchTile;

        Ok(())
    }

    #[inline]
    pub fn get_bg_fifo_mut(&mut self) -> &mut InlineQueue<GbColorID, 16> {
        &mut self.bg_fifo
    }

    #[inline]
    pub fn get_obj_fifo_mut(&mut self) -> &mut InlineQueue<FetchedPixel, 8> {
        &mut self.obj_fifo
    }

    #[inline]
    pub fn get_bg_fifo(&self) -> &InlineQueue<GbColorID, 16> {
        &self.bg_fifo
    }

    #[inline]
    pub fn get_obj_fifo(&self) -> &InlineQueue<FetchedPixel, 8> {
        &self.obj_fifo
    }

    pub fn run_cycle(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        fetching_window: bool,
    ) -> Result<(), FetcherErr> {
        if self.cycles_left != 0 {
            self.cycles_left -= 1;
            return Ok(());
        }

        match self.phase {
            Phase::FetchTile => self.fetch_tile(mem, fetching_window)?,
            Phase::FetchDataLow(_) => self.fetch_data_low(mem)?,
            Phase::FetchDataHigh(_) => self.fetch_data_high(mem)?,
            Phase::Sleep(colors) => self.phase = Phase::Push(colors),
            Phase::Push(_) => self.push()?,
        }

        self.cycles_left = match self.phase {
            Phase::Push(_) => 0, // Push is repeated every cycle until succesful
            _ => 1,
        };

        Ok(())
    }
}

fn addr_from_tile_idx(tile_idx: u8, addressing_mode: bool) -> u16 {
    let tile_addr_usize = match addressing_mode {
        true => 0x8000 + ((tile_idx as usize) * size_of::<Tile>()),
        false => {
            0x9000_usize.wrapping_add_signed(tile_idx as i8 as isize * (size_of::<Tile>() as isize))
        }
    };

    u16::try_from(tile_addr_usize).unwrap()
}

fn get_tile_by_idx(
    is_obj: bool,
    tile_idx: u8,
    mem: &MemController<impl GBAllocator, impl RomReader>,
) -> Result<Tile, ReadError> {
    let addressing_mode = is_obj || mem.io_registers.lcd_control.bg_window_tile_area();
    let tile_addr = addr_from_tile_idx(tile_idx, addressing_mode);

    // log::info!("Getting tile {} at 0x{:x}", tile_idx, tile_addr);

    let tile_bytes: [u8; size_of::<Tile>()] = mem.read_range(tile_addr)?;

    Ok(tile_bytes.into())
}

fn combine_pixdata(lower: u8, upper: u8, idx: usize) -> GbColorID {
    debug_assert!(idx < 8);

    let id_mask: u8 = 0b1 << idx;
    let lower_val = (lower & id_mask) >> idx;
    let upper_val = (upper & id_mask) >> idx;

    debug_assert!(lower_val == 0 || lower_val == 1);
    debug_assert!(upper_val == 0 || upper_val == 1);

    GbColorID::try_from(lower_val + (upper_val << 1)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pix_combine() {
        let lower = 0b10100101_u8;
        let upper = 0b11000011_u8;

        assert_eq!(GbColorID::ID3, combine_pixdata(lower, upper, 0));
        assert_eq!(GbColorID::ID2, combine_pixdata(lower, upper, 1));
        assert_eq!(GbColorID::ID1, combine_pixdata(lower, upper, 2));
        assert_eq!(GbColorID::ID0, combine_pixdata(lower, upper, 3));
        assert_eq!(GbColorID::ID0, combine_pixdata(lower, upper, 4));
        assert_eq!(GbColorID::ID1, combine_pixdata(lower, upper, 5));
        assert_eq!(GbColorID::ID2, combine_pixdata(lower, upper, 6));
        assert_eq!(GbColorID::ID3, combine_pixdata(lower, upper, 7));
    }

    #[test]
    fn test_addressing_mode_zero() {
        assert_eq!(0x9000, addr_from_tile_idx(0, false));
        assert_eq!(0x8800, addr_from_tile_idx(128, false));
        assert_eq!(
            (0x9000 - size_of::<Tile>() as u16),
            addr_from_tile_idx(255, false)
        );
    }

    #[test]
    fn test_addressing_mode_one() {
        assert_eq!(0x8000, addr_from_tile_idx(0, true));
        assert_eq!(0x8800, addr_from_tile_idx(128, true));
        assert_eq!(
            (0x9000 - size_of::<Tile>() as u16),
            addr_from_tile_idx(255, true)
        );
    }
}
