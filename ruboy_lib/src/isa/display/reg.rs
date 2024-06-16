use crate::isa::{Reg16, Reg8};

use super::{Case, FormatOpts};

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum DisplayableReg {
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
    pub const fn with_format(&self, fmt: &FormatOpts) -> &'static str {
        match fmt.reg_case {
            Case::Upper => self.as_upper_str(fmt),
            Case::Lower => self.as_lower_str(fmt),
        }
    }
    const fn as_lower_str(&self, fmt: &FormatOpts) -> &'static str {
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

    const fn as_upper_str(&self, fmt: &FormatOpts) -> &'static str {
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
