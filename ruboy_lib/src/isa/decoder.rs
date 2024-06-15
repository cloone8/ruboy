use thiserror::Error;

use crate::isa::{
    ArithSrc, Condition, IncDecTarget, Instruction, Ld16Dst, Ld16Src, Ld8Dst, Ld8Src, MemLoc,
    Reg16, Reg8,
};

use super::{Bit, PrefArithTarget, RsVec};

#[derive(Error, Debug, Clone, Copy)]
pub enum DecodeError {
    /// Not enough bytes in the input
    /// slice to properly decode the error
    #[error("Could not read enough bytes to decode the instruction")]
    NotEnoughBytes,
}

macro_rules! illegal {
    ($opcode:expr) => {
        Instruction::IllegalInstruction($opcode)
    };
}

macro_rules! ld_regs {
    ($dst:ident, $src:ident) => {
        Instruction::Load8(Ld8Dst::Reg(Reg8::$dst), Ld8Src::Reg(Reg8::$src))
    };
}

macro_rules! ld_reg_hl {
    ($dst:ident) => {
        Instruction::Load8(Ld8Dst::Reg(Reg8::$dst), Ld8Src::Mem(MemLoc::Reg(Reg16::HL)))
    };
}

macro_rules! ld_hl_reg {
    ($src:ident) => {
        Instruction::Load8(Ld8Dst::Mem(MemLoc::Reg(Reg16::HL)), Ld8Src::Reg(Reg8::$src))
    };
}

macro_rules! add_reg {
    ($src:ident) => {
        Instruction::Add(ArithSrc::Reg(Reg8::$src))
    };
}

macro_rules! add_carry_reg {
    ($src:ident) => {
        Instruction::AddCarry(ArithSrc::Reg(Reg8::$src))
    };
}

macro_rules! sub_reg {
    ($src:ident) => {
        Instruction::Sub(ArithSrc::Reg(Reg8::$src))
    };
}

macro_rules! sub_carry_reg {
    ($src:ident) => {
        Instruction::SubCarry(ArithSrc::Reg(Reg8::$src))
    };
}

macro_rules! and_reg {
    ($src:ident) => {
        Instruction::And(ArithSrc::Reg(Reg8::$src))
    };
}

macro_rules! or_reg {
    ($src:ident) => {
        Instruction::Or(ArithSrc::Reg(Reg8::$src))
    };
}

macro_rules! xor_reg {
    ($src:ident) => {
        Instruction::Xor(ArithSrc::Reg(Reg8::$src))
    };
}

