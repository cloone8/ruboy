use core::mem::size_of;

use num::{
    traits::{ConstOne, ConstZero, WrappingAdd, WrappingSub},
    PrimInt,
};

pub trait GbBits {
    fn lsb(self) -> Self;
    fn lsb_set(self) -> bool;
    fn msb(self) -> Self;
    fn msb_set(self) -> bool;
    fn set_lsb(self, val: bool) -> Self;
    fn set_msb(self, val: bool) -> Self;
}

impl<N: PrimInt + ConstOne> GbBits for N {
    #[inline]
    fn lsb(self) -> Self {
        self & N::ONE
    }

    #[inline]
    fn lsb_set(self) -> bool {
        self.lsb().is_one()
    }

    #[inline]
    fn msb(self) -> Self {
        let required_shift = (size_of::<N>() * 8) - 1;
        (self >> required_shift) & N::ONE
    }

    #[inline]
    fn msb_set(self) -> bool {
        self.msb().is_one()
    }

    #[must_use]
    #[inline]
    fn set_lsb(self, val: bool) -> Self {
        if val {
            self | N::ONE
        } else {
            self & N::ONE.not()
        }
    }

    #[must_use]
    #[inline]
    fn set_msb(self, val: bool) -> Self {
        let msb_one = N::ONE << ((size_of::<N>() * 8) - 1);

        if val {
            self | msb_one
        } else {
            self & msb_one.not()
        }
    }
}

pub trait HalfCarry {
    fn halfcarry_sub(self, right: Self) -> bool;
    fn halfcarry_add(self, right: Self) -> bool;
}

impl<N: PrimInt + WrappingAdd + WrappingSub + ConstZero + ConstOne> HalfCarry for N {
    #[inline]
    fn halfcarry_add(self, right: Self) -> bool {
        let result = self.wrapping_add(&right);
        let halfcarry_bit_idx = N::from((size_of::<N>() * 8) / 2).unwrap();
        let halfcarry_one = N::ONE << halfcarry_bit_idx.to_usize().unwrap();

        ((self ^ right ^ result) & halfcarry_one) != N::ZERO
    }

    #[inline]
    fn halfcarry_sub(self, right: Self) -> bool {
        let result = self.wrapping_sub(&right);
        let halfcarry_bit_idx = N::from((size_of::<N>() * 8) / 2).unwrap();
        let halfcarry_one = N::ONE << halfcarry_bit_idx.to_usize().unwrap();

        ((self ^ right.not() ^ result) & halfcarry_one) != N::ZERO
    }
}
