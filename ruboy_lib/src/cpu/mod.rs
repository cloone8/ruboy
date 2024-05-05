mod registers;

use thiserror::Error;

use registers::Registers;

use crate::{
    isa::{self, decoder::{self, DecodeError}, Instruction},
    memcontroller::MemController,
    GBRam,
    Gameboy
};

pub struct Cpu {
    registers: Registers,
}

#[derive(Debug, Error)]
pub enum InstructionExecutionError {

    #[error("Error during instruction decoding: {0}")]
    Decode(#[from] DecodeError),

    #[error("Illegal instruction: {0}")]
    Illegal(u8)
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::new()
        }
    }

    pub fn run_instruction(&mut self, mem: &mut MemController<impl GBRam>) -> Result<(), InstructionExecutionError> {
        log::trace!("Running instruction at {:x}", self.registers.pc());

        let instr = decoder::decode(mem, self.registers.pc())?;

        log::trace!("Decoded instruction: {:?}", instr);

        match instr {
            Instruction::Nop => {},
            Instruction::Stop => todo!("{:?}", instr),
            Instruction::Halt => todo!("{:?}", instr),
            Instruction::EI => todo!("{:?}", instr),
            Instruction::DI => todo!("{:?}", instr),
            Instruction::Add(_) => todo!("{:?}", instr),
            Instruction::AddCarry(_) => todo!("{:?}", instr),
            Instruction::AddHL(_) => todo!("{:?}", instr),
            Instruction::AddSP(_) => todo!("{:?}", instr),
            Instruction::Sub(_) => todo!("{:?}", instr),
            Instruction::SubCarry(_) => todo!("{:?}", instr),
            Instruction::And(_) => todo!("{:?}", instr),
            Instruction::Or(_) => todo!("{:?}", instr),
            Instruction::Xor(_) => todo!("{:?}", instr),
            Instruction::Cmp(_) => todo!("{:?}", instr),
            Instruction::Inc(_) => todo!("{:?}", instr),
            Instruction::Dec(_) => todo!("{:?}", instr),
            Instruction::RotLeftCarry(_) => todo!("{:?}", instr),
            Instruction::RotRightCarry(_) => todo!("{:?}", instr),
            Instruction::RotLeft(_) => todo!("{:?}", instr),
            Instruction::RotRight(_) => todo!("{:?}", instr),
            Instruction::ShiftLeftArith(_) => todo!("{:?}", instr),
            Instruction::ShiftRightArith(_) => todo!("{:?}", instr),
            Instruction::Swap(_) => todo!("{:?}", instr),
            Instruction::ShiftRightLogic(_) => todo!("{:?}", instr),
            Instruction::Bit(_, _) => todo!("{:?}", instr),
            Instruction::Res(_, _) => todo!("{:?}", instr),
            Instruction::Set(_, _) => todo!("{:?}", instr),
            Instruction::Load8(_, _) => todo!("{:?}", instr),
            Instruction::Load16(_, _) => todo!("{:?}", instr),
            Instruction::LoadAtoHLI => todo!("{:?}", instr),
            Instruction::LoadAtoHLD => todo!("{:?}", instr),
            Instruction::LoadHLItoA => todo!("{:?}", instr),
            Instruction::LoadHLDtoA => todo!("{:?}", instr),
            Instruction::LoadSPi8toHL(_) => todo!("{:?}", instr),
            Instruction::Jump(_) => todo!("{:?}", instr),
            Instruction::JumpRel(_) => todo!("{:?}", instr),
            Instruction::JumpHL => todo!("{:?}", instr),
            Instruction::JumpIf(_, _) => todo!("{:?}", instr),
            Instruction::JumpRelIf(_, _) => todo!("{:?}", instr),
            Instruction::Call(_) => todo!("{:?}", instr),
            Instruction::CallIf(_, _) => todo!("{:?}", instr),
            Instruction::Ret => todo!("{:?}", instr),
            Instruction::Reti => todo!("{:?}", instr),
            Instruction::RetIf(_) => todo!("{:?}", instr),
            Instruction::Pop(_) => todo!("{:?}", instr),
            Instruction::Push(_) => todo!("{:?}", instr),
            Instruction::DecimalAdjust => todo!("{:?}", instr),
            Instruction::ComplementAccumulator => todo!("{:?}", instr),
            Instruction::SetCarryFlag => todo!("{:?}", instr),
            Instruction::ComplementCarry => todo!("{:?}", instr),
            Instruction::Rst(_) => todo!("{:?}", instr),
            Instruction::IllegalInstruction(illegal) => {
                return Err(InstructionExecutionError::Illegal(illegal));
            },
        };

        Ok(())
    }
}
