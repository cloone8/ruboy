use core::mem::size_of;

use fetcher::{FetcherErr, PixelFetcher};
use objectdata::ObjectData;
use palette::Palette;
use thiserror::Error;

use crate::{
    extern_traits::{Frame, GBAllocator, GBGraphicsDrawer, RomReader, FRAME_X, FRAME_Y},
    memcontroller::{MemController, ReadError, OAM_START},
    GbColorID, GbMonoColor,
};

mod fetcher;
mod inlinequeue;
mod objectdata;
pub mod palette;
mod tile;
mod tilemap;

const OAM_CYCLES: usize = 80;
const SCANLINE_CYCLES: usize = 456;
const FRAME_CYCLES: usize = SCANLINE_CYCLES * (FRAME_Y + 8);

#[derive(Debug, Clone)]
enum PpuMode {
    Inactive,
    /// Mode 0
    HBlank,

    /// Mode 1
    VBlank,

    /// Mode 2
    OAMScan(OAMScanData),

    /// Mode 3
    Draw(DrawData),
}

#[derive(Debug, Clone)]
struct OAMScanData {
    buffer: [ObjectData; 10],
    num_in_buf: u8,
    cur_obj_index: u8,
    cycles_left: u8,
    window_check_done: bool,
}

impl OAMScanData {
    pub fn new() -> Self {
        Self {
            buffer: [ObjectData::default(); 10],
            num_in_buf: 0,
            cur_obj_index: 0,
            cycles_left: 0,
            window_check_done: false,
        }
    }
}

#[derive(Debug, Clone)]
struct DrawData {
    pix_to_discard: u8,
    pushed_pixels: u8,
    fetcher_cycles_left: u8,
    num_in_buf: u8,
    buffer: [ObjectData; 10],
}

impl DrawData {
    pub fn new(obj_buffer: [ObjectData; 10], num_in_buf: u8, to_discard: u8) -> Self {
        log::trace!("Starting line draw, discarding {} pixels", to_discard);
        Self {
            pix_to_discard: to_discard,
            pushed_pixels: 0,
            fetcher_cycles_left: 0,
            buffer: obj_buffer,
            num_in_buf,
        }
    }
}

const NUM_OAM_OBJECTS: u8 = 40;

