use crate::memcontroller::MemController;

#[derive(Debug, Copy, Clone)]
pub(crate) enum RawInstruction {
    Single(u8),
    Double(u16)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Register8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Register16 {
    BC,
    DE,
    HL,
    SP
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum MemLoc {
    Reg(Register16),
    Imm(u16)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ArithSrc {
    Reg(Register8),
    Imm(u8),
    Mem(MemLoc)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Ld8Src {
    Reg(Register8),
    Mem(MemLoc),
    Imm(u8),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Ld8Dst {
    Mem(MemLoc),
    Reg(Register8)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Ld16Src {
    Reg(Register16),
    Imm(u16),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Ld16Dst {
    Mem(MemLoc),
    Reg(Register16)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum IncDecTarget {
    Reg8(Register8),
    Reg16(Register16),
    Mem(MemLoc)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Instruction {
    /// No operation
    Nop,

    /// Add value from source to register A, store result in A
    Add(ArithSrc),

    /// Add value from source to register HL, store result in HL
    AddHL(Register16),

    /// Subtract value from source from register A, store result in A
    Sub(ArithSrc),

    /// Bitwise AND of register A and source, store result in A
    And(ArithSrc),

    /// Bitwise OR of register A and source, store result in A
    Or(ArithSrc),

    /// Increment value at target
    Inc(IncDecTarget),

    /// Decrement value at target
    Dec(IncDecTarget),

    /// Load 8 bit value from source to destination
    Load8(Ld8Dst, Ld8Src),

    /// Load 16 bit value from source to destination
    Load16(Ld16Dst, Ld16Src),

    /// Illegal instruction, stop CPU
    IllegalInstruction(RawInstruction)
}

#[derive(Debug)]
pub enum DecodeError {
    NotYetImplemented,
}

macro_rules! not_yet_implemented {
    () => {
        return Err(DecodeError::NotYetImplemented)
    };
}

impl Instruction {
    const fn decode_prefixed(instr: u8) -> Result<Instruction, DecodeError> {
        match instr {
            _ => Err(DecodeError::NotYetImplemented)
        }
    }

    pub fn decode(mem: &MemController, pc: u16) -> Result<Instruction, DecodeError> {
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
            0x07 => not_yet_implemented!(),
            0x08 => Instruction::Load16(Ld16Dst::Mem(MemLoc::Imm(mem.read16(pc + 1))), Ld16Src::Reg(Register16::SP)),
            0x09 => Instruction::AddHL(Register16::BC),
            0x0A => Instruction::Load8(Ld8Dst::Reg(Register8::A), Ld8Src::Mem(MemLoc::Reg(Register16::BC))),
            0x0B => Instruction::Dec(IncDecTarget::Reg16(Register16::BC)),
            0x0C => Instruction::Inc(IncDecTarget::Reg8(Register8::C)),
            0x0D => Instruction::Dec(IncDecTarget::Reg8(Register8::C)),
            0x0E => Instruction::Load8(Ld8Dst::Reg(Register8::C), Ld8Src::Imm(mem.read8(pc + 1))),
            0x0F => not_yet_implemented!(),

            // 0x1_
            0x19 => Instruction::AddHL(Register16::DE),

            // 0x2_
            0x29 => Instruction::AddHL(Register16::HL),

            // 0x3_
            0x39 => Instruction::AddHL(Register16::SP),

            // 0x8_
            0x80 => Instruction::Add(ArithSrc::Reg(Register8::B)),
            0x81 => Instruction::Add(ArithSrc::Reg(Register8::C)),
            0x82 => Instruction::Add(ArithSrc::Reg(Register8::D)),
            0x83 => Instruction::Add(ArithSrc::Reg(Register8::E)),
            0x84 => Instruction::Add(ArithSrc::Reg(Register8::H)),
            0x85 => Instruction::Add(ArithSrc::Reg(Register8::L)),
            0x86 => Instruction::Add(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
            0x87 => Instruction::Add(ArithSrc::Reg(Register8::A)),
            
            // 0x9_
            0x90 => Instruction::Sub(ArithSrc::Reg(Register8::B)),
            0x91 => Instruction::Sub(ArithSrc::Reg(Register8::C)),
            0x92 => Instruction::Sub(ArithSrc::Reg(Register8::D)),
            0x93 => Instruction::Sub(ArithSrc::Reg(Register8::E)),
            0x94 => Instruction::Sub(ArithSrc::Reg(Register8::H)),
            0x95 => Instruction::Sub(ArithSrc::Reg(Register8::L)),
            0x96 => Instruction::Sub(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
            0x97 => Instruction::Sub(ArithSrc::Reg(Register8::A)),

            // 0xA_
            0xA0 => Instruction::And(ArithSrc::Reg(Register8::B)),
            0xA1 => Instruction::And(ArithSrc::Reg(Register8::C)),
            0xA2 => Instruction::And(ArithSrc::Reg(Register8::D)),
            0xA3 => Instruction::And(ArithSrc::Reg(Register8::E)),
            0xA4 => Instruction::And(ArithSrc::Reg(Register8::H)),
            0xA5 => Instruction::And(ArithSrc::Reg(Register8::L)),
            0xA6 => Instruction::And(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
            0xA7 => Instruction::And(ArithSrc::Reg(Register8::A)),

            // 0xB_
            0xB0 => Instruction::Or(ArithSrc::Reg(Register8::B)),
            0xB1 => Instruction::Or(ArithSrc::Reg(Register8::C)),
            0xB2 => Instruction::Or(ArithSrc::Reg(Register8::D)),
            0xB3 => Instruction::Or(ArithSrc::Reg(Register8::E)),
            0xB4 => Instruction::Or(ArithSrc::Reg(Register8::H)),
            0xB5 => Instruction::Or(ArithSrc::Reg(Register8::L)),
            0xB6 => Instruction::Or(ArithSrc::Mem(MemLoc::Reg(Register16::HL))),
            0xB7 => Instruction::Or(ArithSrc::Reg(Register8::A)),

            // 0xC_
            0xCB => Instruction::decode_prefixed(mem.read8(pc + 1))?, // Special instruction, maps to another instruction set

            _ => not_yet_implemented!()
        };

        Ok(instr)
    }
}