macro_rules! cmp_reg {
    ($src:ident) => {
        Instruction::Cmp(ArithSrc::Reg(Reg8::$src))
    };
}
const fn decode_prefixed(instr: u8) -> Instruction {
    match instr {
        //TODO: Jesus Christ, proc macro time.
        0x00 => Instruction::RotLeftCircular(PrefArithTarget::Reg(Reg8::B)),
        0x01 => Instruction::RotLeftCircular(PrefArithTarget::Reg(Reg8::C)),
        0x02 => Instruction::RotLeftCircular(PrefArithTarget::Reg(Reg8::D)),
        0x03 => Instruction::RotLeftCircular(PrefArithTarget::Reg(Reg8::E)),
        0x04 => Instruction::RotLeftCircular(PrefArithTarget::Reg(Reg8::H)),
        0x05 => Instruction::RotLeftCircular(PrefArithTarget::Reg(Reg8::L)),
        0x06 => Instruction::RotLeftCircular(PrefArithTarget::MemHL),
        0x07 => Instruction::RotLeftCircular(PrefArithTarget::Reg(Reg8::A)),
        0x08 => Instruction::RotRightCircular(PrefArithTarget::Reg(Reg8::B)),
        0x09 => Instruction::RotRightCircular(PrefArithTarget::Reg(Reg8::C)),
        0x0A => Instruction::RotRightCircular(PrefArithTarget::Reg(Reg8::D)),
        0x0B => Instruction::RotRightCircular(PrefArithTarget::Reg(Reg8::E)),
        0x0C => Instruction::RotRightCircular(PrefArithTarget::Reg(Reg8::H)),
        0x0D => Instruction::RotRightCircular(PrefArithTarget::Reg(Reg8::L)),
        0x0E => Instruction::RotRightCircular(PrefArithTarget::MemHL),
        0x0F => Instruction::RotRightCircular(PrefArithTarget::Reg(Reg8::A)),
        0x10 => Instruction::RotLeft(PrefArithTarget::Reg(Reg8::B)),
        0x11 => Instruction::RotLeft(PrefArithTarget::Reg(Reg8::C)),
        0x12 => Instruction::RotLeft(PrefArithTarget::Reg(Reg8::D)),
        0x13 => Instruction::RotLeft(PrefArithTarget::Reg(Reg8::E)),
        0x14 => Instruction::RotLeft(PrefArithTarget::Reg(Reg8::H)),
        0x15 => Instruction::RotLeft(PrefArithTarget::Reg(Reg8::L)),
        0x16 => Instruction::RotLeft(PrefArithTarget::MemHL),
        0x17 => Instruction::RotLeft(PrefArithTarget::Reg(Reg8::A)),
        0x18 => Instruction::RotRight(PrefArithTarget::Reg(Reg8::B)),
        0x19 => Instruction::RotRight(PrefArithTarget::Reg(Reg8::C)),
        0x1A => Instruction::RotRight(PrefArithTarget::Reg(Reg8::D)),
        0x1B => Instruction::RotRight(PrefArithTarget::Reg(Reg8::E)),
        0x1C => Instruction::RotRight(PrefArithTarget::Reg(Reg8::H)),
        0x1D => Instruction::RotRight(PrefArithTarget::Reg(Reg8::L)),
        0x1E => Instruction::RotRight(PrefArithTarget::MemHL),
        0x1F => Instruction::RotRight(PrefArithTarget::Reg(Reg8::A)),
        0x20 => Instruction::ShiftLeftArith(PrefArithTarget::Reg(Reg8::B)),
        0x21 => Instruction::ShiftLeftArith(PrefArithTarget::Reg(Reg8::C)),
        0x22 => Instruction::ShiftLeftArith(PrefArithTarget::Reg(Reg8::D)),
        0x23 => Instruction::ShiftLeftArith(PrefArithTarget::Reg(Reg8::E)),
        0x24 => Instruction::ShiftLeftArith(PrefArithTarget::Reg(Reg8::H)),
        0x25 => Instruction::ShiftLeftArith(PrefArithTarget::Reg(Reg8::L)),
        0x26 => Instruction::ShiftLeftArith(PrefArithTarget::MemHL),
        0x27 => Instruction::ShiftLeftArith(PrefArithTarget::Reg(Reg8::A)),
        0x28 => Instruction::ShiftRightArith(PrefArithTarget::Reg(Reg8::B)),
        0x29 => Instruction::ShiftRightArith(PrefArithTarget::Reg(Reg8::C)),
        0x2A => Instruction::ShiftRightArith(PrefArithTarget::Reg(Reg8::D)),
        0x2B => Instruction::ShiftRightArith(PrefArithTarget::Reg(Reg8::E)),
        0x2C => Instruction::ShiftRightArith(PrefArithTarget::Reg(Reg8::H)),
        0x2D => Instruction::ShiftRightArith(PrefArithTarget::Reg(Reg8::L)),
        0x2E => Instruction::ShiftRightArith(PrefArithTarget::MemHL),
        0x2F => Instruction::ShiftRightArith(PrefArithTarget::Reg(Reg8::A)),
        0x30 => Instruction::Swap(PrefArithTarget::Reg(Reg8::B)),
        0x31 => Instruction::Swap(PrefArithTarget::Reg(Reg8::C)),
        0x32 => Instruction::Swap(PrefArithTarget::Reg(Reg8::D)),
        0x33 => Instruction::Swap(PrefArithTarget::Reg(Reg8::E)),
        0x34 => Instruction::Swap(PrefArithTarget::Reg(Reg8::H)),
        0x35 => Instruction::Swap(PrefArithTarget::Reg(Reg8::L)),
        0x36 => Instruction::Swap(PrefArithTarget::MemHL),
        0x37 => Instruction::Swap(PrefArithTarget::Reg(Reg8::A)),
        0x38 => Instruction::ShiftRightLogic(PrefArithTarget::Reg(Reg8::B)),
        0x39 => Instruction::ShiftRightLogic(PrefArithTarget::Reg(Reg8::C)),
        0x3A => Instruction::ShiftRightLogic(PrefArithTarget::Reg(Reg8::D)),
        0x3B => Instruction::ShiftRightLogic(PrefArithTarget::Reg(Reg8::E)),
        0x3C => Instruction::ShiftRightLogic(PrefArithTarget::Reg(Reg8::H)),
        0x3D => Instruction::ShiftRightLogic(PrefArithTarget::Reg(Reg8::L)),
        0x3E => Instruction::ShiftRightLogic(PrefArithTarget::MemHL),
        0x3F => Instruction::ShiftRightLogic(PrefArithTarget::Reg(Reg8::A)),
        0x40 => Instruction::Bit(Bit::B0, PrefArithTarget::Reg(Reg8::B)),
        0x41 => Instruction::Bit(Bit::B0, PrefArithTarget::Reg(Reg8::C)),
        0x42 => Instruction::Bit(Bit::B0, PrefArithTarget::Reg(Reg8::D)),
        0x43 => Instruction::Bit(Bit::B0, PrefArithTarget::Reg(Reg8::E)),
        0x44 => Instruction::Bit(Bit::B0, PrefArithTarget::Reg(Reg8::H)),
        0x45 => Instruction::Bit(Bit::B0, PrefArithTarget::Reg(Reg8::L)),
        0x46 => Instruction::Bit(Bit::B0, PrefArithTarget::MemHL),
        0x47 => Instruction::Bit(Bit::B0, PrefArithTarget::Reg(Reg8::A)),
        0x48 => Instruction::Bit(Bit::B1, PrefArithTarget::Reg(Reg8::B)),
        0x49 => Instruction::Bit(Bit::B1, PrefArithTarget::Reg(Reg8::C)),
        0x4A => Instruction::Bit(Bit::B1, PrefArithTarget::Reg(Reg8::D)),
        0x4B => Instruction::Bit(Bit::B1, PrefArithTarget::Reg(Reg8::E)),
        0x4C => Instruction::Bit(Bit::B1, PrefArithTarget::Reg(Reg8::H)),
        0x4D => Instruction::Bit(Bit::B1, PrefArithTarget::Reg(Reg8::L)),
        0x4E => Instruction::Bit(Bit::B1, PrefArithTarget::MemHL),
        0x4F => Instruction::Bit(Bit::B1, PrefArithTarget::Reg(Reg8::A)),
        0x50 => Instruction::Bit(Bit::B2, PrefArithTarget::Reg(Reg8::B)),
        0x51 => Instruction::Bit(Bit::B2, PrefArithTarget::Reg(Reg8::C)),
        0x52 => Instruction::Bit(Bit::B2, PrefArithTarget::Reg(Reg8::D)),
        0x53 => Instruction::Bit(Bit::B2, PrefArithTarget::Reg(Reg8::E)),
        0x54 => Instruction::Bit(Bit::B2, PrefArithTarget::Reg(Reg8::H)),
        0x55 => Instruction::Bit(Bit::B2, PrefArithTarget::Reg(Reg8::L)),
        0x56 => Instruction::Bit(Bit::B2, PrefArithTarget::MemHL),
        0x57 => Instruction::Bit(Bit::B2, PrefArithTarget::Reg(Reg8::A)),
        0x58 => Instruction::Bit(Bit::B3, PrefArithTarget::Reg(Reg8::B)),
        0x59 => Instruction::Bit(Bit::B3, PrefArithTarget::Reg(Reg8::C)),
        0x5A => Instruction::Bit(Bit::B3, PrefArithTarget::Reg(Reg8::D)),
        0x5B => Instruction::Bit(Bit::B3, PrefArithTarget::Reg(Reg8::E)),
        0x5C => Instruction::Bit(Bit::B3, PrefArithTarget::Reg(Reg8::H)),
        0x5D => Instruction::Bit(Bit::B3, PrefArithTarget::Reg(Reg8::L)),
        0x5E => Instruction::Bit(Bit::B3, PrefArithTarget::MemHL),
        0x5F => Instruction::Bit(Bit::B3, PrefArithTarget::Reg(Reg8::A)),
        0x60 => Instruction::Bit(Bit::B4, PrefArithTarget::Reg(Reg8::B)),
        0x61 => Instruction::Bit(Bit::B4, PrefArithTarget::Reg(Reg8::C)),
        0x62 => Instruction::Bit(Bit::B4, PrefArithTarget::Reg(Reg8::D)),
        0x63 => Instruction::Bit(Bit::B4, PrefArithTarget::Reg(Reg8::E)),
        0x64 => Instruction::Bit(Bit::B4, PrefArithTarget::Reg(Reg8::H)),
        0x65 => Instruction::Bit(Bit::B4, PrefArithTarget::Reg(Reg8::L)),
        0x66 => Instruction::Bit(Bit::B4, PrefArithTarget::MemHL),
        0x67 => Instruction::Bit(Bit::B4, PrefArithTarget::Reg(Reg8::A)),
        0x68 => Instruction::Bit(Bit::B5, PrefArithTarget::Reg(Reg8::B)),
        0x69 => Instruction::Bit(Bit::B5, PrefArithTarget::Reg(Reg8::C)),
        0x6A => Instruction::Bit(Bit::B5, PrefArithTarget::Reg(Reg8::D)),
        0x6B => Instruction::Bit(Bit::B5, PrefArithTarget::Reg(Reg8::E)),
        0x6C => Instruction::Bit(Bit::B5, PrefArithTarget::Reg(Reg8::H)),
        0x6D => Instruction::Bit(Bit::B5, PrefArithTarget::Reg(Reg8::L)),
        0x6E => Instruction::Bit(Bit::B5, PrefArithTarget::MemHL),
        0x6F => Instruction::Bit(Bit::B5, PrefArithTarget::Reg(Reg8::A)),
        0x70 => Instruction::Bit(Bit::B6, PrefArithTarget::Reg(Reg8::B)),
        0x71 => Instruction::Bit(Bit::B6, PrefArithTarget::Reg(Reg8::C)),
        0x72 => Instruction::Bit(Bit::B6, PrefArithTarget::Reg(Reg8::D)),
        0x73 => Instruction::Bit(Bit::B6, PrefArithTarget::Reg(Reg8::E)),
        0x74 => Instruction::Bit(Bit::B6, PrefArithTarget::Reg(Reg8::H)),
        0x75 => Instruction::Bit(Bit::B6, PrefArithTarget::Reg(Reg8::L)),
        0x76 => Instruction::Bit(Bit::B6, PrefArithTarget::MemHL),
        0x77 => Instruction::Bit(Bit::B6, PrefArithTarget::Reg(Reg8::A)),
        0x78 => Instruction::Bit(Bit::B7, PrefArithTarget::Reg(Reg8::B)),
        0x79 => Instruction::Bit(Bit::B7, PrefArithTarget::Reg(Reg8::C)),
        0x7A => Instruction::Bit(Bit::B7, PrefArithTarget::Reg(Reg8::D)),
        0x7B => Instruction::Bit(Bit::B7, PrefArithTarget::Reg(Reg8::E)),
        0x7C => Instruction::Bit(Bit::B7, PrefArithTarget::Reg(Reg8::H)),
        0x7D => Instruction::Bit(Bit::B7, PrefArithTarget::Reg(Reg8::L)),
        0x7E => Instruction::Bit(Bit::B7, PrefArithTarget::MemHL),
        0x7F => Instruction::Bit(Bit::B7, PrefArithTarget::Reg(Reg8::A)),
        0x80 => Instruction::Res(Bit::B0, PrefArithTarget::Reg(Reg8::B)),
        0x81 => Instruction::Res(Bit::B0, PrefArithTarget::Reg(Reg8::C)),
        0x82 => Instruction::Res(Bit::B0, PrefArithTarget::Reg(Reg8::D)),
        0x83 => Instruction::Res(Bit::B0, PrefArithTarget::Reg(Reg8::E)),
        0x84 => Instruction::Res(Bit::B0, PrefArithTarget::Reg(Reg8::H)),
        0x85 => Instruction::Res(Bit::B0, PrefArithTarget::Reg(Reg8::L)),
        0x86 => Instruction::Res(Bit::B0, PrefArithTarget::MemHL),
        0x87 => Instruction::Res(Bit::B0, PrefArithTarget::Reg(Reg8::A)),
        0x88 => Instruction::Res(Bit::B1, PrefArithTarget::Reg(Reg8::B)),
        0x89 => Instruction::Res(Bit::B1, PrefArithTarget::Reg(Reg8::C)),
        0x8A => Instruction::Res(Bit::B1, PrefArithTarget::Reg(Reg8::D)),
        0x8B => Instruction::Res(Bit::B1, PrefArithTarget::Reg(Reg8::E)),
        0x8C => Instruction::Res(Bit::B1, PrefArithTarget::Reg(Reg8::H)),
        0x8D => Instruction::Res(Bit::B1, PrefArithTarget::Reg(Reg8::L)),
        0x8E => Instruction::Res(Bit::B1, PrefArithTarget::MemHL),
        0x8F => Instruction::Res(Bit::B1, PrefArithTarget::Reg(Reg8::A)),
        0x90 => Instruction::Res(Bit::B2, PrefArithTarget::Reg(Reg8::B)),
        0x91 => Instruction::Res(Bit::B2, PrefArithTarget::Reg(Reg8::C)),
        0x92 => Instruction::Res(Bit::B2, PrefArithTarget::Reg(Reg8::D)),
        0x93 => Instruction::Res(Bit::B2, PrefArithTarget::Reg(Reg8::E)),
        0x94 => Instruction::Res(Bit::B2, PrefArithTarget::Reg(Reg8::H)),
        0x95 => Instruction::Res(Bit::B2, PrefArithTarget::Reg(Reg8::L)),
        0x96 => Instruction::Res(Bit::B2, PrefArithTarget::MemHL),
        0x97 => Instruction::Res(Bit::B2, PrefArithTarget::Reg(Reg8::A)),
        0x98 => Instruction::Res(Bit::B3, PrefArithTarget::Reg(Reg8::B)),
        0x99 => Instruction::Res(Bit::B3, PrefArithTarget::Reg(Reg8::C)),
        0x9A => Instruction::Res(Bit::B3, PrefArithTarget::Reg(Reg8::D)),
        0x9B => Instruction::Res(Bit::B3, PrefArithTarget::Reg(Reg8::E)),
        0x9C => Instruction::Res(Bit::B3, PrefArithTarget::Reg(Reg8::H)),
        0x9D => Instruction::Res(Bit::B3, PrefArithTarget::Reg(Reg8::L)),
        0x9E => Instruction::Res(Bit::B3, PrefArithTarget::MemHL),
        0x9F => Instruction::Res(Bit::B3, PrefArithTarget::Reg(Reg8::A)),
        0xA0 => Instruction::Res(Bit::B4, PrefArithTarget::Reg(Reg8::B)),
        0xA1 => Instruction::Res(Bit::B4, PrefArithTarget::Reg(Reg8::C)),
        0xA2 => Instruction::Res(Bit::B4, PrefArithTarget::Reg(Reg8::D)),
        0xA3 => Instruction::Res(Bit::B4, PrefArithTarget::Reg(Reg8::E)),
        0xA4 => Instruction::Res(Bit::B4, PrefArithTarget::Reg(Reg8::H)),
        0xA5 => Instruction::Res(Bit::B4, PrefArithTarget::Reg(Reg8::L)),
        0xA6 => Instruction::Res(Bit::B4, PrefArithTarget::MemHL),
        0xA7 => Instruction::Res(Bit::B4, PrefArithTarget::Reg(Reg8::A)),
        0xA8 => Instruction::Res(Bit::B5, PrefArithTarget::Reg(Reg8::B)),
        0xA9 => Instruction::Res(Bit::B5, PrefArithTarget::Reg(Reg8::C)),
        0xAA => Instruction::Res(Bit::B5, PrefArithTarget::Reg(Reg8::D)),
        0xAB => Instruction::Res(Bit::B5, PrefArithTarget::Reg(Reg8::E)),
        0xAC => Instruction::Res(Bit::B5, PrefArithTarget::Reg(Reg8::H)),
        0xAD => Instruction::Res(Bit::B5, PrefArithTarget::Reg(Reg8::L)),
        0xAE => Instruction::Res(Bit::B5, PrefArithTarget::MemHL),
        0xAF => Instruction::Res(Bit::B5, PrefArithTarget::Reg(Reg8::A)),
        0xB0 => Instruction::Res(Bit::B6, PrefArithTarget::Reg(Reg8::B)),
        0xB1 => Instruction::Res(Bit::B6, PrefArithTarget::Reg(Reg8::C)),
        0xB2 => Instruction::Res(Bit::B6, PrefArithTarget::Reg(Reg8::D)),
        0xB3 => Instruction::Res(Bit::B6, PrefArithTarget::Reg(Reg8::E)),
        0xB4 => Instruction::Res(Bit::B6, PrefArithTarget::Reg(Reg8::H)),
        0xB5 => Instruction::Res(Bit::B6, PrefArithTarget::Reg(Reg8::L)),
        0xB6 => Instruction::Res(Bit::B6, PrefArithTarget::MemHL),
        0xB7 => Instruction::Res(Bit::B6, PrefArithTarget::Reg(Reg8::A)),
        0xB8 => Instruction::Res(Bit::B7, PrefArithTarget::Reg(Reg8::B)),
        0xB9 => Instruction::Res(Bit::B7, PrefArithTarget::Reg(Reg8::C)),
        0xBA => Instruction::Res(Bit::B7, PrefArithTarget::Reg(Reg8::D)),
        0xBB => Instruction::Res(Bit::B7, PrefArithTarget::Reg(Reg8::E)),
        0xBC => Instruction::Res(Bit::B7, PrefArithTarget::Reg(Reg8::H)),
        0xBD => Instruction::Res(Bit::B7, PrefArithTarget::Reg(Reg8::L)),
        0xBE => Instruction::Res(Bit::B7, PrefArithTarget::MemHL),
        0xBF => Instruction::Res(Bit::B7, PrefArithTarget::Reg(Reg8::A)),
        0xC0 => Instruction::Set(Bit::B0, PrefArithTarget::Reg(Reg8::B)),
        0xC1 => Instruction::Set(Bit::B0, PrefArithTarget::Reg(Reg8::C)),
        0xC2 => Instruction::Set(Bit::B0, PrefArithTarget::Reg(Reg8::D)),
        0xC3 => Instruction::Set(Bit::B0, PrefArithTarget::Reg(Reg8::E)),
        0xC4 => Instruction::Set(Bit::B0, PrefArithTarget::Reg(Reg8::H)),
        0xC5 => Instruction::Set(Bit::B0, PrefArithTarget::Reg(Reg8::L)),
        0xC6 => Instruction::Set(Bit::B0, PrefArithTarget::MemHL),
        0xC7 => Instruction::Set(Bit::B0, PrefArithTarget::Reg(Reg8::A)),
        0xC8 => Instruction::Set(Bit::B1, PrefArithTarget::Reg(Reg8::B)),
        0xC9 => Instruction::Set(Bit::B1, PrefArithTarget::Reg(Reg8::C)),
        0xCA => Instruction::Set(Bit::B1, PrefArithTarget::Reg(Reg8::D)),
        0xCB => Instruction::Set(Bit::B1, PrefArithTarget::Reg(Reg8::E)),
        0xCC => Instruction::Set(Bit::B1, PrefArithTarget::Reg(Reg8::H)),
        0xCD => Instruction::Set(Bit::B1, PrefArithTarget::Reg(Reg8::L)),
        0xCE => Instruction::Set(Bit::B1, PrefArithTarget::MemHL),
        0xCF => Instruction::Set(Bit::B1, PrefArithTarget::Reg(Reg8::A)),
        0xD0 => Instruction::Set(Bit::B2, PrefArithTarget::Reg(Reg8::B)),
        0xD1 => Instruction::Set(Bit::B2, PrefArithTarget::Reg(Reg8::C)),
        0xD2 => Instruction::Set(Bit::B2, PrefArithTarget::Reg(Reg8::D)),
        0xD3 => Instruction::Set(Bit::B2, PrefArithTarget::Reg(Reg8::E)),
        0xD4 => Instruction::Set(Bit::B2, PrefArithTarget::Reg(Reg8::H)),
        0xD5 => Instruction::Set(Bit::B2, PrefArithTarget::Reg(Reg8::L)),
        0xD6 => Instruction::Set(Bit::B2, PrefArithTarget::MemHL),
        0xD7 => Instruction::Set(Bit::B2, PrefArithTarget::Reg(Reg8::A)),
        0xD8 => Instruction::Set(Bit::B3, PrefArithTarget::Reg(Reg8::B)),
        0xD9 => Instruction::Set(Bit::B3, PrefArithTarget::Reg(Reg8::C)),
        0xDA => Instruction::Set(Bit::B3, PrefArithTarget::Reg(Reg8::D)),
        0xDB => Instruction::Set(Bit::B3, PrefArithTarget::Reg(Reg8::E)),
        0xDC => Instruction::Set(Bit::B3, PrefArithTarget::Reg(Reg8::H)),
        0xDD => Instruction::Set(Bit::B3, PrefArithTarget::Reg(Reg8::L)),
        0xDE => Instruction::Set(Bit::B3, PrefArithTarget::MemHL),
        0xDF => Instruction::Set(Bit::B3, PrefArithTarget::Reg(Reg8::A)),
        0xE0 => Instruction::Set(Bit::B4, PrefArithTarget::Reg(Reg8::B)),
        0xE1 => Instruction::Set(Bit::B4, PrefArithTarget::Reg(Reg8::C)),
        0xE2 => Instruction::Set(Bit::B4, PrefArithTarget::Reg(Reg8::D)),
        0xE3 => Instruction::Set(Bit::B4, PrefArithTarget::Reg(Reg8::E)),
        0xE4 => Instruction::Set(Bit::B4, PrefArithTarget::Reg(Reg8::H)),
        0xE5 => Instruction::Set(Bit::B4, PrefArithTarget::Reg(Reg8::L)),
        0xE6 => Instruction::Set(Bit::B4, PrefArithTarget::MemHL),
        0xE7 => Instruction::Set(Bit::B4, PrefArithTarget::Reg(Reg8::A)),
        0xE8 => Instruction::Set(Bit::B5, PrefArithTarget::Reg(Reg8::B)),
        0xE9 => Instruction::Set(Bit::B5, PrefArithTarget::Reg(Reg8::C)),
        0xEA => Instruction::Set(Bit::B5, PrefArithTarget::Reg(Reg8::D)),
        0xEB => Instruction::Set(Bit::B5, PrefArithTarget::Reg(Reg8::E)),
        0xEC => Instruction::Set(Bit::B5, PrefArithTarget::Reg(Reg8::H)),
        0xED => Instruction::Set(Bit::B5, PrefArithTarget::Reg(Reg8::L)),
        0xEE => Instruction::Set(Bit::B5, PrefArithTarget::MemHL),
        0xEF => Instruction::Set(Bit::B5, PrefArithTarget::Reg(Reg8::A)),
        0xF0 => Instruction::Set(Bit::B6, PrefArithTarget::Reg(Reg8::B)),
        0xF1 => Instruction::Set(Bit::B6, PrefArithTarget::Reg(Reg8::C)),
        0xF2 => Instruction::Set(Bit::B6, PrefArithTarget::Reg(Reg8::D)),
        0xF3 => Instruction::Set(Bit::B6, PrefArithTarget::Reg(Reg8::E)),
        0xF4 => Instruction::Set(Bit::B6, PrefArithTarget::Reg(Reg8::H)),
        0xF5 => Instruction::Set(Bit::B6, PrefArithTarget::Reg(Reg8::L)),
        0xF6 => Instruction::Set(Bit::B6, PrefArithTarget::MemHL),
        0xF7 => Instruction::Set(Bit::B6, PrefArithTarget::Reg(Reg8::A)),
        0xF8 => Instruction::Set(Bit::B7, PrefArithTarget::Reg(Reg8::B)),
        0xF9 => Instruction::Set(Bit::B7, PrefArithTarget::Reg(Reg8::C)),
        0xFA => Instruction::Set(Bit::B7, PrefArithTarget::Reg(Reg8::D)),
        0xFB => Instruction::Set(Bit::B7, PrefArithTarget::Reg(Reg8::E)),
        0xFC => Instruction::Set(Bit::B7, PrefArithTarget::Reg(Reg8::H)),
        0xFD => Instruction::Set(Bit::B7, PrefArithTarget::Reg(Reg8::L)),
        0xFE => Instruction::Set(Bit::B7, PrefArithTarget::MemHL),
        0xFF => Instruction::Set(Bit::B7, PrefArithTarget::Reg(Reg8::A)),
    }
}

