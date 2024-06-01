use std::fmt::Debug;

pub trait GBAllocator {
    type Mem<const N: usize>: GBRam;
    fn allocate<const N: usize>() -> Self::Mem<N>;
}

pub trait GBRam: Sized + Debug {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);

    fn raw(&self) -> &[u8];
    fn raw_mut(&mut self) -> &mut [u8];
    fn size(&self) -> usize;
}

impl<const N: usize> GBRam for [u8; N] {
    fn read(&self, addr: u16) -> u8 {
        self[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self[addr as usize] = val;
    }

    fn raw(&self) -> &[u8] {
        self
    }

    fn raw_mut(&mut self) -> &mut [u8] {
        self
    }

    fn size(&self) -> usize {
        N
    }
}

impl<T: GBRam> GBRam for Box<T> {
    fn read(&self, addr: u16) -> u8 {
        self.as_ref().read(addr)
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.as_mut().write(addr, val)
    }

    fn raw(&self) -> &[u8] {
        self.as_ref().raw()
    }

    fn raw_mut(&mut self) -> &mut [u8] {
        self.as_mut().raw_mut()
    }

    fn size(&self) -> usize {
        self.as_ref().size()
    }
}

pub struct StackAllocator;

impl GBAllocator for StackAllocator {
    type Mem<const N: usize> = [u8; N];

    fn allocate<const N: usize>() -> Self::Mem<N> {
        [0; N]
    }
}

pub struct BoxAllocator;

impl GBAllocator for BoxAllocator {
    type Mem<const N: usize> = Box<[u8; N]>;

    fn allocate<const N: usize>() -> Self::Mem<N> {
        Box::new([0; N])
    }
}
