mod registers;

use thiserror::Error;

use registers::Registers;

use crate::{
    isa::*,
    memcontroller::MemController,
    GBRam
};

use self::decoder::DecodeError;

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

    #[inline]
    const fn get_reg16_value(&self, reg: Reg16) -> u16 { 
        match reg {
            Reg16::AF => self.registers.af(),
            Reg16::BC => self.registers.bc(),
            Reg16::DE => self.registers.de(),
            Reg16::HL => self.registers.hl(),
            Reg16::SP => self.registers.sp(),
        }
    }

    #[inline]
    const fn get_reg8_value(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.registers.a(),
            Reg8::B => self.registers.b(),
            Reg8::C => self.registers.c(),
            Reg8::D => self.registers.d(),
            Reg8::E => self.registers.e(),
            Reg8::F => self.registers.f(),
            Reg8::H => self.registers.h(),
            Reg8::L => self.registers.l()
        }
    }
    
    #[inline]
    fn set_reg8_value(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.registers.set_a(val),
            Reg8::B => self.registers.set_b(val),
            Reg8::C => self.registers.set_c(val),
            Reg8::D => self.registers.set_d(val),
            Reg8::E => self.registers.set_e(val),
            Reg8::F => self.registers.set_f(val),
            Reg8::H => self.registers.set_h(val),
            Reg8::L => self.registers.set_l(val)
        }
    }

    #[inline]
    fn set_reg16_value(&mut self, reg: Reg16, val: u16) {
        match reg {
            Reg16::AF => self.registers.set_af(val),
            Reg16::BC => self.registers.set_bc(val),
            Reg16::DE => self.registers.set_de(val),
            Reg16::HL => self.registers.set_hl(val),
            Reg16::SP => self.registers.set_sp(val),
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
            Instruction::Xor(src) => {
                let val = match src {
                    ArithSrc::Reg(reg) => self.get_reg8_value(reg),
                    ArithSrc::Imm(imm) => imm,
                    ArithSrc::Mem(_) => todo!("{:?}", instr),
                };
                
                let xord = self.registers.a() ^ val;

                self.registers.set_a(xord);
                
                self.registers.set_flags(xord == 0, false, false, false);
            },
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
            Instruction::Load16(dst, src) => {
                let val = match src {
                    Ld16Src::Reg(reg) => self.get_reg16_value(reg),
                    Ld16Src::Imm(imm) => imm,
                }; 

                match dst {
                    Ld16Dst::Mem(_) => todo!("{:?}", instr),
                    Ld16Dst::Reg(reg) => self.set_reg16_value(reg, val),
                }
            },
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
        
        let instr_len = instr.len() as u16;

        log::trace!("Incrementing PC by {}, 0x{:x} -> 0x{:x}", instr_len, self.registers.pc(), self.registers.pc() + instr_len);

        self.registers.set_pc(self.registers.pc() + instr_len);

        Ok(())
    }
}