#[derive(Debug, Error)]
pub enum PpuErr<V: GBGraphicsDrawer> {
    #[error("Error during HBlank: {0}")]
    HBlank(#[from] HBlankErr),

    #[error("Error during VBlank: {0}")]
    VBlank(#[from] VBlankErr<V>),

    #[error("Error during OAM scan: {0}")]
    OAMScan(#[from] OAMScanErr),

    #[error("Error during drawing: {0}")]
    Draw(#[from] DrawErr),
}

#[derive(Debug, Error)]
pub enum HBlankErr {}

#[derive(Debug, Error)]
pub enum VBlankErr<V: GBGraphicsDrawer> {
    #[error("Error returned from graphics output: {0}")]
    OutputErr(#[source] V::Err),
}

#[derive(Debug, Error)]
pub enum OAMScanErr {
    #[error("Error during memory read: {0}")]
    MemRead(#[from] ReadError),
}

#[derive(Debug, Error)]
pub enum DrawErr {
    #[error("Error during pixel fetcher cycle: {0}")]
    Fetcher(#[from] FetcherErr),
}

#[derive(Debug)]
pub struct Ppu<V: GBGraphicsDrawer> {
    output: V,
    mode: PpuMode,
    framebuf: Frame,
    line_data: LineData,
    frame_data: FrameData,
    pix_fetcher: PixelFetcher,
}

#[derive(Debug, Clone, Copy)]
struct LineData {
    /// The number of the current cycle. First cycle is 1, etc.
    cur_cycle: usize,
}

impl LineData {
    pub fn new() -> Self {
        LineData { cur_cycle: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
struct FrameData {
    /// The number of the current cycle. First cycle is 1, etc.
    cur_cycle: usize,

    /// Whether the window Y was reached. Checked at
    /// the start of each OAM scan mode. Stays true until
    /// VBlank, if set
    win_y_reached: bool,
}

impl FrameData {
    pub fn new() -> Self {
        FrameData {
            cur_cycle: 0,
            win_y_reached: false,
        }
    }
}

impl<V: GBGraphicsDrawer> Ppu<V> {
    pub fn new(output: V) -> Self {
        Self {
            output,
            mode: PpuMode::Inactive,
            framebuf: Frame::default(),
            line_data: LineData::new(),
            frame_data: FrameData::new(),
            pix_fetcher: PixelFetcher::new(),
        }
    }

    fn sync_active_state(&mut self, mem: &mut MemController<impl GBAllocator, impl RomReader>) {
        let should_be_active = mem.io_registers.lcd_control.lcd_ppu_enable();
        let is_active = !matches!(self.mode, PpuMode::Inactive);

        if should_be_active && !is_active {
            log::info!("Turning PPU on");

            self.mode = PpuMode::OAMScan(OAMScanData::new())
        } else if !should_be_active && is_active {
            log::info!("Turning PPU off");

            self.mode = PpuMode::Inactive;
            self.line_data = LineData::new();
            self.frame_data = FrameData::new();
            self.pix_fetcher = PixelFetcher::new();
            mem.io_registers.lcd_y = 0;
        }
    }

    fn oam_scan(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), OAMScanErr> {
        let data = match &mut self.mode {
            PpuMode::OAMScan(data) => data,
            _ => panic!("Invalid mode for OAM scan!"),
        };

        if !data.window_check_done {
            data.window_check_done = true;

            self.frame_data.win_y_reached |= mem.io_registers.lcd_y == mem.io_registers.win_y;
        }

        if data.cycles_left > 0 {
            // Wait for the operation to complete
            data.cycles_left -= 1;
            return Ok(());
        }

        if data.cur_obj_index >= NUM_OAM_OBJECTS {
            debug_assert_eq!(OAM_CYCLES + 1, self.line_data.cur_cycle);
            // Operation complete. If no more objects need to be scanned, go to next
            // phase
            log::trace!(
                "OAM scan done, entering Draw mode. Found {} objects",
                data.num_in_buf
            );

            mem.vram_open = false;
            self.mode = PpuMode::Draw(DrawData::new(
                data.buffer,
                data.num_in_buf,
                mem.io_registers.scx % 8,
            ));
            return Ok(());
        }

        if data.num_in_buf < 10 {
            log::trace!("OAM Scanning object {}", data.cur_obj_index);

            let obj_data_raw: [u8; 4] = mem.read_range(
                OAM_START + (size_of::<ObjectData>() as u16 * data.cur_obj_index as u16),
            )?;

            let obj_data: ObjectData = obj_data_raw.into();
            let obj_height = if mem.io_registers.lcd_control.obj_size() {
                16
            } else {
                8
            };

            let xpos_ok = obj_data.offset_xpos() > 0;

            let ly = mem.io_registers.lcd_y;
            let ypos_ok = (ly as i16) >= obj_data.offset_ypos() // Top above current line
                && (ly as i16) < obj_data.offset_ypos() + obj_height; // bottom below current line

            if xpos_ok && ypos_ok {
                log::trace!("Adding object {} to buffer", data.cur_obj_index);

                data.buffer[data.num_in_buf as usize] = obj_data;
                data.num_in_buf += 1;

                if data.num_in_buf == 10 {
                    log::trace!("Object buffer full, not adding any more");
                }
            }
        } else {
            log::trace!("Cannot add more object to buffer in OAM scan, continuing");
        }

        data.cur_obj_index += 1;

        data.cycles_left = 1; // 2 cycles per object, we just did the first

        Ok(())
    }

    fn get_obj_at_x(objs: &mut [ObjectData], x: u8) -> Option<ObjectData> {
        let idx = objs.iter().position(|obj| x as i16 == obj.offset_xpos());

        match idx {
            Some(idx) => {
                // Found an object! Remove it from the buffer
                let found_obj = objs[idx];

                // Shift all elements starting at the removed one left
                for i in (idx + 1)..objs.len() {
                    objs[i - 1] = objs[i];
                }

                // Callee should take care of actually shortening the buffer
                Some(found_obj)
            }
            None => None,
        }
    }

    fn draw(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), DrawErr> {
        let data = match &mut self.mode {
            PpuMode::Draw(data) => data,
            _ => panic!("Invalid mode for drawing!"),
        };

        self.pix_fetcher.run_cycle(mem, false)?;

        // Find out if there's an object at the current x we need to fetch
        if !self.pix_fetcher.is_fetching_obj() {
            let found_obj = Self::get_obj_at_x(
                &mut data.buffer[..data.num_in_buf as usize],
                data.pushed_pixels,
            );

            if let Some(obj) = found_obj {
                // [get_obj_at_x] should have already shifted the remaining objects
                // to the left
                data.num_in_buf -= 1;
                self.pix_fetcher.fetch_obj(obj);
            }
        }

        // _After_ we've initiated a fetch, we wait for fetching to be complete.
        // We need to check this here, so that multiple overlapping objects
        // correctly pause rendering
        if self.pix_fetcher.is_fetching_obj() {
            return Ok(());
        }

        if self.pix_fetcher.get_bg_fifo().len() > 8 {
            if data.pix_to_discard > 0 {
                _ = self.pix_fetcher.get_bg_fifo_mut().pop().unwrap();
                data.pix_to_discard -= 1;
            } else {
                let bg_palette = Palette::load_bg(mem);
                let bg_pix = self.pix_fetcher.get_bg_fifo_mut().pop().unwrap();
                let bg_color = bg_palette.make_color(bg_pix);
                let obj_pix = self.pix_fetcher.get_obj_fifo_mut().pop();

                let color = if let Ok(obj_pix) = obj_pix {
                    if (obj_pix.color == GbColorID::ID0)
                        || (!obj_pix.prio_always && bg_pix != GbColorID::ID0)
                    {
                        bg_color
                    } else {
                        Palette::load_obj(obj_pix.palette_id, mem).make_color(obj_pix.color)
                    }
                } else {
                    bg_color
                };

                self.framebuf
                    .set_pix(data.pushed_pixels, mem.io_registers.lcd_y, color);

                data.pushed_pixels += 1;
            }
        }

        if data.pushed_pixels as usize == FRAME_X {
            mem.vram_open = true;
            mem.oam_open = true;
            self.mode = PpuMode::HBlank;
        }

        Ok(())
    }

    fn hblank(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), HBlankErr> {
        if self.line_data.cur_cycle == SCANLINE_CYCLES {
            self.line_data = LineData::new();
            mem.io_registers.lcd_y += 1;

            self.pix_fetcher.hblank_reset();

            if mem.io_registers.lcd_y as usize == FRAME_Y {
                self.mode = PpuMode::VBlank;
            } else {
                mem.oam_open = false;
                self.mode = PpuMode::OAMScan(OAMScanData::new());
            }
        }

        Ok(())
    }

    fn vblank(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), VBlankErr<V>> {
        if self.line_data.cur_cycle == SCANLINE_CYCLES {
            self.line_data = LineData::new();
            mem.io_registers.lcd_y += 1;

            if mem.io_registers.lcd_y as usize == (FRAME_Y + 10) {
                mem.io_registers.lcd_y = 0;

                self.output
                    .output(&self.framebuf)
                    .map_err(|e| VBlankErr::<V>::OutputErr(e))?;

                self.frame_data = FrameData::new();

                mem.oam_open = false;
                self.mode = PpuMode::OAMScan(OAMScanData::new());

                self.pix_fetcher.vblank_reset();
            }
        }

        Ok(())
    }

    pub fn run_cycle(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), PpuErr<V>> {
        self.sync_active_state(mem);

        if !matches!(self.mode, PpuMode::Inactive) {
            self.line_data.cur_cycle += 1;
            self.frame_data.cur_cycle += 1;
        }

        match &mut self.mode {
            PpuMode::Inactive => {}
            PpuMode::HBlank => self.hblank(mem)?,
            PpuMode::VBlank => self.vblank(mem)?,
            PpuMode::OAMScan(_) => self.oam_scan(mem)?,
            PpuMode::Draw(_) => self.draw(mem)?,
        }

        debug_assert!((mem.io_registers.lcd_y as usize) < (FRAME_Y + 10));

        if mem.io_registers.lcd_y as usize >= FRAME_Y {
            debug_assert!(matches!(self.mode, PpuMode::VBlank));
        }

        Ok(())
    }
}
