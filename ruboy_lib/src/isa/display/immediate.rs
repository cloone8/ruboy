use core::fmt::{Display, LowerHex, UpperHex};

use super::ImmediateFormat;

#[derive(Debug, Clone, Copy)]
pub enum DisplayableImmediate {
    U8(u8),
    I8(i8),
    U16(u16),
}

fn format_immediate(fmt: &ImmediateFormat, num: impl Display + LowerHex + UpperHex) -> String {
    match fmt {
        ImmediateFormat::Decimal => format!("{}", num),
        ImmediateFormat::LowerHex { prefix } => format!("{}{:x}", prefix, num),
        ImmediateFormat::UpperHex { prefix } => format!("{}{:X}", prefix, num),
    }
}

impl DisplayableImmediate {
    pub fn with_format(&self, fmt: &ImmediateFormat) -> String {
        match self {
            DisplayableImmediate::U8(x) => format_immediate(fmt, x),
            DisplayableImmediate::I8(x) => {
                let abs = (*x as i16).abs(); // Upcast to prevent overflow
                let abs_fmt = format_immediate(fmt, abs);

                if abs.is_negative() {
                    format!("-{}", abs_fmt)
                } else {
                    abs_fmt
                }
            }
            DisplayableImmediate::U16(x) => format_immediate(fmt, x),
        }
    }
}
