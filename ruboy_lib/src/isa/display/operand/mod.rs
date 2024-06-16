use optype::DisplayableOperandType;

use crate::isa::{
    ArithSrc, IncDecTarget, Ld16Dst, Ld16Src, Ld8Dst, Ld8Src, MemLoc, PrefArithTarget, Reg16, RsVec,
};

use super::{immediate::DisplayableImmediate, reg::DisplayableReg, Case, FormatOpts};

pub mod optype;

#[derive(Debug, Clone, Copy)]
pub enum MemType {
    None,
    Normal,
    HighMem,
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayableOperand {
    pub memory: MemType,
    pub operand: DisplayableOperandType,
}

impl DisplayableOperand {
    pub fn with_format(&self, fmt: &FormatOpts) -> String {
        let op_fmt = match self.operand {
            DisplayableOperandType::Reg(reg) => reg.with_format(fmt).to_owned(),
            DisplayableOperandType::Imm(imm) => imm.with_format(&fmt.imm_format),
            DisplayableOperandType::SpOffset(imm) => {
                let sp = DisplayableReg::SP.with_format(fmt);
                format!("{} + {}", sp, imm.with_format(&fmt.imm_format))
            }
            DisplayableOperandType::Extension(prefmt) => match fmt.mnemonic_case {
                Case::Upper => prefmt.to_uppercase(),
                Case::Lower => prefmt.to_lowercase(),
            },
        };

        match self.memory {
            MemType::None => op_fmt,
            MemType::Normal => format!("[{}]", op_fmt),
            MemType::HighMem => format!(
                "[{} + {}]",
                DisplayableImmediate::U16(0xFF00).with_format(&fmt.imm_format),
                op_fmt
            ),
        }
    }
}

impl From<DisplayableOperandType> for DisplayableOperand {
    fn from(value: DisplayableOperandType) -> Self {
        Self {
            memory: MemType::None,
            operand: value,
        }
    }
}

impl From<u16> for DisplayableOperand {
    fn from(value: u16) -> Self {
        DisplayableOperand {
            memory: MemType::None,
            operand: value.into(),
        }
    }
}

impl From<MemLoc> for DisplayableOperand {
    fn from(value: MemLoc) -> Self {
        match value {
            MemLoc::HighMemReg(reg) => DisplayableOperand {
                memory: MemType::HighMem,
                operand: reg.into(),
            },
            MemLoc::Reg(reg) => DisplayableOperand {
                memory: MemType::Normal,
                operand: reg.into(),
            },
            MemLoc::HighMemImm(imm) => DisplayableOperand {
                memory: MemType::HighMem,
                operand: imm.into(),
            },
            MemLoc::Imm(imm) => DisplayableOperand {
                memory: MemType::Normal,
                operand: imm.into(),
            },
        }
    }
}

impl From<ArithSrc> for DisplayableOperand {
    fn from(value: ArithSrc) -> Self {
        match value {
            ArithSrc::Reg(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            ArithSrc::Imm(imm) => DisplayableOperand {
                memory: MemType::None,
                operand: imm.into(),
            },
            ArithSrc::Mem(memloc) => memloc.into(),
        }
    }
}

impl From<Reg16> for DisplayableOperand {
    fn from(value: Reg16) -> Self {
        DisplayableOperand {
            memory: MemType::None,
            operand: value.into(),
        }
    }
}

impl From<IncDecTarget> for DisplayableOperand {
    fn from(value: IncDecTarget) -> Self {
        match value {
            IncDecTarget::Reg8(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            IncDecTarget::Reg16(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            IncDecTarget::MemHL => DisplayableOperand {
                memory: MemType::Normal,
                operand: (Reg16::HL).into(),
            },
        }
    }
}

impl From<PrefArithTarget> for DisplayableOperand {
    fn from(value: PrefArithTarget) -> Self {
        match value {
            PrefArithTarget::Reg(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            PrefArithTarget::MemHL => DisplayableOperand {
                memory: MemType::Normal,
                operand: (Reg16::HL).into(),
            },
        }
    }
}

impl From<Ld8Src> for DisplayableOperand {
    fn from(value: Ld8Src) -> Self {
        match value {
            Ld8Src::Reg(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            Ld8Src::Mem(mem) => mem.into(),
            Ld8Src::Imm(imm) => DisplayableOperand {
                memory: MemType::None,
                operand: imm.into(),
            },
        }
    }
}

impl From<Ld8Dst> for DisplayableOperand {
    fn from(value: Ld8Dst) -> Self {
        match value {
            Ld8Dst::Reg(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            Ld8Dst::Mem(mem) => mem.into(),
        }
    }
}

impl From<Ld16Src> for DisplayableOperand {
    fn from(value: Ld16Src) -> Self {
        match value {
            Ld16Src::Reg(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            Ld16Src::Imm(imm) => DisplayableOperand {
                memory: MemType::None,
                operand: imm.into(),
            },
        }
    }
}

impl From<Ld16Dst> for DisplayableOperand {
    fn from(value: Ld16Dst) -> Self {
        match value {
            Ld16Dst::Reg(reg) => DisplayableOperand {
                memory: MemType::None,
                operand: reg.into(),
            },
            Ld16Dst::Mem(mem) => mem.into(),
        }
    }
}

impl From<RsVec> for DisplayableOperand {
    fn from(value: RsVec) -> Self {
        (value as u8).into()
    }
}

impl From<u8> for DisplayableOperand {
    fn from(value: u8) -> Self {
        Self {
            memory: MemType::None,
            operand: DisplayableOperandType::Imm(DisplayableImmediate::U8(value)),
        }
    }
}

impl From<i8> for DisplayableOperand {
    fn from(value: i8) -> Self {
        Self {
            memory: MemType::None,
            operand: DisplayableOperandType::from(value),
        }
    }
}

impl From<&str> for DisplayableOperand {
    fn from(value: &str) -> Self {
        let reg = DisplayableReg::try_from(value).unwrap();

        Self {
            memory: MemType::None,
            operand: DisplayableOperandType::Reg(reg),
        }
    }
}
