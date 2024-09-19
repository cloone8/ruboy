use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};
use ruboy_lib::isa::display::{Case, ImmediateFormat, OperandOrder};

#[derive(Parser, Debug)]
#[command(author, about, version)]
pub struct CLIArgs {
    pub file: PathBuf,

    /// The case used for mnemonics, e.g. "ADD" or "add"
    #[arg(value_enum, short, long)]
    pub mnemonic_case: Option<ParsableCase>,

    /// The case used for registers, e.g. "HL" or "hl"
    #[arg(value_enum, short, long)]
    pub register_case: Option<ParsableCase>,

    /// Use signs ("HL+") or letters ("HLI") for HLI/HLD
    #[arg(short = 's', long)]
    pub hlid_signs: Option<bool>,

    /// Which operand to put first, src or dst
    #[arg(value_enum, short, long)]
    pub first_operand: Option<FirstOperand>,

    #[command(flatten)]
    pub immediate_format: ParsableImmediateFormat,

    #[arg(long, default_value_t = false)]
    pub no_print_label: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ParsableCase {
    Upper,
    Lower,
}

impl From<ParsableCase> for Case {
    fn from(value: ParsableCase) -> Self {
        match value {
            ParsableCase::Upper => Case::Upper,
            ParsableCase::Lower => Case::Lower,
        }
    }
}

#[derive(Args, Debug, Clone)]
#[group(id = "immediate_format", multiple = false)]
pub struct ParsableImmediateFormat {
    /// Print immediate values in decimal format (e.g. "5" or "11")
    #[arg(short, long)]
    pub decimal: bool,

    /// Print immediate values in lowercase hex format, with prefix.
    /// For example, prefix "0x" results in "0x5" or "0xb"
    #[arg(short = 'x', long, value_name = "PREFIX", default_missing_value = "$", num_args = 0.., require_equals = true)]
    pub hex_lowercase: Option<String>,

    /// Print immediate values in lowercase hex format, with prefix.
    /// For example, prefix "0x" results in "0x5" or "0xB"
    #[arg(short = 'X', long, value_name = "PREFIX", default_missing_value = "$", num_args = 0.., require_equals = true)]
    pub hex_uppercase: Option<String>,
}

impl TryFrom<ParsableImmediateFormat> for ImmediateFormat {
    type Error = ();

    fn try_from(value: ParsableImmediateFormat) -> Result<Self, Self::Error> {
        assert!(
            (value.decimal as u8)
                + (value.hex_lowercase.is_some() as u8)
                + (value.hex_uppercase.is_some() as u8)
                <= 1
        );

        if value.decimal {
            Ok(Self::Decimal)
        } else if let Some(prefix) = value.hex_lowercase {
            Ok(Self::LowerHex { prefix })
        } else if let Some(prefix) = value.hex_uppercase {
            Ok(Self::UpperHex { prefix })
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FirstOperand {
    Dst,
    Src,
}

impl From<FirstOperand> for OperandOrder {
    fn from(value: FirstOperand) -> Self {
        match value {
            FirstOperand::Dst => OperandOrder::DstFirst,
            FirstOperand::Src => OperandOrder::SrcFirst,
        }
    }
}
