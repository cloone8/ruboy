use core::num::Wrapping;

use thiserror::Error;

use crate::ppu::palette::Palette;

use super::interrupts::Interrupts;

#[derive(Debug, Copy, Clone, Default)]
pub struct LcdControl(u8);

impl From<u8> for LcdControl {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<LcdControl> for u8 {
    fn from(value: LcdControl) -> Self {
        value.0
    }
}

impl LcdControl {
    #[inline]
    const fn get(self, mask: u8) -> bool {
        (self.0 & mask) != 0
    }

    #[inline]
    fn set(&mut self, mask: u8, val: bool) {
        if val {
            self.0 |= mask
        } else {
            self.0 &= !mask
        }
    }

    #[inline]
    pub const fn bg_win_enable(self) -> bool {
        self.get(0b1)
    }

    #[inline]
    pub const fn obj_enable(self) -> bool {
        self.get(0b10)
    }

    #[inline]
    pub const fn obj_size(self) -> bool {
        self.get(0b100)
    }

    #[inline]
    pub const fn bg_tilemap_area(self) -> bool {
        self.get(0b1000)
    }

    #[inline]
    pub const fn bg_window_tile_area(self) -> bool {
        self.get(0b10000)
    }

    #[inline]
    pub const fn window_enable(self) -> bool {
        self.get(0b100000)
    }

    #[inline]
    pub const fn window_tilemap_area(self) -> bool {
        self.get(0b1000000)
    }

    #[inline]
    pub const fn lcd_ppu_enable(self) -> bool {
        self.get(0b10000000)
    }

    #[inline]
    pub fn set_bg_win_enable(&mut self, val: bool) {
        self.set(0b1, val)
    }

    #[inline]
    pub fn set_obj_enable(&mut self, val: bool) {
        self.set(0b10, val)
    }

    #[inline]
    pub fn set_obj_size(&mut self, val: bool) {
        self.set(0b100, val)
    }

    #[inline]
    pub fn set_bg_tilemap_area(&mut self, val: bool) {
        self.set(0b1000, val)
    }

    #[inline]
    pub fn set_bg_window_tile_area(&mut self, val: bool) {
        self.set(0b10000, val)
    }

    #[inline]
    pub fn set_window_enable(&mut self, val: bool) {
        self.set(0b100000, val)
    }

    #[inline]
    pub fn set_window_tilemap_area(&mut self, val: bool) {
        self.set(0b1000000, val)
    }

    #[inline]
    pub fn set_lcd_ppu_enable(&mut self, val: bool) {
        self.set(0b10000000, val)
    }
}

#[derive(Debug)]
pub struct IoRegs {
    /// 0xFF00
    pub joypad: u8,

    /// 0xFF04
    pub timer_div: Wrapping<u8>,

    /// 0xFF05
    pub timer_counter: u8,

    /// 0xFF06
    pub timer_modulo: u8,

    /// 0xFF07
    pub timer_control: u8,

    /// 0xFF0F
    pub interrupts_requested: Interrupts,

    /// 0xFF40
    pub lcd_control: LcdControl,

    /// 0xFF41
    pub lcd_stat: u8,

    /// 0xFF42
    pub scy: u8,

    /// 0xFF43
    pub scx: u8,

    /// 0xFF44
    pub lcd_y: u8,

    /// 0xFF45
    pub lcd_y_comp: u8,

    /// 0xFF46
    pub oam_dma: u8,

    /// 0xFF47
    pub bg_palette: Palette,

    /// 0xFF48
    pub obj0_palette: Palette,

    /// 0xFF49
    pub obj1_palette: Palette,

    /// 0xFF4A
    pub win_y: u8,

    /// 0xFF4B
    pub win_x: u8,

    /// 0xFF50
    pub boot_rom_enabled: bool,
}

#[derive(Debug, Error)]
pub enum IoWriteErr {}

#[derive(Debug, Error)]
pub enum IoReadErr {}

impl Default for IoRegs {
    fn default() -> Self {
        Self::new()
    }
}

impl IoRegs {
    pub fn new() -> Self {
        Self {
            joypad: 0,
            timer_div: Wrapping(0),
            timer_counter: 0,
            timer_modulo: 0,
            timer_control: 0,
            interrupts_requested: Interrupts::default(),
            lcd_control: LcdControl::default(),
            lcd_stat: 0,
            scy: 0,
            scx: 0,
            lcd_y: 0,
            oam_dma: 0,
            lcd_y_comp: 0,
            bg_palette: Palette::new(),
            obj0_palette: Palette::new(),
            obj1_palette: Palette::new(),
            win_y: 0,
            win_x: 0,
            boot_rom_enabled: cfg!(feature = "boot_img_enabled"),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<(), IoWriteErr> {
        match addr {
            ..=0xFEFF => panic!("Too low for I/O range"),
            0xFF00 => self.joypad = (self.joypad & 0x0F) | (val & 0xF0),
            0xFF04 => self.timer_div.0 = 0, // Writing to div register always resets it
            0xFF05 => self.timer_counter = val,
            0xFF06 => self.timer_modulo = val,
            0xFF07 => self.timer_control = val,
            0xFF40 => self.lcd_control = val.into(),
            0xFF41 => self.lcd_stat = val,
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            // 0xFF44 => self.lcd_y = val,
            0xFF45 => self.lcd_y_comp = val,
            0xFF46 => self.oam_dma = val,
            0xFF47 => self.bg_palette = val.into(),
            0xFF48 => self.obj0_palette = val.into(),
            0xFF49 => self.obj1_palette = val.into(),
            0xFF4A => self.win_y = val,
            0xFF4B => self.win_x = val,
            0xFF50 => {
                if self.boot_rom_enabled && val != 0 {
                    log::info!("Disabling boot ROM");
                }

                self.boot_rom_enabled = self.boot_rom_enabled && val == 0; // Disable boot-rom if non-zero is written
            }
            0xFF80.. => panic!("Too high for I/O range"),
            _ => {
                log::debug!("I/O register not implemented for writing: 0x{:x}", addr);
            }
        };

        Ok(())
    }

    pub fn read(&self, addr: u16) -> Result<u8, IoReadErr> {
        match addr {
            ..=0xFEFF => panic!("Too low for I/O range"),
            0xFF00 => Ok(self.joypad),
            0xFF04 => Ok(self.timer_div.0),
            0xFF05 => Ok(self.timer_counter),
            0xFF06 => Ok(self.timer_modulo),
            0xFF07 => Ok(self.timer_control),
            0xFF40 => Ok(self.lcd_control.into()),
            0xFF41 => Ok(self.lcd_stat),
            0xFF42 => Ok(self.scy),
            0xFF43 => Ok(self.scx),
            0xFF44 => Ok(self.lcd_y),
            0xFF45 => Ok(self.lcd_y_comp),
            0xFF46 => Ok(self.oam_dma),
            0xFF47 => Ok(self.bg_palette.into()),
            0xFF48 => Ok(self.obj0_palette.into()),
            0xFF49 => Ok(self.obj1_palette.into()),
            0xFF4A => Ok(self.win_y),
            0xFF4B => Ok(self.win_x),
            0xFF80.. => panic!("Too high for I/O range"),
            _ => {
                log::debug!(
                    "I/O register not implemented for reading: 0x{:x}, returning 0x00",
                    addr
                );
                Ok(0x00)
            }
        }
    }
}
