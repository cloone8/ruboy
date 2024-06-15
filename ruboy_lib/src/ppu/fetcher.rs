use core::mem::size_of;

use thiserror::Error;

use crate::{
    memcontroller::{MemController, ReadError},
    ppu::tilemap,
    GBAllocator, GbColorID, GbMonoColor, RomReader,
};

use super::{inlinequeue::InlineQueue, objectdata::ObjectData, tile::Tile, PaletteID};

#[derive(Debug)]
pub struct PixelFetcher {
    cycles_left: u8,
    x_pos: u8,
    window_lines_drawn: u8,
    win_x_reached: bool,
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
    color: GbColorID,
    palette: PaletteID,
    priority: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum FetchedPixels {
    Background([FetchedPixel; 8]),
    Object([FetchedPixel; 8]),
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
            phase: Phase::FetchTile,
            bg_fifo: InlineQueue::new(),
            obj_fifo: InlineQueue::new(),
        }
    }

    pub fn vblank_reset(&mut self) {
        self.window_lines_drawn = 0;
    }

    fn fetch_tile(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        fetching_window: bool,
    ) -> Result<(), FetchTileErr> {
        let tilemap_base: u16 = match fetching_window {
            true => {
                if mem.io_registers.lcd_control.window_tilemap_area() {
                    0x9C00
                } else {
                    0x9800
                }
            }
            false => {
                if mem.io_registers.lcd_control.bg_tilemap_area() {
                    0x9C00
                } else {
                    0x9800
                }
            }
        };

        let (x, y) = match fetching_window {
            true => {
                todo!()
            }
            false => (
                ((mem.io_registers.scx / 8) + self.x_pos) & 0x1F,
                (mem.io_registers.lcd_y + mem.io_registers.scy),
            ),
        };

        debug_assert!(x <= 31, "tile X wrong size: {}", x);

        let tile_offset = tilemap::calc_offset(x, y) & 0x3FF; // AND mask make sure that it stays within the bounds of the tilemap

        let tile_addr = tilemap_base + tile_offset;
        let tile_idx = mem.read8(tile_addr)?;

        self.phase = Phase::FetchDataLow(FetchDataLowData { tile_idx });
        Ok(())
    }

    fn fetch_data_low(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), FetchDataErr> {
        let data = match &mut self.phase {
            Phase::FetchDataLow(data) => data,
            _ => panic!("Invalid mode for fetch_data_low!"),
        };

        let tile = get_tile_by_idx(data.tile_idx, mem)?;
        let tile_line = mem.io_registers.lcd_y % (Tile::Y_SIZE as u8);

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
        let data = match &mut self.phase {
            Phase::FetchDataHigh(data) => data,
            _ => panic!("Invalid mode for fetch_data_high!"),
        };

        let tile = get_tile_by_idx(data.tile_idx, mem)?;
        let tile_line = mem.io_registers.lcd_y % (Tile::Y_SIZE as u8);

        let pix_lower = data.lower;
        let pix_upper = tile.get_upper_for_row(tile_line);

        let pix_ids: [GbColorID; 8] = std::array::from_fn(|i| {
            let id_mask: u8 = 0b1 << i;
            let lower_val = (pix_lower & id_mask) >> i;
            let upper_val = (pix_upper & id_mask) >> i;

            debug_assert!(lower_val == 0 || lower_val == 1);
            debug_assert!(upper_val == 0 || upper_val == 1);

            GbColorID::try_from(lower_val + (upper_val << 1)).unwrap()
        });

        self.phase = Phase::Sleep(pix_ids);

        Ok(())
    }

    fn push(
        &mut self,
        mem: &MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), PushErr> {
        if self.bg_fifo.space_remaining() < 8 {
            return Ok(());
        }

        let pixels = match self.phase {
            Phase::Push(pixels) => pixels,
            _ => panic!("Invalid mode for push!"),
        };

        self.bg_fifo.push_n(pixels).unwrap();

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
        objs: &[ObjectData],
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
            Phase::Push(_) => self.push(mem)?,
        }

        self.cycles_left = match self.phase {
            Phase::Push(_) => 0, // Push is repeated every cycle until succesful
            _ => 1,
        };

        Ok(())
    }
}

fn get_tile_by_idx(
    tile_idx: u8,
    mem: &MemController<impl GBAllocator, impl RomReader>,
) -> Result<Tile, ReadError> {
    let addressing_mode = mem.io_registers.lcd_control.bg_window_tile_area();

    let tile_addr_usize = match addressing_mode {
        true => 0x8000 + ((tile_idx as usize) * size_of::<Tile>()),
        false => {
            0x9000_usize.wrapping_add_signed(tile_idx as i8 as isize * (size_of::<Tile>() as isize))
        }
    };

    let tile_addr = u16::try_from(tile_addr_usize).unwrap();

    let tile_bytes: [u8; size_of::<Tile>()] = mem.read_range(tile_addr)?;

    Ok(tile_bytes.into())
}
