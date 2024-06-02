mod registers;

use std::collections::VecDeque;

use thiserror::Error;

use registers::Registers;

use crate::{
    extern_traits::{GBAllocator, RomReader},
    isa::*,
    memcontroller::{MemController, MemControllerDecoderErr, ReadError, WriteError},
};

pub struct Cpu {
    cycles_remaining: u8,
    interrupts_master: bool,
    registers: Registers,
}

#[derive(Debug, Error)]
pub enum CpuErr {
    #[error("Error during instruction decoding: {0}")]
    Decode(#[from] MemControllerDecoderErr),

    #[error("Illegal instruction: {0}")]
    Illegal(u8),

    #[error("Could not write to memory: {0}")]
    MemWriteError(#[from] WriteError),

    #[error("Could not read to memory: {0}")]
    MemReadError(#[from] ReadError),
}

macro_rules! instr_todo {
    ($instr:expr) => {
        todo!("{}", $instr)
    };
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            cycles_remaining: 0,
            interrupts_master: false,
            registers: Registers::new(),
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
            Reg8::L => self.registers.l(),
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
            Reg8::L => self.registers.set_l(val),
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

    fn memloc_to_addr(&self, memloc: MemLoc) -> u16 {
        match memloc {
            MemLoc::HighMemReg(reg) => 0xFF00 | (self.get_reg8_value(reg) as u16),
            MemLoc::Reg(reg) => self.get_reg16_value(reg),
            MemLoc::HighMemImm(imm) => 0xFF00 | (imm as u16),
            MemLoc::Imm(imm) => imm,
        }
    }

    fn check_condition(&self, cond: Condition) -> bool {
        match cond {
            Condition::Zero => self.registers.zero_flag(),
            Condition::NotZero => !self.registers.zero_flag(),
            Condition::Carry => self.registers.carry_flag(),
            Condition::NotCarry => !self.registers.carry_flag(),
        }
    }

    fn do_rel_jump(&mut self, base: u16, offset: i8) {
        let jump_addr = u16::try_from((base as i32) + (offset as i32)).unwrap();
        self.registers.set_pc(jump_addr);
    }

    fn do_push8(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        val: u8,
    ) -> Result<(), WriteError> {
        self.registers.set_sp(self.registers.sp() - 1);
        mem.write8(self.registers.sp(), val)
    }

    fn do_pop8(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<u8, ReadError> {
        let val = mem.read8(self.registers.sp());

        self.registers.set_sp(self.registers.sp() + 1);

        val
    }

    fn do_push16(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        val: u16,
    ) -> Result<(), WriteError> {
        self.registers.set_sp(self.registers.sp() - 2);
        mem.write16(self.registers.sp(), val)
    }

    fn do_pop16(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<u16, ReadError> {
        let val = mem.read16(self.registers.sp());

        self.registers.set_sp(self.registers.sp() + 2);

        val
    }

    fn do_call(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        return_addr: u16,
        call_addr: u16,
    ) -> Result<(), WriteError> {
        self.do_push16(mem, return_addr)?;
        self.registers.set_pc(call_addr);
        Ok(())
    }

    fn get_prefarith_tgt(
        &self,
        mem: &MemController<impl GBAllocator, impl RomReader>,
        tgt: PrefArithTarget,
    ) -> Result<u8, ReadError> {
        match tgt {
            PrefArithTarget::Reg(reg) => Ok(self.get_reg8_value(reg)),
            PrefArithTarget::MemHL => mem.read8(self.registers.hl()),
        }
    }

    fn set_prefarith_tgt(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        tgt: PrefArithTarget,
        val: u8,
    ) -> Result<(), WriteError> {
        match tgt {
            PrefArithTarget::Reg(reg) => {
                self.set_reg8_value(reg, val);
                Ok(())
            }
            PrefArithTarget::MemHL => mem.write8(self.registers.hl(), val),
        }
    }

    pub fn run_cycle(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), CpuErr> {
        if self.cycles_remaining != 0 {
            // Still executing, continue later
            self.cycles_remaining -= 1;
            return Ok(());
        }

        let instr = decoder::decode(mem, self.registers.pc())?;

        log::trace!("Running 0x{:x}: {}", self.registers.pc(), instr);

        let jumped = match instr {
            Instruction::Nop => false,
            Instruction::Stop(_) => instr_todo!(instr),
            Instruction::Halt => instr_todo!(instr),
            Instruction::EI => instr_todo!(instr),
            Instruction::DI => instr_todo!(instr),
            Instruction::Add(_) => instr_todo!(instr),
            Instruction::AddCarry(_) => instr_todo!(instr),
            Instruction::AddHL(_) => instr_todo!(instr),
            Instruction::AddSP(_) => instr_todo!(instr),
            Instruction::Sub(_) => instr_todo!(instr),
            Instruction::SubCarry(_) => instr_todo!(instr),
            Instruction::And(_) => instr_todo!(instr),
            Instruction::Or(_) => instr_todo!(instr),
            Instruction::Xor(src) => {
                let val = match src {
                    ArithSrc::Reg(reg) => self.get_reg8_value(reg),
                    ArithSrc::Imm(imm) => imm,
                    ArithSrc::Mem(_) => instr_todo!(instr),
                };

                let xord = self.registers.a() ^ val;

                self.registers.set_a(xord);

                self.registers.set_flags(xord == 0, false, false, false);

                false
            }
            Instruction::Cmp(_) => instr_todo!(instr),
            Instruction::Inc(tgt) => {
                match tgt {
                    IncDecTarget::Reg8(reg) => self.set_reg8_value(reg, self.get_reg8_value(reg)),
                    IncDecTarget::Reg16(reg) => {
                        self.set_reg16_value(reg, self.get_reg16_value(reg))
                    }
                    IncDecTarget::MemHL => {
                        let addr = self.registers.hl();
                        mem.write8(addr, mem.read8(addr)? + 1)?;
                    }
                };
                false
            }
            Instruction::Dec(tgt) => {
                match tgt {
                    IncDecTarget::Reg8(reg) => self.set_reg8_value(reg, self.get_reg8_value(reg)),
                    IncDecTarget::Reg16(reg) => {
                        self.set_reg16_value(reg, self.get_reg16_value(reg))
                    }
                    IncDecTarget::MemHL => {
                        let addr = self.registers.hl();
                        mem.write8(addr, mem.read8(addr)? - 1)?;
                    }
                };
                false
            }
            Instruction::RotLeftCarry(_) => instr_todo!(instr),
            Instruction::RotRightCarry(_) => instr_todo!(instr),
            Instruction::RotLeft(tgt) => {
                let carry_set = self.registers.carry_flag();
                let cur_val = self.get_prefarith_tgt(mem, tgt)?;
                let (shifted, overflown) = cur_val.overflowing_shl(1);

                self.registers.set_carry_flag(overflown);
                self.set_prefarith_tgt(mem, tgt, shifted | (carry_set as u8))?;

                false
            }
            Instruction::RotRight(_) => instr_todo!(instr),
            Instruction::ShiftLeftArith(_) => instr_todo!(instr),
            Instruction::ShiftRightArith(_) => instr_todo!(instr),
            Instruction::Swap(_) => instr_todo!(instr),
            Instruction::ShiftRightLogic(_) => instr_todo!(instr),
            Instruction::Bit(bit, tgt) => {
                let val = match tgt {
                    PrefArithTarget::Reg(reg) => self.get_reg8_value(reg),
                    PrefArithTarget::MemHL => instr_todo!(instr),
                };

                let is_zero = val & (1 << (bit as usize)) == 0;

                self.registers.set_zero_flag(is_zero);
                false
            }
            Instruction::Res(_, _) => instr_todo!(instr),
            Instruction::Set(_, _) => instr_todo!(instr),
            Instruction::Load8(dst, src) => {
                let val = match src {
                    Ld8Src::Reg(reg) => self.get_reg8_value(reg),
                    Ld8Src::Mem(memloc) => mem.read8(self.memloc_to_addr(memloc))?,
                    Ld8Src::Imm(imm) => imm,
                };

                match dst {
                    Ld8Dst::Mem(memloc) => mem.write8(self.memloc_to_addr(memloc), val)?,
                    Ld8Dst::Reg(reg) => self.set_reg8_value(reg, val),
                };

                false
            }
            Instruction::Load16(dst, src) => {
                let val = match src {
                    Ld16Src::Reg(reg) => self.get_reg16_value(reg),
                    Ld16Src::Imm(imm) => imm,
                };

                match dst {
                    Ld16Dst::Mem(memloc) => mem.write16(self.memloc_to_addr(memloc), val)?,
                    Ld16Dst::Reg(reg) => self.set_reg16_value(reg, val),
                };

                false
            }
            Instruction::LoadAtoHLI => {
                let val = self.registers.a();
                let addr = self.registers.hl();

                mem.write8(addr, val)?;

                self.registers.set_hl(addr + 1);

                false
            }
            Instruction::LoadAtoHLD => {
                let val = self.registers.a();
                let addr = self.registers.hl();

                mem.write8(addr, val)?;

                self.registers.set_hl(addr - 1);

                false
            }
            Instruction::LoadHLItoA => instr_todo!(instr),
            Instruction::LoadHLDtoA => instr_todo!(instr),
            Instruction::LoadSPi8toHL(_) => instr_todo!(instr),
            Instruction::Jump(_) => instr_todo!(instr),
            Instruction::JumpRel(offset) => {
                self.do_rel_jump(self.registers.pc() + (instr.len() as u16), offset);
                true
            }
            Instruction::JumpHL => instr_todo!(instr),
            Instruction::JumpIf(_, _) => instr_todo!(instr),
            Instruction::JumpRelIf(offset, condition) => {
                if self.check_condition(condition) {
                    self.do_rel_jump(self.registers.pc() + (instr.len() as u16), offset);
                    true
                } else {
                    false
                }
            }
            Instruction::Call(addr) => {
                let curr_addr = self.registers.pc();
                let return_addr = curr_addr + (instr.len() as u16);

                self.do_call(mem, return_addr, addr)?;

                true
            }
            Instruction::CallIf(addr, cond) => {
                if self.check_condition(cond) {
                    let curr_addr = self.registers.pc();
                    let return_addr = curr_addr + (instr.len() as u16);

                    self.do_call(mem, return_addr, addr)?;

                    true
                } else {
                    false
                }
            }
            Instruction::Ret => {
                let ret_addr = self.do_pop16(mem)?;
                self.registers.set_pc(ret_addr);

                true
            }
            Instruction::Reti => instr_todo!(instr),
            Instruction::RetIf(_) => instr_todo!(instr),
            Instruction::Pop(reg) => {
                let val = self.do_pop16(mem)?;
                self.set_reg16_value(reg, val);

                false
            }
            Instruction::Push(reg) => {
                self.do_push16(mem, self.get_reg16_value(reg))?;

                false
            }
            Instruction::DecimalAdjust => instr_todo!(instr),
            Instruction::ComplementAccumulator => instr_todo!(instr),
            Instruction::SetCarryFlag => instr_todo!(instr),
            Instruction::ComplementCarry => instr_todo!(instr),
            Instruction::Rst(_) => instr_todo!(instr),
            Instruction::IllegalInstruction(illegal) => {
                return Err(CpuErr::Illegal(illegal));
            }
        };

        // Set PC to next instruction, if we didn't jump
        if !jumped {
            let instr_len = instr.len() as u16;

            self.registers.set_pc(self.registers.pc() + instr_len);
        }

        // Calculate remaining cycles
        match instr.cycles() {
            TCycles::Static(cycles) => self.cycles_remaining = cycles - 1,
            TCycles::Branching { taken, non_taken } => {
                let actual_cycles = if jumped { taken } else { non_taken };

                self.cycles_remaining = actual_cycles - 1;
            }
        }

        Ok(())
    }
}
