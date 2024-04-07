
    
use crate::{isa::{ArithSrc, Condition, IncDecTarget, Instruction, Ld16Dst, Ld16Src, Ld8Dst, Ld8Src, MemLoc, Register16, Register8}, memcontroller::MemController};

use super::RawInstruction;

#[derive(Debug)]
pub enum DecodeError {
    NotYetImplemented,
}

macro_rules! err_not_yet_impl {
    () => {
        return Err(DecodeError::NotYetImplemented)
    };
}

macro_rules! reljump {
    ($addr:expr, $mem:expr) => {
        $addr.wrapping_add_signed($mem.read8($addr) as i8 as i16)
    };
}

macro_rules! ld_regs {
    ($dst:ident, $src:ident) => {
        Instruction::Load8(Ld8Dst::Reg(Register8::$dst), Ld8Src::Reg(Register8::$src))
    };
}

macro_rules! ld_reg_hl {
    ($dst:ident) => {
        Instruction::Load8(Ld8Dst::Reg(Register8::$dst), Ld8Src::Mem(MemLoc::Reg(Register16::HL)))
    };
}

macro_rules! ld_hl_reg {
    ($src:ident) => {
        Instruction::Load8(Ld8Dst::Mem(MemLoc::Reg(Register16::HL)), Ld8Src::Reg(Register8::$src)) 
    };
}

macro_rules! add_reg {
    ($src:ident) => {
        Instruction::Add(ArithSrc::Reg(Register8::$src))
    };
}

macro_rules! add_carry_reg {
    ($src:ident) => {
        Instruction::AddCarry(ArithSrc::Reg(Register8::$src))
    };
}

macro_rules! sub_reg {
    ($src:ident) => {
        Instruction::Sub(ArithSrc::Reg(Register8::$src))
    };
}

macro_rules! sub_carry_reg {
    ($src:ident) => {
        Instruction::SubCarry(ArithSrc::Reg(Register8::$src))
    };
}

macro_rules! and_reg {
    ($src:ident) => {
        Instruction::And(ArithSrc::Reg(Register8::$src))
    };
}

macro_rules! or_reg {
    ($src:ident) => {
        Instruction::Or(ArithSrc::Reg(Register8::$src))
    };
}

macro_rules! xor_reg {
    ($src:ident) => {
        Instruction::Xor(ArithSrc::Reg(Register8::$src))
    };
}

macro_rules! cmp_reg {
    ($src:ident) => {
        Instruction::Cmp(ArithSrc::Reg(Register8::$src))
    };
}
const fn decode_prefixed(instr: u8) -> Result<Instruction, DecodeError> {
    match instr {
        _ => Err(DecodeError::NotYetImplemented)
    }
}

