use super::{
    ArithSrc, Condition, IncDecTarget, Instruction, Ld16Dst, Ld16Src, Ld8Dst, Ld8Src, MemLoc,
    PrefArithTarget, Reg16, Reg8, RsVec,
};

#[derive(Debug, Clone, Copy)]
enum DisplayableReg {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    HLD,
    HLI,
}

impl From<Reg8> for DisplayableReg {
    fn from(value: Reg8) -> Self {
        match value {
            Reg8::A => DisplayableReg::A,
            Reg8::B => DisplayableReg::B,
            Reg8::C => DisplayableReg::C,
            Reg8::D => DisplayableReg::D,
            Reg8::E => DisplayableReg::E,
            Reg8::F => DisplayableReg::F,
            Reg8::H => DisplayableReg::H,
            Reg8::L => DisplayableReg::L,
        }
    }
}

impl From<Reg16> for DisplayableReg {
    fn from(value: Reg16) -> Self {
        match value {
            Reg16::AF => DisplayableReg::AF,
            Reg16::BC => DisplayableReg::BC,
            Reg16::DE => DisplayableReg::DE,
            Reg16::HL => DisplayableReg::HL,
            Reg16::SP => DisplayableReg::SP,
        }
    }
}

impl TryFrom<&str> for DisplayableReg {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let reg = match value {
            "a" => DisplayableReg::A,
            "b" => DisplayableReg::B,
            "c" => DisplayableReg::C,
            "d" => DisplayableReg::D,
            "e" => DisplayableReg::E,
            "f" => DisplayableReg::F,
            "h" => DisplayableReg::H,
            "l" => DisplayableReg::L,
            "af" => DisplayableReg::AF,
            "bc" => DisplayableReg::BC,
            "de" => DisplayableReg::DE,
            "hl" => DisplayableReg::HL,
            "sp" => DisplayableReg::SP,
            "hld" => DisplayableReg::HLD,
            "hli" => DisplayableReg::HLI,
            _ => return Err(()),
        };

        Ok(reg)
    }
}

