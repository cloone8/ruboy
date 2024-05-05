use crate::isa::decoder::DecoderReadable;

pub struct InlineGBRam {
    mem: [u8; 0xFFFF],
}

impl Default for InlineGBRam {
    fn default() -> Self {
        InlineGBRam {
            mem: [0; 0xFFFF]
        }
    }
}

impl InlineGBRam {
    pub fn new() -> InlineGBRam {
        InlineGBRam::default()
    }
}

pub struct BoxedGBRam {
    mem: Box<[u8; 0xFFFF]>,
}

impl Default for BoxedGBRam {
    fn default() -> Self {
        BoxedGBRam {
            mem: Box::new([0; 0xFFFF]),
        }
    }
}

impl BoxedGBRam {
    pub fn new() -> BoxedGBRam {
        BoxedGBRam::default()
    }
}

pub struct MemController<T: GBRam> {
    ram: T,
}

impl<T: GBRam> MemController<T> {
    pub fn new() -> Self {
        MemController {
            ram: T::default()
        }
    }
    
    pub fn read8(&self, addr: u16) -> u8 {
        self.ram.read8(addr)
    }

    pub fn read16(&self, addr: u16) -> u16 {
        self.ram.read16(addr)
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        self.ram.write8(addr, value)
    }

    pub fn write16(&mut self, addr: u16, value: u16) {
        self.ram.write16(addr, value)
    }
}

impl<T: GBRam> DecoderReadable for MemController<T> {
    fn read_at(&self, idx: usize) -> Option<u8> {
        match u16::try_from(idx) {
            Ok(addr) => Some(self.read8(addr)),
            Err(_) => None,
        }
    }
}

pub trait GBRam: Default {
    fn read8(&self, addr: u16) -> u8;
    fn read16(&self, addr: u16) -> u16;
    fn write8(&mut self, addr: u16, value: u8);
    fn write16(&mut self, addr: u16, value: u16);
}

macro_rules! impl_basic_ram {
    ($name:ident) => {
        impl GBRam for $name {
            fn read8(&self, addr: u16) -> u8 {
                self.mem[addr as usize]
            }

            fn read16(&self, addr: u16) -> u16 {
                let b1 = self.mem[addr as usize];
                let b2 = self.mem[(addr + 1) as usize];

                u16::from_le_bytes([b1, b2])
            }

            fn write8(&mut self, addr: u16, value: u8) {
                self.mem[addr as usize] = value;
            }

            fn write16(&mut self, addr: u16, value: u16) {
                let bytes = value.to_le_bytes();

                self.mem[addr as usize] = bytes[0];
                self.mem[(addr + 1) as usize] = bytes[1];
            }
        }
    };
}

impl_basic_ram!(InlineGBRam);
impl_basic_ram!(BoxedGBRam);