pub fn decode(mem: &impl MemController, pc: u16) -> Result<Instruction, DecodeError> {
    let opcode = mem.read8(pc);

    let instr = match opcode {
        // 0x0_
        0x00 => Instruction::Nop,
        0x01 => Instruction::Load16(Ld16Dst::Reg(Register16::BC), Ld16Src::Imm(mem.read16(pc + 1))),
        0x02 => Instruction::Load8(Ld8Dst::Mem(MemLoc::Reg(Register16::BC)), Ld8Src::Reg(Register8::A)),
        0x03 => Instruction::Inc(IncDecTarget::Reg16(Register16::BC)),
        0x04 => Instruction::Inc(IncDecTarget::Reg8(Register8::B)),
        0x05 => Instruction::Dec(IncDecTarget::Reg8(Register8::B)),
        0x06 => Instruction::Load8(Ld8Dst::Reg(Register8::C), Ld8Src::Imm(mem.read8(pc + 1))),
        0x07 => err_not_yet_impl!(),
        0x08 => Instruction::Load16(Ld16Dst::Mem(MemLoc::Imm(mem.read16(pc + 1))), Ld16Src::Reg(Register16::SP)),
        0x09 => Instruction::AddHL(Register16::BC),
        0x0A => Instruction::Load8(Ld8Dst::Reg(Register8::A), Ld8Src::Mem(MemLoc::Reg(Register16::BC))),
        0x0B => Instruction::Dec(IncDecTarget::Reg16(Register16::BC)),
        0x0C => Instruction::Inc(IncDecTarget::Reg8(Register8::C)),
        0x0D => Instruction::Dec(IncDecTarget::Reg8(Register8::C)),
        0x0E => Instruction::Load8(Ld8Dst::Reg(Register8::C), Ld8Src::Imm(mem.read8(pc + 1))),
        0x0F => err_not_yet_impl!(),

        // 0x1_
        0x10 => Instruction::Stop,
        0x11 => Instruction::Load16(Ld16Dst::Reg(Register16::DE), Ld16Src::Imm(mem.read16(pc + 1))),
        0x12 => Instruction::Load8(Ld8Dst::Mem(MemLoc::Reg(Register16::DE)), Ld8Src::Reg(Register8::A)),
        0x13 => Instruction::Inc(IncDecTarget::Reg16(Register16::DE)),
        0x14 => Instruction::Inc(IncDecTarget::Reg8(Register8::D)),
        0x15 => Instruction::Dec(IncDecTarget::Reg8(Register8::D)),
        0x16 => Instruction::Load8(Ld8Dst::Reg(Register8::D), Ld8Src::Imm(mem.read8(pc + 1))),
        0x17 => err_not_yet_impl!(),
        0x18 => Instruction::Jump(reljump!(pc + 1, mem)),
        0x19 => Instruction::AddHL(Register16::DE),
        0x1A => Instruction::Load8(Ld8Dst::Reg(Register8::A), Ld8Src::Mem(MemLoc::Reg(Register16::DE))),
        0x1B => Instruction::Dec(IncDecTarget::Reg16(Register16::DE)),
        0x1C => Instruction::Inc(IncDecTarget::Reg8(Register8::E)),
        0x1D => Instruction::Dec(IncDecTarget::Reg8(Register8::E)),
        0x1E => Instruction::Load8(Ld8Dst::Reg(Register8::E), Ld8Src::Imm(mem.read8(pc + 1))),
        0x1F => err_not_yet_impl!(),

        // 0x2_
        0x20 => Instruction::JumpIf(reljump!(pc + 1, mem), Condition::NotZero),
        0x21 => Instruction::Load16(Ld16Dst::Reg(Register16::HL), Ld16Src::Imm(mem.read16(pc + 1))),
        0x22 => Instruction::LoadAtoHLI,
        0x23 => Instruction::Inc(IncDecTarget::Reg16(Register16::HL)),
        0x24 => Instruction::Inc(IncDecTarget::Reg8(Register8::H)),
        0x25 => Instruction::Dec(IncDecTarget::Reg8(Register8::H)),
        0x26 => Instruction::Load8(Ld8Dst::Reg(Register8::H), Ld8Src::Imm(mem.read8(pc + 1))),
        0x27 => err_not_yet_impl!(),
        0x28 => Instruction::JumpIf(reljump!(pc + 1, mem), Condition::Zero),
        0x29 => Instruction::AddHL(Register16::HL),
        0x2A => Instruction::LoadHLItoA,
        0x2B => Instruction::Dec(IncDecTarget::Reg16(Register16::HL)),
        0x2C => Instruction::Inc(IncDecTarget::Reg8(Register8::L)),
        0x2D => Instruction::Dec(IncDecTarget::Reg8(Register8::L)),
        0x2E => Instruction::Load8(Ld8Dst::Reg(Register8::L), Ld8Src::Imm(mem.read8(pc + 1))),
        0x2F => err_not_yet_impl!(),

        // 0x3_
        0x30 => Instruction::JumpIf(reljump!(pc + 1, mem), Condition::NotCarry),
        0x31 => Instruction::Load16(Ld16Dst::Reg(Register16::SP), Ld16Src::Imm(mem.read16(pc + 1))),
        0x32 => Instruction::LoadAtoHLD,
        0x33 => Instruction::Inc(IncDecTarget::Reg16(Register16::SP)),
        0x34 => Instruction::Inc(IncDecTarget::Mem(MemLoc::Reg(Register16::HL))),
        0x35 => Instruction::Dec(IncDecTarget::Mem(MemLoc::Reg(Register16::HL))),
        0x36 => Instruction::Load8(Ld8Dst::Mem(MemLoc::Reg(Register16::HL)), Ld8Src::Imm(mem.read8(pc + 1))),
        0x37 => err_not_yet_impl!(),
        0x38 => Instruction::JumpIf(reljump!(pc + 1, mem), Condition::Carry),
        0x39 => Instruction::AddHL(Register16::SP),
        0x3A => Instruction::LoadHLDtoA,
        0x3B => Instruction::Dec(IncDecTarget::Reg16(Register16::SP)),
        0x3C => Instruction::Inc(IncDecTarget::Reg8(Register8::A)),
        0x3D => Instruction::Dec(IncDecTarget::Reg8(Register8::A)),
        0x3E => Instruction::Load8(Ld8Dst::Reg(Register8::A), Ld8Src::Imm(mem.read8(pc + 1))),
        0x3F => err_not_yet_impl!(),

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
        0x86 => Instruction::Add(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0x87 => add_reg!(A),
        0x88 => add_carry_reg!(B),
        0x89 => add_carry_reg!(C),
        0x8A => add_carry_reg!(D),
        0x8B => add_carry_reg!(E),
        0x8C => add_carry_reg!(H),
        0x8D => add_carry_reg!(L),
        0x8E => Instruction::AddCarry(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0x8F => add_carry_reg!(A),
        
        // 0x9_
        0x90 => sub_reg!(B),
        0x91 => sub_reg!(C),
        0x92 => sub_reg!(D),
        0x93 => sub_reg!(E),
        0x94 => sub_reg!(H),
        0x95 => sub_reg!(L),
        0x96 => Instruction::Sub(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0x97 => sub_reg!(A),
        0x98 => sub_carry_reg!(B),
        0x99 => sub_carry_reg!(C),
        0x9A => sub_carry_reg!(D),
        0x9B => sub_carry_reg!(E),
        0x9C => sub_carry_reg!(H),
        0x9D => sub_carry_reg!(L),
        0x9E => Instruction::SubCarry(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0x9F => sub_carry_reg!(A),

        // 0xA_
        0xA0 => and_reg!(B),
        0xA1 => and_reg!(C),
        0xA2 => and_reg!(D),
        0xA3 => and_reg!(E),
        0xA4 => and_reg!(H),
        0xA5 => and_reg!(L),
        0xA6 => Instruction::And(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0xA7 => and_reg!(A),
        0xA8 => xor_reg!(B),
        0xA9 => xor_reg!(C),
        0xAA => xor_reg!(D),
        0xAB => xor_reg!(E),
        0xAC => xor_reg!(H),
        0xAD => xor_reg!(L),
        0xAE => Instruction::Xor(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0xAF => xor_reg!(A),

        // 0xB_
        0xB0 => or_reg!(B),
        0xB1 => or_reg!(C),
        0xB2 => or_reg!(D),
        0xB3 => or_reg!(E),
        0xB4 => or_reg!(H),
        0xB5 => or_reg!(L),
        0xB6 => Instruction::Or(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0xB7 => or_reg!(A),
        0xB8 => cmp_reg!(B),
        0xB9 => cmp_reg!(C),
        0xBA => cmp_reg!(D),
        0xBB => cmp_reg!(E),
        0xBC => cmp_reg!(H),
        0xBD => cmp_reg!(L),
        0xBE => Instruction::Cmp(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
        0xBF => cmp_reg!(A),

        // 0xC_
        0xC0 => Instruction::RetIf(Condition::NotZero),
        0xC1 => Instruction::Pop(Register16::BC),
        0xC2 => Instruction::JumpIf(mem.read16(pc + 1), Condition::NotZero),
        0xC3 => Instruction::Jump(mem.read16(pc + 1)),
        0xC4 => Instruction::CallIf(mem.read16(pc + 1), Condition::NotZero),
        0xC5 => Instruction::Push(Register16::BC),
        0xC6 => Instruction::Add(ArithSrc::Imm(mem.read8(pc + 1))),
        0xC7 => err_not_yet_impl!(),
        0xC8 => Instruction::RetIf(Condition::Zero),
        0xC9 => Instruction::Ret,
        0xCA => Instruction::JumpIf(mem.read16(pc + 1), Condition::Zero),
        0xCB => decode_prefixed(mem.read8(pc + 1))?, // Special instruction, maps to another instruction set
        0xCC => Instruction::CallIf(mem.read16(pc + 1), Condition::Zero),
        0xCD => Instruction::Call(mem.read16(pc + 1)),
        0xCE => Instruction::AddCarry(ArithSrc::Imm(mem.read8(pc + 1))),
        0xCF => err_not_yet_impl!(),

        // 0xD_
        0xD0 => Instruction::RetIf(Condition::NotCarry),
        0xD1 => Instruction::Pop(Register16::DE),
        0xD2 => Instruction::JumpIf(mem.read16(pc + 1), Condition::NotCarry),
        0xD3 => Instruction::IllegalInstruction(RawInstruction::Single(0xD3)),
        0xD4 => Instruction::CallIf(mem.read16(pc + 1), Condition::NotCarry),
        0xD5 => Instruction::Push(Register16::DE),
        0xD6 => Instruction::Sub(ArithSrc::Imm(mem.read8(pc + 1))),
        0xD7 => err_not_yet_impl!(),
        0xD8 => Instruction::RetIf(Condition::Carry),
        0xD9 => Instruction::Reti,
        0xDA => Instruction::JumpIf(mem.read16(pc + 1), Condition::Carry),
        0xDB => Instruction::IllegalInstruction(RawInstruction::Single(0xDB)),
        0xDC => Instruction::CallIf(mem.read16(pc + 1), Condition::Carry),
        0xDD => Instruction::IllegalInstruction(RawInstruction::Single(0xDD)),
        0xDE => Instruction::SubCarry(ArithSrc::Imm(mem.read8(pc + 1))),
        0xDF => err_not_yet_impl!(),

        // 0xE_

        // 0xF_

        _ => err_not_yet_impl!()
    };

    Ok(instr)
}
