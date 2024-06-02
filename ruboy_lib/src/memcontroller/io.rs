use thiserror::Error;

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
        self.get(0b01)
    }

    #[inline]
    pub const fn obj_size(self) -> bool {
        self.get(0b001)
    }

    #[inline]
    pub const fn bg_tilemap_area(self) -> bool {
        self.get(0b0001)
    }

    #[inline]
    pub const fn bg_window_tile_area(self) -> bool {
        self.get(0b00001)
    }

    #[inline]
    pub const fn window_enable(self) -> bool {
        self.get(0b000001)
    }

    #[inline]
    pub const fn window_tilemap_area(self) -> bool {
        self.get(0b0000001)
    }

    #[inline]
    pub const fn lcd_ppu_enable(self) -> bool {
        self.get(0b00000001)
    }

    #[inline]
    pub fn set_bg_win_enable(&mut self, val: bool) {
        self.set(0b1, val)
    }

    #[inline]
    pub fn set_obj_enable(&mut self, val: bool) {
        self.set(0b01, val)
    }

    #[inline]
    pub fn set_obj_size(&mut self, val: bool) {
        self.set(0b001, val)
    }

    #[inline]
    pub fn set_bg_tilemap_area(&mut self, val: bool) {
        self.set(0b0001, val)
    }

    #[inline]
    pub fn set_bg_window_tile_area(&mut self, val: bool) {
        self.set(0b00001, val)
    }

    #[inline]
    pub fn set_window_enable(&mut self, val: bool) {
        self.set(0b000001, val)
    }

    #[inline]
    pub fn set_window_tilemap_area(&mut self, val: bool) {
        self.set(0b0000001, val)
    }

    #[inline]
    pub fn set_lcd_ppu_enable(&mut self, val: bool) {
        self.set(0b00000001, val)
    }
}

#[derive(Debug)]
pub struct IoRegs {
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

impl IoRegs {
    pub fn new() -> Self {
        Self {
            interrupts_requested: Interrupts::default(),
            lcd_control: LcdControl::default(),
            lcd_stat: 0,
            scy: 0,
            scx: 0,
            lcd_y: 0,
            lcd_y_comp: 0,
            win_y: 0,
            win_x: 0,
            boot_rom_enabled: cfg!(feature = "boot_img_enabled"),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<(), IoWriteErr> {
        match addr {
            ..=0xFEFF => panic!("Too low for I/O range"),
            0xFF40 => self.lcd_control = val.into(),
            0xFF41 => self.lcd_stat = val,
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.lcd_y = val,
            0xFF45 => self.lcd_y_comp = val,
            0xFF4A => self.win_y = val,
            0xFF4B => self.win_x = val,
            0xFF50 => {
                if self.boot_rom_enabled && val != 0 {
                    log::debug!("Disabling boot ROM");
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
            0xFF40 => Ok(self.lcd_control.into()),
            0xFF41 => Ok(self.lcd_stat),
            0xFF42 => Ok(self.scy),
            0xFF43 => Ok(self.scx),
            0xFF44 => Ok(self.lcd_y),
            0xFF45 => Ok(self.lcd_y_comp),
            0xFF4A => Ok(self.win_y),
            0xFF4B => Ok(self.win_x),
            0xFF80.. => panic!("Too high for I/O range"),
            _ => {
                log::debug!(
                    "I/O register not implemented for reading: 0x{:x}, returning 0xFF",
                    addr
                );
                Ok(0xFF)
            }
        }
    }
}
