use super::{
    ArithSrc, IncDecTarget, Instruction, Ld16Dst, Ld16Src, Ld8Dst, Ld8Src, MemLoc, PrefArithTarget,
    Reg8,
};

#[derive(Debug, Clone, Copy)]
pub enum TCycles {
    Static(u8),
    Branching { taken: u8, non_taken: u8 },
}

macro_rules! cycles {
    ($single:literal) => {
        TCycles::Static($single)
    };

    ($taken:literal or $nontaken:literal) => {
        TCycles::Branching {
            taken: $taken,
            non_taken: $nontaken,
        }
    };
}

const fn arith_cycles(src: ArithSrc) -> TCycles {
    match src {
        ArithSrc::Reg(_) => cycles!(4),
        ArithSrc::Imm(_) => cycles!(8),
        ArithSrc::Mem(_) => cycles!(8),
    }
}

const fn pref_arith_short(tgt: PrefArithTarget) -> TCycles {
    match tgt {
        PrefArithTarget::Reg(_) => cycles!(8),
        PrefArithTarget::MemHL => cycles!(12),
    }
}

const fn pref_arith_long(tgt: PrefArithTarget) -> TCycles {
    match tgt {
        PrefArithTarget::Reg(_) => cycles!(8),
        PrefArithTarget::MemHL => cycles!(16),
    }
}

const fn rot(tgt: PrefArithTarget) -> TCycles {
    match tgt {
        PrefArithTarget::Reg(Reg8::A) => cycles!(4),
        _ => pref_arith_long(tgt),
    }
}

