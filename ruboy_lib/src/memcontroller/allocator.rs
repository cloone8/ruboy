pub trait GBAllocator {
    type Mem<const N: usize>: Sized;
    fn allocate<const N: usize>() -> Self::Mem<N>;
    fn read<const N: usize>(ram: &Self::Mem<N>, addr: u16) -> u8;
    fn write<const N: usize>(ram: &mut Self::Mem<N>, addr: u16, val: u8);
}

pub struct StackAllocator;

impl GBAllocator for StackAllocator {
    type Mem<const N: usize> = [u8; N];

    fn allocate<const N: usize>() -> Self::Mem<N> {
        [0; N]
    }

    fn read<const N: usize>(ram: &Self::Mem<N>, addr: u16) -> u8 {
        ram[addr as usize]
    }

    fn write<const N: usize>(ram: &mut Self::Mem<N>, addr: u16, val: u8) {
        ram[addr as usize] = val;
    }
}

pub struct BoxAllocator;

impl GBAllocator for BoxAllocator {
    type Mem<const N: usize> = Box<[u8; N]>;

    fn allocate<const N: usize>() -> Self::Mem<N> {
        Box::new([0; N])
    }

    fn read<const N: usize>(ram: &Self::Mem<N>, addr: u16) -> u8 {
        ram[addr as usize]
    }

    fn write<const N: usize>(ram: &mut Self::Mem<N>, addr: u16, val: u8) {
        ram[addr as usize] = val
    }
}
