use immediate::DisplayableImmediate;
use operand::{optype::DisplayableOperandType, DisplayableOperand, MemType};

use super::{Bit, Condition, Instruction, PrefArithTarget};

mod immediate;
mod operand;
mod reg;

#[derive(Debug, Clone)]
enum DisplayableOperands {
    None,
    Single(DisplayableOperand),
    Dual {
        src: DisplayableOperand,
        dst: DisplayableOperand,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum OperandOrder {
    DstFirst,
    SrcFirst,
}

#[derive(Debug, Clone, Copy)]
pub enum Case {
    Upper,
    Lower,
}

#[derive(Debug, Clone)]
pub enum ImmediateFormat {
    Decimal,
    LowerHex { prefix: String },
    UpperHex { prefix: String },
}

#[derive(Debug, Clone)]
pub struct FormatOpts {
    pub mnemonic_case: Case,
    pub reg_case: Case,
    pub hlid_as_signs: bool,
    pub imm_format: ImmediateFormat,
    pub operand_order: OperandOrder,
}

impl FormatOpts {
    pub fn rgdbs() -> Self {
        FormatOpts {
            mnemonic_case: Case::Lower,
            reg_case: Case::Lower,
            hlid_as_signs: false,
            imm_format: ImmediateFormat::UpperHex {
                prefix: "$".to_owned(),
            },
            operand_order: OperandOrder::DstFirst,
        }
    }
}

impl Default for FormatOpts {
    fn default() -> Self {
        Self::rgdbs()
    }
}

#[derive(Debug, Clone)]
pub struct DisplayableInstruction {
    mnemonic: &'static str,
    operands: DisplayableOperands,
}

impl DisplayableInstruction {
    const fn from_none(mnemonic: &'static str) -> Self {
        Self {
            mnemonic,
            operands: DisplayableOperands::None,
        }
    }

    const fn from_single(mnemonic: &'static str, operand: DisplayableOperand) -> Self {
        Self {
            mnemonic,
            operands: DisplayableOperands::Single(operand),
        }
    }

    const fn from_dual(
        mnemonic: &'static str,
        src: DisplayableOperand,
        dst: DisplayableOperand,
    ) -> Self {
        Self {
            mnemonic,
            operands: DisplayableOperands::Dual { src, dst },
        }
    }

    pub fn with_format(&self, fmt: &FormatOpts) -> String {
        let fmt_mnemonic = match fmt.mnemonic_case {
            Case::Upper => self.mnemonic.to_uppercase(),
            Case::Lower => self.mnemonic.to_lowercase(),
        };

        match self.operands {
            DisplayableOperands::None => fmt_mnemonic,
            DisplayableOperands::Single(operand) => {
                format!("{} {}", fmt_mnemonic, operand.with_format(fmt))
            }
            DisplayableOperands::Dual { src, dst } => {
                if matches!(fmt.operand_order, OperandOrder::DstFirst)
                    || matches!(dst.operand, DisplayableOperandType::Extension(_))
                {
                    format!(
                        "{} {}, {}",
                        fmt_mnemonic,
                        dst.with_format(fmt),
                        src.with_format(fmt)
                    )
                } else {
                    format!(
                        "{} {}, {}",
                        fmt_mnemonic,
                        src.with_format(fmt),
                        dst.with_format(fmt)
                    )
                }
            }
        }
    }
}

fn to_display_bit(
    bit: Bit,
    mnemonic: &'static str,
    tgt: PrefArithTarget,
) -> DisplayableInstruction {
    match bit {
        Bit::B0 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("0")),
        ),
        Bit::B1 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("1")),
        ),
        Bit::B2 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("2")),
        ),
        Bit::B3 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("3")),
        ),
        Bit::B4 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("4")),
        ),
        Bit::B5 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("5")),
        ),
        Bit::B6 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("6")),
        ),
        Bit::B7 => DisplayableInstruction::from_dual(
            mnemonic,
            DisplayableOperand::from(tgt),
            DisplayableOperand::from(DisplayableOperandType::Extension("7")),
        ),
    }
}