impl DisplayableReg {
    const fn with_format(&self, fmt: FormatOpts) -> &'static str {
        match fmt.reg_case {
            Case::Upper => self.as_upper_str(fmt),
            Case::Lower => self.as_lower_str(fmt),
        }
    }
    const fn as_lower_str(&self, fmt: FormatOpts) -> &'static str {
        match self {
            DisplayableReg::A => "a",
            DisplayableReg::B => "b",
            DisplayableReg::C => "c",
            DisplayableReg::D => "d",
            DisplayableReg::E => "e",
            DisplayableReg::F => "f",
            DisplayableReg::H => "h",
            DisplayableReg::L => "l",
            DisplayableReg::AF => "af",
            DisplayableReg::BC => "bc",
            DisplayableReg::DE => "de",
            DisplayableReg::HL => "hl",
            DisplayableReg::SP => "sp",
            DisplayableReg::HLD => match fmt.hlid_as_signs {
                true => "hl-",
                false => "hld",
            },
            DisplayableReg::HLI => match fmt.hlid_as_signs {
                true => "hl+",
                false => "hli",
            },
        }
    }

    const fn as_upper_str(&self, fmt: FormatOpts) -> &'static str {
        match self {
            DisplayableReg::A => "A",
            DisplayableReg::B => "B",
            DisplayableReg::C => "C",
            DisplayableReg::D => "D",
            DisplayableReg::E => "E",
            DisplayableReg::F => "F",
            DisplayableReg::H => "H",
            DisplayableReg::L => "L",
            DisplayableReg::AF => "AF",
            DisplayableReg::BC => "BC",
            DisplayableReg::DE => "DE",
            DisplayableReg::HL => "HL",
            DisplayableReg::SP => "SP",
            DisplayableReg::HLD => match fmt.hlid_as_signs {
                true => "HL-",
                false => "HLD",
            },
            DisplayableReg::HLI => match fmt.hlid_as_signs {
                true => "HL+",
                false => "HLI",
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DisplayableImmediate {
    U8(u8),
    I8(i8),
    U16(u16),
}

macro_rules! format_immediate {
    ($fmt:expr, $num:expr) => {
        match $fmt {
            ImmediateFormat::Decimal => format!("{}", $num),
            ImmediateFormat::LowerHex { prefix } => format!("{}{:x}", prefix, $num),
            ImmediateFormat::UpperHex { prefix } => format!("{}{:X}", prefix, $num),
        }
    };
}

impl DisplayableImmediate {
    fn with_format(&self, fmt: ImmediateFormat) -> String {
        match self {
            DisplayableImmediate::U8(x) => format_immediate!(fmt, x),
            DisplayableImmediate::I8(x) => {
                let abs = (*x as i16).abs(); // Upcast to prevent overflow
                let abs_fmt = format_immediate!(fmt, abs);

                if abs.is_negative() {
                    format!("-{}", abs_fmt)
                } else {
                    abs_fmt
                }
            }
            DisplayableImmediate::U16(x) => format_immediate!(fmt, x),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DisplayableOperandType {
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

#[derive(Debug, Clone, Copy)]
enum MemType {
    None,
    Normal,
    HighMem,
}

#[derive(Debug, Clone, Copy)]
struct DisplayableOperand {
    memory: MemType,
    operand: DisplayableOperandType,
}

impl DisplayableOperand {
    fn with_format(&self, fmt: FormatOpts) -> String {
        let op_fmt = match self.operand {
            DisplayableOperandType::Reg(reg) => reg.with_format(fmt).to_owned(),
            DisplayableOperandType::Imm(imm) => imm.with_format(fmt.imm_format),
            DisplayableOperandType::SpOffset(imm) => {
                let sp = DisplayableReg::SP.with_format(fmt);
                format!("{} + {}", sp, imm.with_format(fmt.imm_format))
            }
            DisplayableOperandType::Extension(prefmt) => match fmt.opcode_case {
                Case::Upper => prefmt.to_uppercase(),
                Case::Lower => prefmt.to_lowercase(),
            },
        };

        match self.memory {
            MemType::None => op_fmt,
            MemType::Normal => format!("[{}]", op_fmt),
            MemType::HighMem => format!(
                "[{} + {}]",
                DisplayableImmediate::U16(0xFF00).with_format(fmt.imm_format),
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

#[derive(Debug, Clone, Copy)]
pub enum ImmediateFormat {
    Decimal,
    LowerHex { prefix: &'static str },
    UpperHex { prefix: &'static str },
}

#[derive(Debug, Clone, Copy)]
pub struct FormatOpts {
    pub opcode_case: Case,
    pub reg_case: Case,
    pub hlid_as_signs: bool,
    pub imm_format: ImmediateFormat,
    pub operand_order: OperandOrder,
}

impl FormatOpts {
    pub fn rgdbs() -> Self {
        FormatOpts {
            opcode_case: Case::Lower,
            reg_case: Case::Lower,
            hlid_as_signs: false,
            imm_format: ImmediateFormat::UpperHex { prefix: "$" },
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
    opcode: &'static str,
    operands: DisplayableOperands,
}

impl DisplayableInstruction {
    const fn from_none(opcode: &'static str) -> Self {
        Self {
            opcode,
            operands: DisplayableOperands::None,
        }
    }

    const fn from_single(opcode: &'static str, operand: DisplayableOperand) -> Self {
        Self {
            opcode,
            operands: DisplayableOperands::Single(operand),
        }
    }

    const fn from_dual(
        opcode: &'static str,
        src: DisplayableOperand,
        dst: DisplayableOperand,
    ) -> Self {
        Self {
            opcode,
            operands: DisplayableOperands::Dual { src, dst },
        }
    }

    pub fn with_format(&self, fmt: FormatOpts) -> String {
        let fmt_opcode = match fmt.opcode_case {
            Case::Upper => self.opcode.to_uppercase(),
            Case::Lower => self.opcode.to_lowercase(),
        };

        match self.operands {
            DisplayableOperands::None => fmt_opcode,
            DisplayableOperands::Single(operand) => {
                format!("{} {}", fmt_opcode, operand.with_format(fmt))
            }
            DisplayableOperands::Dual { src, dst } => {
                if matches!(fmt.operand_order, OperandOrder::DstFirst)
                    || matches!(dst.operand, DisplayableOperandType::Extension(_))
                {
                    format!(
                        "{} {}, {}",
                        fmt_opcode,
                        dst.with_format(fmt),
                        src.with_format(fmt)
                    )
                } else {
                    format!(
                        "{} {}, {}",
                        fmt_opcode,
                        src.with_format(fmt),
                        dst.with_format(fmt)
                    )
                }
            }
        }
    }
}

macro_rules! to_display {
    ($opcode:expr) => {
        DisplayableInstruction::from_none($opcode)
    };

    ($opcode:expr, $operand:expr) => {
        DisplayableInstruction::from_single($opcode, DisplayableOperand::from($operand))
    };

    ($opcode:expr, $src:expr, $dst:expr) => {
        DisplayableInstruction::from_dual(
            $opcode,
            DisplayableOperand::from($src),
            DisplayableOperand::from($dst),
        )
    };
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

macro_rules! with_memtype {
    ($x:expr, $memtype:ident) => {
        DisplayableOperand {
            memory: MemType::$memtype,
            operand: $x.into(),
        }
    };
}

impl From<u16> for DisplayableOperand {
    fn from(value: u16) -> Self {
        with_memtype!(value, None)
    }
}

impl From<MemLoc> for DisplayableOperand {
    fn from(value: MemLoc) -> Self {
        match value {
            MemLoc::HighMemReg(reg) => with_memtype!(reg, HighMem),
            MemLoc::Reg(reg) => with_memtype!(reg, Normal),
            MemLoc::HighMemImm(imm) => with_memtype!(imm, HighMem),
            MemLoc::Imm(imm) => with_memtype!(imm, Normal),
        }
    }
}

impl From<ArithSrc> for DisplayableOperand {
    fn from(value: ArithSrc) -> Self {
        match value {
            ArithSrc::Reg(reg) => with_memtype!(reg, None),
            ArithSrc::Imm(imm) => with_memtype!(imm, None),
            ArithSrc::Mem(memloc) => memloc.into(),
        }
    }
}

impl From<Reg16> for DisplayableOperand {
    fn from(value: Reg16) -> Self {
        with_memtype!(value, None)
    }
}

impl From<IncDecTarget> for DisplayableOperand {
    fn from(value: IncDecTarget) -> Self {
        match value {
            IncDecTarget::Reg8(reg) => with_memtype!(reg, None),
            IncDecTarget::Reg16(reg) => with_memtype!(reg, None),
            IncDecTarget::MemHL => with_memtype!(Reg16::HL, Normal),
        }
    }
}

impl From<PrefArithTarget> for DisplayableOperand {
    fn from(value: PrefArithTarget) -> Self {
        match value {
            PrefArithTarget::Reg(reg) => with_memtype!(reg, None),
            PrefArithTarget::MemHL => with_memtype!(Reg16::HL, Normal),
        }
    }
}

impl From<Ld8Src> for DisplayableOperand {
    fn from(value: Ld8Src) -> Self {
        match value {
            Ld8Src::Reg(reg) => with_memtype!(reg, None),
            Ld8Src::Mem(mem) => mem.into(),
            Ld8Src::Imm(imm) => with_memtype!(imm, None),
        }
    }
}

impl From<Ld8Dst> for DisplayableOperand {
    fn from(value: Ld8Dst) -> Self {
        match value {
            Ld8Dst::Reg(reg) => with_memtype!(reg, None),
            Ld8Dst::Mem(mem) => mem.into(),
        }
    }
}

impl From<Ld16Src> for DisplayableOperand {
    fn from(value: Ld16Src) -> Self {
        match value {
            Ld16Src::Reg(reg) => with_memtype!(reg, None),
            Ld16Src::Imm(imm) => with_memtype!(imm, None),
        }
    }
}

impl From<Ld16Dst> for DisplayableOperand {
    fn from(value: Ld16Dst) -> Self {
        match value {
            Ld16Dst::Reg(reg) => with_memtype!(reg, None),
            Ld16Dst::Mem(mem) => mem.into(),
        }
    }
}

impl From<RsVec> for DisplayableOperand {
    fn from(value: RsVec) -> Self {
        (value as u8).into()
    }
}

macro_rules! to_display_bit {
    ($bit:expr, $opcode:expr, $tgt:expr) => {
        match $bit {
            super::Bit::B0 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("0")),
            super::Bit::B1 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("1")),
            super::Bit::B2 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("2")),
            super::Bit::B3 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("3")),
            super::Bit::B4 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("4")),
            super::Bit::B5 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("5")),
            super::Bit::B6 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("6")),
            super::Bit::B7 => to_display!($opcode, $tgt, DisplayableOperandType::Extension("7")),
        }
    };
}

macro_rules! to_display_cond {
    ($cond:expr, $opcode:expr, $tgt:expr) => {
        match $cond {
            Condition::Zero => to_display!($opcode, $tgt, DisplayableOperandType::Extension("z")),
            Condition::NotZero => {
                to_display!($opcode, $tgt, DisplayableOperandType::Extension("nz"))
            }
            Condition::Carry => to_display!($opcode, $tgt, DisplayableOperandType::Extension("c")),
            Condition::NotCarry => {
                to_display!($opcode, $tgt, DisplayableOperandType::Extension("nc"))
            }
        }
    };

    ($cond:expr, $opcode:expr) => {
        match $cond {
            Condition::Zero => to_display!($opcode, DisplayableOperandType::Extension("z")),
            Condition::NotZero => {
                to_display!($opcode, DisplayableOperandType::Extension("nz"))
            }
            Condition::Carry => to_display!($opcode, DisplayableOperandType::Extension("c")),
            Condition::NotCarry => {
                to_display!($opcode, DisplayableOperandType::Extension("nc"))
            }
        }
    };
}

impl From<Instruction> for DisplayableInstruction {
    fn from(value: Instruction) -> Self {
        match value {
            Instruction::Nop => to_display!("nop"),
            Instruction::Stop(code) => to_display!("stop", code),
            Instruction::Halt => to_display!("halt"),
            Instruction::EI => to_display!("ei"),
            Instruction::DI => to_display!("di"),
            Instruction::Add(src) => to_display!("add", src, "a"),
            Instruction::AddCarry(src) => to_display!("adc", src, "a"),
            Instruction::AddHL(src) => to_display!("add", src, "hl"),
            Instruction::AddSP(src) => to_display!("add", src, "sp"),
            Instruction::Sub(src) => to_display!("sub", src, "a"),
            Instruction::SubCarry(src) => to_display!("sbc", src, "a"),
            Instruction::And(src) => to_display!("and", src, "a"),
            Instruction::Or(src) => to_display!("or", src, "a"),
            Instruction::Xor(src) => to_display!("xor", src, "a"),
            Instruction::Cmp(src) => to_display!("cmp", src, "a"),
            Instruction::Inc(tgt) => to_display!("inc", tgt),
            Instruction::Dec(tgt) => to_display!("dec", tgt),
            Instruction::RotLeftCarry(tgt) => to_display!("rlc", tgt),
            Instruction::RotRightCarry(tgt) => to_display!("rrc", tgt),
            Instruction::RotLeft(tgt) => to_display!("rl", tgt),
            Instruction::RotRight(tgt) => to_display!("rr", tgt),
            Instruction::ShiftLeftArith(tgt) => to_display!("sla", tgt),
            Instruction::ShiftRightArith(tgt) => to_display!("sra", tgt),
            Instruction::Swap(tgt) => to_display!("swap", tgt),
            Instruction::ShiftRightLogic(tgt) => to_display!("srl", tgt),
            Instruction::Bit(bit, tgt) => to_display_bit!(bit, "bit", tgt),
            Instruction::Res(bit, tgt) => to_display_bit!(bit, "res", tgt),
            Instruction::Set(bit, tgt) => to_display_bit!(bit, "set", tgt),
            Instruction::Load8(dst, src) => to_display!("ld", src, dst),
            Instruction::Load16(dst, src) => to_display!("ld", src, dst),
            Instruction::LoadAtoHLI => to_display!("ld", "a", "hli"),
            Instruction::LoadAtoHLD => to_display!("ld", "a", "hld"),
            Instruction::LoadHLItoA => to_display!("ld", "hli", "a"),
            Instruction::LoadHLDtoA => to_display!("ld", "hld", "a"),
            Instruction::LoadSPi8toHL(offset) => to_display!(
                "ld",
                "hl",
                DisplayableOperand {
                    memory: MemType::None,
                    operand: DisplayableOperandType::SpOffset(DisplayableImmediate::I8(offset))
                }
            ),
            Instruction::Jump(tgt) => to_display!("jp", tgt),
            Instruction::JumpRel(tgt) => to_display!("jr", tgt),
            Instruction::JumpHL => to_display!("jp", "hl"),
            Instruction::JumpIf(tgt, cond) => to_display_cond!(cond, "jp", tgt),
            Instruction::JumpRelIf(tgt, cond) => to_display_cond!(cond, "jr", tgt),
            Instruction::Call(tgt) => to_display!("call", tgt),
            Instruction::CallIf(tgt, cond) => to_display_cond!(cond, "call", tgt),
            Instruction::Ret => to_display!("ret"),
            Instruction::Reti => to_display!("reti"),
            Instruction::RetIf(cond) => to_display_cond!(cond, "ret"),
            Instruction::Pop(tgt) => to_display!("pop", tgt),
            Instruction::Push(src) => to_display!("push", src),
            Instruction::DecimalAdjust => to_display!("daa"),
            Instruction::ComplementAccumulator => to_display!("cpl"),
            Instruction::SetCarryFlag => to_display!("scf"),
            Instruction::ComplementCarry => to_display!("ccf"),
            Instruction::Rst(tgt) => to_display!("rst", tgt),
            Instruction::IllegalInstruction(opcode) => to_display!("???", opcode),
        }
    }
}
