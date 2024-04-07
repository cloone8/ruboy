pub(crate) enum Instruction {
    
}

pub const PREFIX_16_BIT: u8 = 0xCB;


#[derive(Debug)]
pub enum DecodeError {
    Not8Bit,
    Not16Bit,
    NotYetImplemented,
}

impl TryFrom<u8> for Instruction {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == PREFIX_16_BIT {
            return Err(DecodeError::Not8Bit);
        }

        Err(DecodeError::NotYetImplemented)
    }
}

impl TryFrom<u16> for Instruction {
    type Error = DecodeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value.to_be_bytes()[0] != PREFIX_16_BIT {
            return Err(DecodeError::Not16Bit);
        }

        Err(DecodeError::NotYetImplemented)
    }
}
