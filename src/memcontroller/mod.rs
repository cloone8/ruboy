pub(crate) struct MemController {
    mem: [u8; 0xFFFF]
}

impl MemController {
    pub fn new() -> MemController {
        MemController {
            mem: [0; 0xFFFF]
        }
    }

    pub const fn read8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub const fn read16(&self, addr: u16) -> u16 {
        let b1 = self.mem[addr as usize];
        let b2 = self.mem[(addr + 1) as usize];

        u16::from_le_bytes([b1, b2])
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }

    pub fn write16(&mut self, addr: u16, value: u16) {
        let bytes = value.to_le_bytes();

        self.mem[addr as usize] = bytes[0];
        self.mem[(addr + 1) as usize] = bytes[1];
    }
}