fn to_display_cond(cond: Condition, mnemonic: &'static str) -> DisplayableInstruction {
    DisplayableInstruction::from_single(
        mnemonic,
        DisplayableOperand::from(DisplayableOperandType::from(cond)),
    )
}

fn to_display_cond_with_tgt(
    cond: Condition,
    mnemonic: &'static str,
    tgt: impl Into<DisplayableOperand>,
) -> DisplayableInstruction {
    DisplayableInstruction::from_dual(
        mnemonic,
        tgt.into(),
        DisplayableOperand::from(DisplayableOperandType::from(cond)),
    )
}

impl From<Instruction> for DisplayableInstruction {
    fn from(value: Instruction) -> Self {
        match value {
            Instruction::Nop => DisplayableInstruction::from_none("nop"),
            Instruction::Stop(code) => {
                DisplayableInstruction::from_single("stop", DisplayableOperand::from(code))
            }
            Instruction::Halt => DisplayableInstruction::from_none("halt"),
            Instruction::EI => DisplayableInstruction::from_none("ei"),
            Instruction::DI => DisplayableInstruction::from_none("di"),
            Instruction::Add(src) => DisplayableInstruction::from_dual(
                "add",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::AddCarry(src) => DisplayableInstruction::from_dual(
                "adc",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::AddHL(src) => DisplayableInstruction::from_dual(
                "add",
                DisplayableOperand::from(src),
                DisplayableOperand::from("hl"),
            ),
            Instruction::AddSP(src) => DisplayableInstruction::from_dual(
                "add",
                DisplayableOperand::from(src),
                DisplayableOperand::from("sp"),
            ),
            Instruction::Sub(src) => DisplayableInstruction::from_dual(
                "sub",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::SubCarry(src) => DisplayableInstruction::from_dual(
                "sbc",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::And(src) => DisplayableInstruction::from_dual(
                "and",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::Or(src) => DisplayableInstruction::from_dual(
                "or",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::Xor(src) => DisplayableInstruction::from_dual(
                "xor",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::Cmp(src) => DisplayableInstruction::from_dual(
                "cmp",
                DisplayableOperand::from(src),
                DisplayableOperand::from("a"),
            ),
            Instruction::Inc(tgt) => {
                DisplayableInstruction::from_single("inc", DisplayableOperand::from(tgt))
            }
            Instruction::Dec(tgt) => {
                DisplayableInstruction::from_single("dec", DisplayableOperand::from(tgt))
            }
            Instruction::RotLeftCircular(tgt) => {
                DisplayableInstruction::from_single("rlc", DisplayableOperand::from(tgt))
            }
            Instruction::RotRightCircular(tgt) => {
                DisplayableInstruction::from_single("rrc", DisplayableOperand::from(tgt))
            }
            Instruction::RotLeft(tgt) => {
                DisplayableInstruction::from_single("rl", DisplayableOperand::from(tgt))
            }
            Instruction::RotRight(tgt) => {
                DisplayableInstruction::from_single("rr", DisplayableOperand::from(tgt))
            }
            Instruction::ShiftLeftArith(tgt) => {
                DisplayableInstruction::from_single("sla", DisplayableOperand::from(tgt))
            }
            Instruction::ShiftRightArith(tgt) => {
                DisplayableInstruction::from_single("sra", DisplayableOperand::from(tgt))
            }
            Instruction::Swap(tgt) => {
                DisplayableInstruction::from_single("swap", DisplayableOperand::from(tgt))
            }
            Instruction::ShiftRightLogic(tgt) => {
                DisplayableInstruction::from_single("srl", DisplayableOperand::from(tgt))
            }
            Instruction::Bit(bit, tgt) => to_display_bit(bit, "bit", tgt),
            Instruction::Res(bit, tgt) => to_display_bit(bit, "res", tgt),
            Instruction::Set(bit, tgt) => to_display_bit(bit, "set", tgt),
            Instruction::Load8(dst, src) => DisplayableInstruction::from_dual(
                "ld",
                DisplayableOperand::from(src),
                DisplayableOperand::from(dst),
            ),
            Instruction::Load16(dst, src) => DisplayableInstruction::from_dual(
                "ld",
                DisplayableOperand::from(src),
                DisplayableOperand::from(dst),
            ),
            Instruction::LoadAtoHLI => DisplayableInstruction::from_dual(
                "ld",
                DisplayableOperand::from("a"),
                DisplayableOperand::from("hli"),
            ),
            Instruction::LoadAtoHLD => DisplayableInstruction::from_dual(
                "ld",
                DisplayableOperand::from("a"),
                DisplayableOperand::from("hld"),
            ),
            Instruction::LoadHLItoA => DisplayableInstruction::from_dual(
                "ld",
                DisplayableOperand::from("hli"),
                DisplayableOperand::from("a"),
            ),
            Instruction::LoadHLDtoA => DisplayableInstruction::from_dual(
                "ld",
                DisplayableOperand::from("hld"),
                DisplayableOperand::from("a"),
            ),
            Instruction::LoadSPi8toHL(offset) => DisplayableInstruction::from_dual(
                "ld",
                DisplayableOperand::from("hl"),
                DisplayableOperand {
                    memory: MemType::None,
                    operand: DisplayableOperandType::SpOffset(DisplayableImmediate::I8(offset)),
                },
            ),
            Instruction::Jump(tgt) => {
                DisplayableInstruction::from_single("jp", DisplayableOperand::from(tgt))
            }
            Instruction::JumpRel(tgt) => {
                DisplayableInstruction::from_single("jr", DisplayableOperand::from(tgt))
            }
            Instruction::JumpHL => {
                DisplayableInstruction::from_single("jp", DisplayableOperand::from("hl"))
            }
            Instruction::JumpIf(tgt, cond) => to_display_cond_with_tgt(cond, "jp", tgt),
            Instruction::JumpRelIf(tgt, cond) => to_display_cond_with_tgt(cond, "jr", tgt),
            Instruction::Call(tgt) => {
                DisplayableInstruction::from_single("call", DisplayableOperand::from(tgt))
            }
            Instruction::CallIf(tgt, cond) => to_display_cond_with_tgt(cond, "call", tgt),
            Instruction::Ret => DisplayableInstruction::from_none("ret"),
            Instruction::Reti => DisplayableInstruction::from_none("reti"),
            Instruction::RetIf(cond) => to_display_cond(cond, "ret"),
            Instruction::Pop(tgt) => {
                DisplayableInstruction::from_single("pop", DisplayableOperand::from(tgt))
            }
            Instruction::Push(src) => {
                DisplayableInstruction::from_single("push", DisplayableOperand::from(src))
            }
            Instruction::DecimalAdjust => DisplayableInstruction::from_none("daa"),
            Instruction::ComplementAccumulator => DisplayableInstruction::from_none("cpl"),
            Instruction::SetCarryFlag => DisplayableInstruction::from_none("scf"),
            Instruction::ComplementCarry => DisplayableInstruction::from_none("ccf"),
            Instruction::Rst(tgt) => {
                DisplayableInstruction::from_single("rst", DisplayableOperand::from(tgt))
            }
            Instruction::RotLeftCircularA => DisplayableInstruction::from_none("rlca"),
            Instruction::RotRightCircularA => DisplayableInstruction::from_none("rrca"),
            Instruction::RotLeftA => DisplayableInstruction::from_none("rla"),
            Instruction::RotRightA => DisplayableInstruction::from_none("rra"),
            Instruction::IllegalInstruction(mnemonic) => {
                DisplayableInstruction::from_single("???", DisplayableOperand::from(mnemonic))
            }
        }
    }
}
