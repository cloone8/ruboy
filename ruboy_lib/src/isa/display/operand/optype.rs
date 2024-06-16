use crate::isa::{
    display::{immediate::DisplayableImmediate, reg::DisplayableReg},
    Condition, Reg16, Reg8,
};

#[derive(Debug, Clone, Copy)]
pub enum DisplayableOperandType {
    Reg(DisplayableReg),
    Imm(DisplayableImmediate),
    SpOffset(DisplayableImmediate),
    Extension(&'static str),
}

impl From<u8> for DisplayableOperandType {
    fn from(value: u8) -> Self {
        Self::Imm(DisplayableImmediate::U8(value))
    }
}
impl From<u16> for DisplayableOperandType {
    fn from(value: u16) -> Self {
        Self::Imm(DisplayableImmediate::U16(value))
    }
}
impl From<i8> for DisplayableOperandType {
    fn from(value: i8) -> Self {
        Self::Imm(DisplayableImmediate::I8(value))
    }
}

impl From<Condition> for DisplayableOperandType {
    fn from(value: Condition) -> Self {
        match value {
            Condition::Zero => DisplayableOperandType::Extension("z"),
            Condition::NotZero => DisplayableOperandType::Extension("nz"),
            Condition::Carry => DisplayableOperandType::Extension("c"),
            Condition::NotCarry => DisplayableOperandType::Extension("nc"),
        }
    }
}

impl From<Reg8> for DisplayableOperandType {
    fn from(value: Reg8) -> Self {
        DisplayableOperandType::Reg(DisplayableReg::from(value))
    }
}

impl From<Reg16> for DisplayableOperandType {
    fn from(value: Reg16) -> Self {
        DisplayableOperandType::Reg(DisplayableReg::from(value))
    }
}
