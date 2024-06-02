use thiserror::Error;

use crate::{
    extern_traits::{
        Frame, GBAllocator, GBGraphicsDrawer, GbColorVal, RomReader, FRAME_X, FRAME_Y,
    },
    memcontroller::MemController,
};

#[derive(Debug, Clone, Copy)]
enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    Search = 2,
    Draw = 3,
}

pub struct Ppu<V: GBGraphicsDrawer> {
    output: V,
    cur_scanline: u8,
    cur_col: u8,
    cur_mode: PpuMode,
    framebuf: Frame,
}

#[derive(Debug, Error)]
pub enum PpuErr {}

impl<V: GBGraphicsDrawer> Ppu<V> {
    pub fn new(output: V) -> Self {
        Self {
            output,
            cur_scanline: 0,
            cur_col: 0,
            cur_mode: PpuMode::Search,
            framebuf: Frame::default(),
        }
    }

    pub fn run_cycle(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), PpuErr> {
        Ok(())
    }
}