pub trait DecoderReadable {
    type Err;
    fn read_at(&self, idx: usize) -> Result<u8, Self::Err>;
}

impl DecoderReadable for &[u8] {
    type Err = DecodeError;
    fn read_at(&self, idx: usize) -> Result<u8, Self::Err> {
        self.get(idx).cloned().ok_or(DecodeError::NotEnoughBytes)
    }
}

fn read8<T: DecoderReadable>(mem: &T, idx: u16) -> Result<u8, T::Err> {
    mem.read_at(idx as usize)
}

fn read16<T: DecoderReadable>(mem: &T, idx: u16) -> Result<u16, T::Err> {
    let b1 = mem.read_at(idx as usize)?;
    let b2 = mem.read_at((idx + 1) as usize)?;

    Ok(u16::from_le_bytes([b1, b2]))
}

pub fn decode<T: DecoderReadable>(mem: &T, pc: u16) -> Result<Instruction, T::Err> {
    let opcode = read8(mem, pc)?;

    let instr = match opcode {
        // 0x0_
        0x00 => Instruction::Nop,
        0x01 => Instruction::Load16(Ld16Dst::Reg(Reg16::BC), Ld16Src::Imm(read16(mem, pc + 1)?)),
        0x02 => Instruction::Load8(Ld8Dst::Mem(MemLoc::Reg(Reg16::BC)), Ld8Src::Reg(Reg8::A)),
        0x03 => Instruction::Inc(IncDecTarget::Reg16(Reg16::BC)),
        0x04 => Instruction::Inc(IncDecTarget::Reg8(Reg8::B)),
        0x05 => Instruction::Dec(IncDecTarget::Reg8(Reg8::B)),
        0x06 => Instruction::Load8(Ld8Dst::Reg(Reg8::B), Ld8Src::Imm(read8(mem, pc + 1)?)),
        0x07 => Instruction::RotLeftCircularA,
        0x08 => Instruction::Load16(
            Ld16Dst::Mem(MemLoc::Imm(read16(mem, pc + 1)?)),
            Ld16Src::Reg(Reg16::SP),
        ),
        0x09 => Instruction::AddHL(Reg16::BC),
        0x0A => Instruction::Load8(Ld8Dst::Reg(Reg8::A), Ld8Src::Mem(MemLoc::Reg(Reg16::BC))),
        0x0B => Instruction::Dec(IncDecTarget::Reg16(Reg16::BC)),
        0x0C => Instruction::Inc(IncDecTarget::Reg8(Reg8::C)),
        0x0D => Instruction::Dec(IncDecTarget::Reg8(Reg8::C)),
        0x0E => Instruction::Load8(Ld8Dst::Reg(Reg8::C), Ld8Src::Imm(read8(mem, pc + 1)?)),
        0x0F => Instruction::RotRightCircularA,

        // 0x1_
        0x10 => Instruction::Stop(read8(mem, pc + 1)?),
        0x11 => Instruction::Load16(Ld16Dst::Reg(Reg16::DE), Ld16Src::Imm(read16(mem, pc + 1)?)),
        0x12 => Instruction::Load8(Ld8Dst::Mem(MemLoc::Reg(Reg16::DE)), Ld8Src::Reg(Reg8::A)),
        0x13 => Instruction::Inc(IncDecTarget::Reg16(Reg16::DE)),
        0x14 => Instruction::Inc(IncDecTarget::Reg8(Reg8::D)),
        0x15 => Instruction::Dec(IncDecTarget::Reg8(Reg8::D)),
        0x16 => Instruction::Load8(Ld8Dst::Reg(Reg8::D), Ld8Src::Imm(read8(mem, pc + 1)?)),
        0x17 => Instruction::RotLeftA,
        0x18 => Instruction::JumpRel(read8(mem, pc + 1)? as i8),
        0x19 => Instruction::AddHL(Reg16::DE),
        0x1A => Instruction::Load8(Ld8Dst::Reg(Reg8::A), Ld8Src::Mem(MemLoc::Reg(Reg16::DE))),
        0x1B => Instruction::Dec(IncDecTarget::Reg16(Reg16::DE)),
        0x1C => Instruction::Inc(IncDecTarget::Reg8(Reg8::E)),
        0x1D => Instruction::Dec(IncDecTarget::Reg8(Reg8::E)),
        0x1E => Instruction::Load8(Ld8Dst::Reg(Reg8::E), Ld8Src::Imm(read8(mem, pc + 1)?)),
        0x1F => Instruction::RotRightA,

        // 0x2_
        0x20 => Instruction::JumpRelIf(read8(mem, pc + 1)? as i8, Condition::NotZero),
        0x21 => Instruction::Load16(Ld16Dst::Reg(Reg16::HL), Ld16Src::Imm(read16(mem, pc + 1)?)),
        0x22 => Instruction::LoadAtoHLI,
        0x23 => Instruction::Inc(IncDecTarget::Reg16(Reg16::HL)),
        0x24 => Instruction::Inc(IncDecTarget::Reg8(Reg8::H)),
        0x25 => Instruction::Dec(IncDecTarget::Reg8(Reg8::H)),
        0x26 => Instruction::Load8(Ld8Dst::Reg(Reg8::H), Ld8Src::Imm(read8(mem, pc + 1)?)),
        0x27 => Instruction::DecimalAdjust,
        0x28 => Instruction::JumpRelIf(read8(mem, pc + 1)? as i8, Condition::Zero),
        0x29 => Instruction::AddHL(Reg16::HL),
        0x2A => Instruction::LoadHLItoA,
        0x2B => Instruction::Dec(IncDecTarget::Reg16(Reg16::HL)),
        0x2C => Instruction::Inc(IncDecTarget::Reg8(Reg8::L)),
        0x2D => Instruction::Dec(IncDecTarget::Reg8(Reg8::L)),
        0x2E => Instruction::Load8(Ld8Dst::Reg(Reg8::L), Ld8Src::Imm(read8(mem, pc + 1)?)),
        0x2F => Instruction::ComplementAccumulator,

        // 0x3_
        0x30 => Instruction::JumpRelIf(read8(mem, pc + 1)? as i8, Condition::NotCarry),
        0x31 => Instruction::Load16(Ld16Dst::Reg(Reg16::SP), Ld16Src::Imm(read16(mem, pc + 1)?)),
        0x32 => Instruction::LoadAtoHLD,
        0x33 => Instruction::Inc(IncDecTarget::Reg16(Reg16::SP)),
        0x34 => Instruction::Inc(IncDecTarget::MemHL),
        0x35 => Instruction::Dec(IncDecTarget::MemHL),
        0x36 => Instruction::Load8(
            Ld8Dst::Mem(MemLoc::Reg(Reg16::HL)),
            Ld8Src::Imm(read8(mem, pc + 1)?),
        ),
        0x37 => Instruction::SetCarryFlag,
        0x38 => Instruction::JumpRelIf(read8(mem, pc + 1)? as i8, Condition::Carry),
        0x39 => Instruction::AddHL(Reg16::SP),
        0x3A => Instruction::LoadHLDtoA,
        0x3B => Instruction::Dec(IncDecTarget::Reg16(Reg16::SP)),
        0x3C => Instruction::Inc(IncDecTarget::Reg8(Reg8::A)),
        0x3D => Instruction::Dec(IncDecTarget::Reg8(Reg8::A)),
        0x3E => Instruction::Load8(Ld8Dst::Reg(Reg8::A), Ld8Src::Imm(read8(mem, pc + 1)?)),
        0x3F => Instruction::ComplementCarry,

        // 0x4_
        0x40 => ld_regs!(B, B),
        0x41 => ld_regs!(B, C),
        0x42 => ld_regs!(B, D),
        0x43 => ld_regs!(B, E),
        0x44 => ld_regs!(B, H),
        0x45 => ld_regs!(B, L),
        0x46 => ld_reg_hl!(B),
        0x47 => ld_regs!(B, A),
        0x48 => ld_regs!(C, B),
        0x49 => ld_regs!(C, C),
        0x4A => ld_regs!(C, D),
        0x4B => ld_regs!(C, E),
        0x4C => ld_regs!(C, H),
        0x4D => ld_regs!(C, L),
        0x4E => ld_reg_hl!(C),
        0x4F => ld_regs!(C, A),

        // 0x5_
        0x50 => ld_regs!(D, B),
        0x51 => ld_regs!(D, C),
        0x52 => ld_regs!(D, D),
        0x53 => ld_regs!(D, E),
        0x54 => ld_regs!(D, H),
        0x55 => ld_regs!(D, L),
        0x56 => ld_reg_hl!(D),
        0x57 => ld_regs!(D, A),
        0x58 => ld_regs!(E, B),
        0x59 => ld_regs!(E, C),
        0x5A => ld_regs!(E, D),
        0x5B => ld_regs!(E, E),
        0x5C => ld_regs!(E, H),
        0x5D => ld_regs!(E, L),
        0x5E => ld_reg_hl!(E),
        0x5F => ld_regs!(E, A),

        // 0x6_
        0x60 => ld_regs!(H, B),
        0x61 => ld_regs!(H, C),
        0x62 => ld_regs!(H, D),
        0x63 => ld_regs!(H, E),
        0x64 => ld_regs!(H, H),
        0x65 => ld_regs!(H, L),
        0x66 => ld_reg_hl!(H),
        0x67 => ld_regs!(H, A),
        0x68 => ld_regs!(L, B),
        0x69 => ld_regs!(L, C),
        0x6A => ld_regs!(L, D),
        0x6B => ld_regs!(L, E),
        0x6C => ld_regs!(L, H),
        0x6D => ld_regs!(L, L),
        0x6E => ld_reg_hl!(L),
        0x6F => ld_regs!(L, A),

        // 0x7_
        0x70 => ld_hl_reg!(B),
        0x71 => ld_hl_reg!(C),
        0x72 => ld_hl_reg!(D),
        0x73 => ld_hl_reg!(E),
        0x74 => ld_hl_reg!(H),
        0x75 => ld_hl_reg!(L),
        0x76 => Instruction::Halt,
        0x77 => ld_hl_reg!(A),
        0x78 => ld_regs!(A, B),
        0x79 => ld_regs!(A, C),
        0x7A => ld_regs!(A, D),
        0x7B => ld_regs!(A, E),
        0x7C => ld_regs!(A, H),
        0x7D => ld_regs!(A, L),
        0x7E => ld_reg_hl!(A),
        0x7F => ld_regs!(A, A),

        // 0x8_
        0x80 => add_reg!(B),
        0x81 => add_reg!(C),
        0x82 => add_reg!(D),
        0x83 => add_reg!(E),
        0x84 => add_reg!(H),
        0x85 => add_reg!(L),
        0x86 => Instruction::Add(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0x87 => add_reg!(A),
        0x88 => add_carry_reg!(B),
        0x89 => add_carry_reg!(C),
        0x8A => add_carry_reg!(D),
        0x8B => add_carry_reg!(E),
        0x8C => add_carry_reg!(H),
        0x8D => add_carry_reg!(L),
        0x8E => Instruction::AddCarry(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0x8F => add_carry_reg!(A),

        // 0x9_
        0x90 => sub_reg!(B),
        0x91 => sub_reg!(C),
        0x92 => sub_reg!(D),
        0x93 => sub_reg!(E),
        0x94 => sub_reg!(H),
        0x95 => sub_reg!(L),
        0x96 => Instruction::Sub(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0x97 => sub_reg!(A),
        0x98 => sub_carry_reg!(B),
        0x99 => sub_carry_reg!(C),
        0x9A => sub_carry_reg!(D),
        0x9B => sub_carry_reg!(E),
        0x9C => sub_carry_reg!(H),
        0x9D => sub_carry_reg!(L),
        0x9E => Instruction::SubCarry(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0x9F => sub_carry_reg!(A),

        // 0xA_
        0xA0 => and_reg!(B),
        0xA1 => and_reg!(C),
        0xA2 => and_reg!(D),
        0xA3 => and_reg!(E),
        0xA4 => and_reg!(H),
        0xA5 => and_reg!(L),
        0xA6 => Instruction::And(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0xA7 => and_reg!(A),
        0xA8 => xor_reg!(B),
        0xA9 => xor_reg!(C),
        0xAA => xor_reg!(D),
        0xAB => xor_reg!(E),
        0xAC => xor_reg!(H),
        0xAD => xor_reg!(L),
        0xAE => Instruction::Xor(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0xAF => xor_reg!(A),

        // 0xB_
        0xB0 => or_reg!(B),
        0xB1 => or_reg!(C),
        0xB2 => or_reg!(D),
        0xB3 => or_reg!(E),
        0xB4 => or_reg!(H),
        0xB5 => or_reg!(L),
        0xB6 => Instruction::Or(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0xB7 => or_reg!(A),
        0xB8 => cmp_reg!(B),
        0xB9 => cmp_reg!(C),
        0xBA => cmp_reg!(D),
        0xBB => cmp_reg!(E),
        0xBC => cmp_reg!(H),
        0xBD => cmp_reg!(L),
        0xBE => Instruction::Cmp(ArithSrc::Mem(MemLoc::Reg(Reg16::HL))),
        0xBF => cmp_reg!(A),

        // 0xC_
        0xC0 => Instruction::RetIf(Condition::NotZero),
        0xC1 => Instruction::Pop(Reg16::BC),
        0xC2 => Instruction::JumpIf(read16(mem, pc + 1)?, Condition::NotZero),
        0xC3 => Instruction::Jump(read16(mem, pc + 1)?),
        0xC4 => Instruction::CallIf(read16(mem, pc + 1)?, Condition::NotZero),
        0xC5 => Instruction::Push(Reg16::BC),
        0xC6 => Instruction::Add(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xC7 => Instruction::Rst(RsVec::Rst0),
        0xC8 => Instruction::RetIf(Condition::Zero),
        0xC9 => Instruction::Ret,
        0xCA => Instruction::JumpIf(read16(mem, pc + 1)?, Condition::Zero),
        0xCB => decode_prefixed(read8(mem, pc + 1)?), // Special instruction, maps to another instruction set
        0xCC => Instruction::CallIf(read16(mem, pc + 1)?, Condition::Zero),
        0xCD => Instruction::Call(read16(mem, pc + 1)?),
        0xCE => Instruction::AddCarry(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xCF => Instruction::Rst(RsVec::Rst1),

        // 0xD_
        0xD0 => Instruction::RetIf(Condition::NotCarry),
        0xD1 => Instruction::Pop(Reg16::DE),
        0xD2 => Instruction::JumpIf(read16(mem, pc + 1)?, Condition::NotCarry),
        0xD3 => illegal!(0xD3),
        0xD4 => Instruction::CallIf(read16(mem, pc + 1)?, Condition::NotCarry),
        0xD5 => Instruction::Push(Reg16::DE),
        0xD6 => Instruction::Sub(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xD7 => Instruction::Rst(RsVec::Rst2),
        0xD8 => Instruction::RetIf(Condition::Carry),
        0xD9 => Instruction::Reti,
        0xDA => Instruction::JumpIf(read16(mem, pc + 1)?, Condition::Carry),
        0xDB => illegal!(0xDB),
        0xDC => Instruction::CallIf(read16(mem, pc + 1)?, Condition::Carry),
        0xDD => illegal!(0xDD),
        0xDE => Instruction::SubCarry(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xDF => Instruction::Rst(RsVec::Rst3),

        // 0xE_
        0xE0 => Instruction::Load8(
            Ld8Dst::Mem(MemLoc::HighMemImm(read8(mem, pc + 1)?)),
            Ld8Src::Reg(Reg8::A),
        ),
        0xE1 => Instruction::Pop(Reg16::HL),
        0xE2 => Instruction::Load8(
            Ld8Dst::Mem(MemLoc::HighMemReg(Reg8::C)),
            Ld8Src::Reg(Reg8::A),
        ),
        0xE3 => illegal!(0xE3),
        0xE4 => illegal!(0xE4),
        0xE5 => Instruction::Push(Reg16::HL),
        0xE6 => Instruction::And(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xE7 => Instruction::Rst(RsVec::Rst4),
        0xE8 => Instruction::AddSP(read8(mem, pc + 1)? as i8),
        0xE9 => Instruction::JumpHL,
        0xEA => Instruction::Load8(
            Ld8Dst::Mem(MemLoc::Imm(read16(mem, pc + 1)?)),
            Ld8Src::Reg(Reg8::A),
        ),
        0xEB => illegal!(0xEB),
        0xEC => illegal!(0xEC),
        0xED => illegal!(0xED),
        0xEE => Instruction::Xor(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xEF => Instruction::Rst(RsVec::Rst5),

        // 0xF_
        0xF0 => Instruction::Load8(
            Ld8Dst::Reg(Reg8::A),
            Ld8Src::Mem(MemLoc::HighMemImm(read8(mem, pc + 1)?)),
        ),
        0xF1 => Instruction::Pop(Reg16::AF),
        0xF2 => Instruction::Load8(
            Ld8Dst::Reg(Reg8::A),
            Ld8Src::Mem(MemLoc::HighMemReg(Reg8::C)),
        ),
        0xF3 => Instruction::DI,
        0xF4 => illegal!(0xF4),
        0xF5 => Instruction::Push(Reg16::AF),
        0xF6 => Instruction::Or(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xF7 => Instruction::Rst(RsVec::Rst6),
        0xF8 => Instruction::LoadSPi8toHL(read8(mem, pc + 1)? as i8),
        0xF9 => Instruction::Load16(Ld16Dst::Reg(Reg16::SP), Ld16Src::Reg(Reg16::HL)),
        0xFA => Instruction::Load8(
            Ld8Dst::Reg(Reg8::A),
            Ld8Src::Mem(MemLoc::Imm(read16(mem, pc + 1)?)),
        ),
        0xFB => Instruction::EI,
        0xFC => illegal!(0xFC),
        0xFD => illegal!(0xFD),
        0xFE => Instruction::Cmp(ArithSrc::Imm(read8(mem, pc + 1)?)),
        0xFF => Instruction::Rst(RsVec::Rst7),
    };

    Ok(instr)
}

#[cfg(test)]
mod tests {
    use crate::isa::testutils;

    use super::*;

    #[test]
    fn all_decode_ok() {
        for opcode in testutils::legal_instrs() {
            let result = decode(&opcode.as_slice(), 0x0);

            assert!(result.is_ok(), "Opcode {:?} not decoded!", opcode);
            assert!(
                !matches!(result.unwrap(), Instruction::IllegalInstruction(_)),
                "Opcode {:x?} was decoded as illegal!",
                opcode
            );
        }
    }

    #[test]
    fn decode_illegals() {
        for opcode in testutils::illegal_opcodes() {
            let result = decode(&[opcode].as_slice(), 0x0);

            assert!(result.is_ok());

            match result.unwrap() {
                Instruction::IllegalInstruction(illegal_opcode) => {
                    assert_eq!(opcode, illegal_opcode, "Wrong illegal opcode detected")
                }
                _ => panic!("Illegal opcode was decoded"),
            }
        }
    }
}