impl Instruction {
    pub const fn cycles(self) -> TCycles {
        match self {
            Instruction::Nop => cycles!(4),
            Instruction::Stop(_) => cycles!(4),
            Instruction::Halt => cycles!(4),
            Instruction::EI => cycles!(4),
            Instruction::DI => cycles!(4),
            Instruction::Add(src) => arith_cycles(src),
            Instruction::AddCarry(src) => arith_cycles(src),
            Instruction::AddHL(_) => cycles!(8),
            Instruction::AddSP(_) => cycles!(16),
            Instruction::Sub(src) => arith_cycles(src),
            Instruction::SubCarry(src) => arith_cycles(src),
            Instruction::And(src) => arith_cycles(src),
            Instruction::Or(src) => arith_cycles(src),
            Instruction::Xor(src) => arith_cycles(src),
            Instruction::Cmp(src) => arith_cycles(src),
            Instruction::Inc(tgt) => match tgt {
                IncDecTarget::Reg8(_) => cycles!(4),
                IncDecTarget::Reg16(_) => cycles!(8),
                IncDecTarget::MemHL => cycles!(12),
            },
            Instruction::Dec(tgt) => match tgt {
                IncDecTarget::Reg8(_) => cycles!(4),
                IncDecTarget::Reg16(_) => cycles!(8),
                IncDecTarget::MemHL => cycles!(12),
            },
            Instruction::RotLeftCarry(tgt) => pref_arith_long(tgt),
            Instruction::RotRightCarry(tgt) => pref_arith_long(tgt),
            Instruction::RotLeft(tgt) => pref_arith_long(tgt),
            Instruction::RotRight(tgt) => pref_arith_long(tgt),
            Instruction::ShiftLeftArith(tgt) => pref_arith_long(tgt),
            Instruction::ShiftRightArith(tgt) => pref_arith_long(tgt),
            Instruction::Swap(tgt) => pref_arith_long(tgt),
            Instruction::ShiftRightLogic(tgt) => pref_arith_long(tgt),
            Instruction::Bit(_, tgt) => pref_arith_short(tgt),
            Instruction::Res(_, tgt) => pref_arith_long(tgt),
            Instruction::Set(_, tgt) => pref_arith_long(tgt),
            Instruction::Load8(dst, src) => {
                if matches!(src, Ld8Src::Mem(MemLoc::Imm(_)))
                    || matches!(dst, Ld8Dst::Mem(MemLoc::Imm(_)))
                {
                    cycles!(16)
                } else if matches!(dst, Ld8Dst::Reg(_)) {
                    if matches!(src, Ld8Src::Reg(_)) {
                        cycles!(4)
                    } else {
                        cycles!(8)
                    }
                } else if matches!(src, Ld8Src::Imm(_)) {
                    // Only LD [HL], n8 has length 12
                    cycles!(12)
                } else {
                    cycles!(8)
                }
            }
            Instruction::Load16(dst, src) => {
                if matches!(dst, Ld16Dst::Mem(MemLoc::Imm(_))) {
                    cycles!(20)
                } else {
                    match src {
                        Ld16Src::Reg(_) => cycles!(8),
                        Ld16Src::Imm(_) => cycles!(12),
                    }
                }
            }
            Instruction::LoadAtoHLI => cycles!(8),
            Instruction::LoadAtoHLD => cycles!(8),
            Instruction::LoadHLItoA => cycles!(8),
            Instruction::LoadHLDtoA => cycles!(8),
            Instruction::LoadSPi8toHL(_) => cycles!(12),
            Instruction::Jump(_) => cycles!(16),
            Instruction::JumpRel(_) => cycles!(12),
            Instruction::JumpHL => cycles!(4),
            Instruction::JumpIf(_, _) => cycles!(16 or 12),
            Instruction::JumpRelIf(_, _) => cycles!(12 or 8),
            Instruction::Call(_) => cycles!(24),
            Instruction::CallIf(_, _) => cycles!(24 or 12),
            Instruction::Ret => cycles!(16),
            Instruction::Reti => cycles!(16),
            Instruction::RetIf(_) => cycles!(20 or 8),
            Instruction::Pop(_) => cycles!(12),
            Instruction::Push(_) => cycles!(16),
            Instruction::DecimalAdjust => cycles!(4),
            Instruction::ComplementAccumulator => cycles!(4),
            Instruction::SetCarryFlag => cycles!(4),
            Instruction::ComplementCarry => cycles!(4),
            Instruction::Rst(_) => cycles!(16),
            Instruction::RotLeftCarryA => cycles!(4),
            Instruction::RotRightCarryA => cycles!(4),
            Instruction::RotLeftA => cycles!(4),
            Instruction::RotRightA => cycles!(4),
            Instruction::IllegalInstruction(_) => panic!("Illegal instruction has no cycle count"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::{decoder::decode, testutils};

    #[test]
    fn all_legal_have_cycles() {
        for opcode in testutils::legal_instrs() {
            let result = decode(&opcode.as_slice(), 0x0);

            assert!(result.is_ok(), "Opcode {:?} not decoded!", opcode);

            match result.unwrap().cycles() {
                crate::isa::TCycles::Static(cycles) => {
                    assert!(cycles > 0, "Opcode {:?} has no cycles!", opcode)
                }
                crate::isa::TCycles::Branching { taken, non_taken } => {
                    assert!(taken > 0, "Opcode {:?} has no taken cycles", opcode);
                    assert!(non_taken > 0, "Opcode {:?} has no non-taken cycles", opcode);
                }
            }
        }
    }

    #[test]
    fn all_cycles_are_divisible() {
        for opcode in testutils::legal_instrs() {
            let result = decode(&opcode.as_slice(), 0x0);

            assert!(result.is_ok(), "Opcode {:?} not decoded!", opcode);

            match result.unwrap().cycles() {
                crate::isa::TCycles::Static(cycles) => {
                    assert_eq!(
                        0,
                        cycles % 4,
                        "Opcode {:?} has a cycle length that is not divisible by 4!",
                        opcode
                    );
                }
                crate::isa::TCycles::Branching { taken, non_taken } => {
                    assert_eq!(
                        0,
                        taken % 4,
                        "Opcode {:?} has a taken cycle length that is not divisible by 4!",
                        opcode
                    );
                    assert_eq!(
                        0,
                        non_taken % 4,
                        "Opcode {:?} has a non-taken cycle length that is not divisible by 4!",
                        opcode
                    );
                }
            }
        }
    }
}
