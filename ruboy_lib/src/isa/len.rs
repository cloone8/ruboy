use super::{ArithSrc, IncDecTarget, Instruction, Ld16Dst, Ld8Dst, Ld8Src, MemLoc};

impl Instruction {
    /// Returns the length of this [`Instruction`] in bytes.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(self) -> u8 {
        match self {
            Instruction::Nop => 1,
            Instruction::Stop(_) => 2,
            Instruction::Halt => 1,
            Instruction::EI => 1,
            Instruction::DI => 2,
            Instruction::Add(src) => 1 + src.op_size(),
            Instruction::AddCarry(src) => 1 + src.op_size(),
            Instruction::AddHL(_) => 1,
            Instruction::AddSP(_) => 2,
            Instruction::Sub(src) => 1 + src.op_size(),
            Instruction::SubCarry(src) => 1 + src.op_size(),
            Instruction::And(src) => 1 + src.op_size(),
            Instruction::Or(src) => 1 + src.op_size(),
            Instruction::Xor(src) => 1 + src.op_size(),
            Instruction::Cmp(src) => 1 + src.op_size(),
            Instruction::Inc(tgt) => 1 + tgt.op_size(),
            Instruction::Dec(tgt) => 1 + tgt.op_size(),
            Instruction::RotLeftCarry(_) => 2,
            Instruction::RotRightCarry(_) => 2,
            Instruction::RotLeft(_) => 2,
            Instruction::RotRight(_) => 2,
            Instruction::ShiftLeftArith(_) => 2,
            Instruction::ShiftRightArith(_) => 2,
            Instruction::Swap(_) => 2,
            Instruction::ShiftRightLogic(_) => 2,
            Instruction::Bit(_, _) => 2,
            Instruction::Res(_, _) => 2,
            Instruction::Set(_, _) => 2,
            Instruction::Load8(dst, src) => 1 + dst.op_size() + src.op_size(),
            Instruction::Load16(dst, src) => 1 + dst.op_size() + src.op_size(),
            Instruction::LoadAtoHLI => 1,
            Instruction::LoadAtoHLD => 1,
            Instruction::LoadHLItoA => 1,
            Instruction::LoadHLDtoA => 1,
            Instruction::LoadSPi8toHL(_) => 2,
            Instruction::Jump(_) => 3,
            Instruction::JumpRel(_) => 2,
            Instruction::JumpHL => 1,
            Instruction::JumpIf(_, _) => 3,
            Instruction::JumpRelIf(_, _) => 2,
            Instruction::Call(_) => 3,
            Instruction::CallIf(_, _) => 3,
            Instruction::Ret => 1,
            Instruction::Reti => 1,
            Instruction::RetIf(_) => 1,
            Instruction::Pop(_) => 1,
            Instruction::Push(_) => 1,
            Instruction::DecimalAdjust => 1,
            Instruction::ComplementAccumulator => 1,
            Instruction::SetCarryFlag => 1,
            Instruction::ComplementCarry => 1,
            Instruction::Rst(_) => 1,
            Instruction::IllegalInstruction(_) => panic!("Illegal instruction has no length"),
        }
    }
}

impl ArithSrc {
    const fn op_size(&self) -> u8 {
        match self {
            ArithSrc::Reg(_) => 0,
            ArithSrc::Imm(_) => 1,
            ArithSrc::Mem(memloc) => memloc.op_size(),
        }
    }
}

impl MemLoc {
    const fn op_size(&self) -> u8 {
        match self {
            MemLoc::HighMemReg(_) => 0,
            MemLoc::Reg(_) => 0,
            MemLoc::HighMemImm(_) => 1,
            MemLoc::Imm(_) => 2,
        }
    }
}

impl Ld8Src {
    const fn op_size(&self) -> u8 {
        match self {
            Ld8Src::Reg(_) => 0,
            Ld8Src::Mem(memloc) => memloc.op_size(),
            Ld8Src::Imm(_) => 1,
        }
    }
}

impl Ld8Dst {
    const fn op_size(&self) -> u8 {
        match self {
            Ld8Dst::Mem(memloc) => memloc.op_size(),
            Ld8Dst::Reg(_) => 0,
        }
    }
}

impl Ld16Dst {
    const fn op_size(&self) -> u8 {
        match self {
            Ld16Dst::Mem(memloc) => memloc.op_size(),
            Ld16Dst::Reg(_) => 0,
        }
    }
}

impl IncDecTarget {
    const fn op_size(&self) -> u8 {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::{decoder::decode, testutils};

    #[test]
    fn all_legal_have_length() {
        for opcode in testutils::legal_instrs() {
            let result = decode(&opcode.as_slice(), 0x0);

            assert!(result.is_ok(), "Opcode {:?} not decoded!", opcode);
            assert!(
                result.unwrap().len() > 0,
                "Opcode {:x?} has no length!",
                opcode
            );
        }
    }

    #[test]
    fn all_legal_max_length_of_three() {
        for opcode in testutils::legal_instrs() {
            let result = decode(&opcode.as_slice(), 0x0);

            assert!(result.is_ok(), "Opcode {:?} not decoded!", opcode);
            assert!(result.unwrap().len() <= 3, "Opcode {:x?} too long!", opcode);
        }
    }
}
