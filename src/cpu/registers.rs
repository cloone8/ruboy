#[derive(Default)]
pub(crate) struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
}

/// Basic register operations
impl Registers {
    pub fn new() -> Registers {
        Self::default()
    }

    pub const fn a(&self) -> u8 {
        self.a
    }

    pub fn set_a(&mut self, value: u8) {
        self.a = value;
    }

    pub const fn b(&self) -> u8 {
        self.b
    }

    pub fn set_b(&mut self, value: u8) {
        self.b = value;
    }

    pub const fn c(&self) -> u8 {
        self.c
    }

    pub fn set_c(&mut self, value: u8) {
        self.c = value;
    }

    pub const fn d(&self) -> u8 {
        self.d
    }

    pub fn set_d(&mut self, value: u8) {
        self.d = value;
    }

    pub const fn e(&self) -> u8 {
        self.e
    }

    pub fn set_e(&mut self, value: u8) {
        self.e = value;
    }

    pub const fn f(&self) -> u8 {
        self.f
    }

    pub fn set_f(&mut self, value: u8) {
        self.f = value;
    }

    pub const fn h(&self) -> u8 {
        self.h
    }

    pub fn set_h(&mut self, value: u8) {
        self.h = value;
    }

    pub const fn l(&self) -> u8 {
        self.l
    }

    pub fn set_l(&mut self, value: u8) {
        self.l = value;
    }
}

/// Combined "virtual" registers
impl Registers {
    pub const fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xff00) >> 8) as u8;
        self.f = (value & 0x00ff) as u8;
    }

    pub const fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xff00) >> 8) as u8;
        self.c = (value & 0x00ff) as u8;
    }
    
    pub const fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xff00) >> 8) as u8;
        self.e = (value & 0x00ff) as u8;
    }

    pub const fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xff00) >> 8) as u8;
        self.l = (value & 0x00ff) as u8;
    }
}

/// Flag register operations
impl Registers {
    // TODO: Convert to macro?
    const fn get_flag<const BIT: u8>(&self) -> bool {
        debug_assert!(BIT < 8);

        self.f & (1 << BIT) != 0
    }

    // TODO: Convert to macro?
    fn set_flag<const BIT: u8>(&mut self, value: bool) {
        debug_assert!(BIT < 8);

        if value {
            self.f |= 1 << BIT;
        } else {
            self.f &= !(1 << BIT)
        }
    }

    pub const fn zero_flag(&self) -> bool {
        self.get_flag::<7>()
    }

    pub fn set_zero_flag(&mut self, value: bool) {
        self.set_flag::<7>(value);
    }

    pub const fn subtract_flag(&self) -> bool {
        self.get_flag::<6>()
    }

    pub fn set_subtract_flag(&mut self, value: bool) {
        self.set_flag::<6>(value);
    }

    pub const fn half_carry_flag(&self) -> bool {
        self.get_flag::<5>()
    }

    pub fn set_half_carry_flag(&mut self, value: bool) {
        self.set_flag::<5>(value);
    }

    pub const fn carry_flag(&self) -> bool {
        self.get_flag::<4>()
    }

    pub fn set_carry_flag(&mut self, value: bool) {
        self.set_flag::<4>(value);
    }
}
