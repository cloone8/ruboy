#[derive(Debug, Clone, Copy, Default)]
pub struct Interrupts {
    raw: u8,
}

impl Interrupts {
    #[inline]
    const fn get(self, mask: u8) -> bool {
        (self.raw & mask) != 0
    }

    #[inline]
    fn set(&mut self, mask: u8, val: bool) {
        if val {
            self.raw |= mask
        } else {
            self.raw &= !mask
        }
    }

    #[inline]
    pub fn vblank(self) -> bool {
        self.get(0b1)
    }

    #[inline]
    pub fn lcd(self) -> bool {
        self.get(0b01)
    }

    #[inline]
    pub fn timer(self) -> bool {
        self.get(0b001)
    }

    #[inline]
    pub fn serial(self) -> bool {
        self.get(0b0001)
    }

    #[inline]
    pub fn joypad(self) -> bool {
        self.get(0b00001)
    }

    #[inline]
    pub fn set_vblank(&mut self, val: bool) {
        self.set(0b1, val)
    }

    #[inline]
    pub fn set_lcd(&mut self, val: bool) {
        self.set(0b01, val)
    }

    #[inline]
    pub fn set_timer(&mut self, val: bool) {
        self.set(0b001, val)
    }

    #[inline]
    pub fn set_serial(&mut self, val: bool) {
        self.set(0b0001, val)
    }

    #[inline]
    pub fn set_joypad(&mut self, val: bool) {
        self.set(0b00001, val)
    }
}

impl From<u8> for Interrupts {
    fn from(value: u8) -> Self {
        Interrupts { raw: value }
    }
}

impl From<Interrupts> for u8 {
    fn from(value: Interrupts) -> Self {
        value.raw
    }
}
