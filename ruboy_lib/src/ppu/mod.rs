use std::mem::size_of;

use thiserror::Error;

use crate::{
    extern_traits::{
        Frame, GBAllocator, GBGraphicsDrawer, GbColorVal, RomReader, FRAME_X, FRAME_Y,
    },
    memcontroller::{MemController, ReadError, OAM_START},
};

#[derive(Debug, Clone)]
enum PpuMode {
    Inactive,
    HBlank(HBlankData),
    VBlank(VBlankData),
    OAMScan(OAMScanData),
    Draw(DrawData),
}

#[derive(Debug, Clone)]
struct HBlankData {}

#[derive(Debug, Clone)]
struct VBlankData {}

#[derive(Debug, Clone)]
struct OAMScanData {
    buffer: [ObjectData; 10],
    num_in_buf: u8,
    cur_obj_index: u8,
    cycles_left: u8,
}

#[derive(Debug, Clone)]
struct DrawData {}

#[derive(Debug, Clone, Copy)]
struct Tile([u8; 16]);

#[derive(Debug, Clone, Copy)]
struct BackgroundMap([u8; 32 * 32]);

#[derive(Debug, Clone, Copy, Default)]
struct ObjectData([u8; 4]);

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

impl From<[u8; 4]> for ObjectData {
    fn from(value: [u8; 4]) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
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

const NUM_OAM_OBJECTS: u8 = 40;

#[derive(Debug, Error)]
pub enum PpuErr {
    #[error("Error during VBlank: {0}")]
    HBlank(#[from] HBlankErr),

    #[error("Error during HBlank: {0}")]
    VBlank(#[from] VBlankErr),

    #[error("Error during OAM scan: {0}")]
    OAMScan(#[from] OAMScanErr),

    #[error("Error during drawing: {0}")]
    Draw(#[from] DrawErr),
}

#[derive(Debug, Error)]
pub enum HBlankErr {}

#[derive(Debug, Error)]
pub enum VBlankErr {}

#[derive(Debug, Error)]
pub enum OAMScanErr {
    #[error("Error during memory read: {0}")]
    MemRead(#[from] ReadError),
}

#[derive(Debug, Error)]
pub enum DrawErr {}

pub struct Ppu<V: GBGraphicsDrawer> {
    output: V,
    mode: PpuMode,
    framebuf: Frame,
}

impl<V: GBGraphicsDrawer> Ppu<V> {
    pub fn new(output: V) -> Self {
        Self {
            output,
            mode: PpuMode::Inactive,
            framebuf: Frame::default(),
        }
    }

    fn sync_active_state(&mut self, mem: &mut MemController<impl GBAllocator, impl RomReader>) {
        let should_be_active = mem.io_registers.lcd_control.lcd_ppu_enable();
        let is_active = !matches!(self.mode, PpuMode::Inactive);

        if should_be_active && !is_active {
            log::debug!("Turning PPU on, starting OAM scan");

            self.mode = PpuMode::OAMScan(OAMScanData {
                buffer: [ObjectData::default(); 10],
                num_in_buf: 0,
                cur_obj_index: 0,
                cycles_left: 0,
            })
        } else if !should_be_active && is_active {
            log::debug!("Turning PPU off");

            self.mode = PpuMode::Inactive;
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

        if data.cycles_left > 0 {
            // Wait for the operation to complete
            data.cycles_left -= 1;
            return Ok(());
        }

        if data.cur_obj_index >= NUM_OAM_OBJECTS {
            // Operation complete. If no more objects need to be scanned, go to next
            // phase
            log::debug!("OAM scan done, entering Draw mode");
            self.mode = PpuMode::Draw(DrawData {});
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
            let ypos_ok = ((ly + 16) as i16) >= obj_data.offset_ypos()
                && ((ly + 16) as i16) < obj_data.offset_ypos() + obj_height;

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

    fn draw(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), DrawErr> {
        Ok(())
    }

    fn hblank(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), HBlankErr> {
        Ok(())
    }

    fn vblank(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), VBlankErr> {
        Ok(())
    }

    pub fn run_cycle(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), PpuErr> {
        self.sync_active_state(mem);

        match &mut self.mode {
            PpuMode::Inactive => {}
            PpuMode::HBlank(_) => self.hblank(mem)?,
            PpuMode::VBlank(_) => self.vblank(mem)?,
            PpuMode::OAMScan(_) => self.oam_scan(mem)?,
            PpuMode::Draw(_) => self.draw(mem)?,
        }

        Ok(())
    }
}
