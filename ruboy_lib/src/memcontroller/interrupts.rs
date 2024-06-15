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
        self.get(0b10)
    }

    #[inline]
    pub fn timer(self) -> bool {
        self.get(0b100)
    }

    #[inline]
    pub fn serial(self) -> bool {
        self.get(0b1000)
    }

    #[inline]
    pub fn joypad(self) -> bool {
        self.get(0b10000)
    }

    #[inline]
    pub fn set_vblank(&mut self, val: bool) {
        self.set(0b1, val)
    }

    #[inline]
    pub fn set_lcd(&mut self, val: bool) {
        self.set(0b10, val)
    }

    #[inline]
    pub fn set_timer(&mut self, val: bool) {
        self.set(0b100, val)
    }

    #[inline]
    pub fn set_serial(&mut self, val: bool) {
        self.set(0b1000, val)
    }

    #[inline]
    pub fn set_joypad(&mut self, val: bool) {
        self.set(0b10000, val)
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
