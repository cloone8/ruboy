mod nums;
mod registers;

use nums::{GbBits, HalfCarry};
use thiserror::Error;

use registers::Registers;

use crate::{
    extern_traits::{GBAllocator, RomReader},
    isa::*,
    memcontroller::{
        interrupts::Interrupts, MemController, MemControllerDecoderErr, ReadError, WriteError,
    },
};

pub struct Cpu {
    cycles_remaining: u8,
    interrupts_master: bool,
    /// Whether the interrupts master flag should be re-enabled after the next instruction
    ei_queued: bool,

    registers: Registers,
}

#[derive(Debug, Error)]
pub enum CpuErr {
    #[error("Error during instruction decoding")]
    Decode(#[from] MemControllerDecoderErr),

    #[error("Illegal instruction: 0x{:x}", 0)]
    Illegal(u8),

    #[error("Could not write to memory")]
    MemWriteError(#[from] WriteError),

    #[error("Could not read from memory")]
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
            ei_queued: false,
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

    fn get_arith_src(
        &self,
        mem: &MemController<impl GBAllocator, impl RomReader>,
        src: ArithSrc,
    ) -> Result<u8, ReadError> {
        match src {
            ArithSrc::Reg(reg) => Ok(self.get_reg8_value(reg)),
            ArithSrc::Imm(imm) => Ok(imm),
            ArithSrc::Mem(memloc) => mem.read8(self.memloc_to_addr(memloc)),
        }
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

        let should_enable_interrupts = self.ei_queued;

        let jumped = match instr {
            Instruction::Nop => false,
            Instruction::Stop(_) => instr_todo!(instr),
            Instruction::Halt => instr_todo!(instr),
            Instruction::EI => {
                self.ei_queued = true;
                false
            }
            Instruction::DI => {
                self.interrupts_master = false;
                false
            }
            Instruction::Add(src) => {
                let base = self.registers.a();
                let val = self.get_arith_src(mem, src)?;

                let (res, carry) = base.overflowing_add(val);

                self.registers
                    .set_flags(res == 0, false, base.halfcarry_add(val), carry);

                self.registers.set_a(res);

                false
            }
            Instruction::AddCarry(_) => instr_todo!(instr),
            Instruction::AddHL(_) => instr_todo!(instr),
            Instruction::AddSP(_) => instr_todo!(instr),
            Instruction::Sub(src) => {
                let base = self.registers.a();
                let val = self.get_arith_src(mem, src)?;

                let (res, carry) = base.overflowing_sub(val);

                self.registers
                    .set_flags(res == 0, true, base.halfcarry_sub(val), carry);

                self.registers.set_a(res);

                false
            }
            Instruction::SubCarry(_) => instr_todo!(instr),
            Instruction::And(src) => {
                let val = self.get_arith_src(mem, src)?;

                let and = val & self.registers.a();

                self.registers.set_flags(and == 0, false, true, false);
                self.registers.set_a(and);

                false
            }
            Instruction::Or(src) => {
                let val = self.get_arith_src(mem, src)?;

                let or = val | self.registers.a();

                self.registers.set_flags(or == 0, false, false, false);
                self.registers.set_a(or);

                false
            }
            Instruction::Xor(src) => {
                let val = self.get_arith_src(mem, src)?;

                let xord = self.registers.a() ^ val;

                self.registers.set_a(xord);

                self.registers.set_flags(xord == 0, false, false, false);

                false
            }
            Instruction::Cmp(src) => {
                let base = self.registers.a();
                let val = self.get_arith_src(mem, src)?;

                let (res, carry) = base.overflowing_sub(val);

                self.registers
                    .set_flags(res == 0, true, base.halfcarry_sub(val), carry);

                false
            }
            Instruction::Inc(tgt) => {
                match tgt {
                    IncDecTarget::Reg8(reg) => {
                        let val = self.get_reg8_value(reg);
                        let incremented = val.wrapping_add(1);

                        self.registers.set_zero_flag(incremented == 0);
                        self.registers.set_subtract_flag(false);
                        self.registers.set_half_carry_flag(val.halfcarry_add(1));

                        self.set_reg8_value(reg, incremented);
                    }
                    IncDecTarget::Reg16(reg) => {
                        let val = self.get_reg16_value(reg);
                        let incremented = val.wrapping_add(1);

                        self.set_reg16_value(reg, incremented);
                    }
                    IncDecTarget::MemHL => {
                        let addr = self.registers.hl();
                        let val = mem.read8(addr)?;
                        let incremented = val.wrapping_add(1);

                        self.registers.set_zero_flag(incremented == 0);
                        self.registers.set_subtract_flag(false);
                        self.registers.set_half_carry_flag(val.halfcarry_add(1));

                        mem.write8(addr, incremented)?;
                    }
                };
                false
            }
            Instruction::Dec(tgt) => {
                match tgt {
                    IncDecTarget::Reg8(reg) => {
                        let val = self.get_reg8_value(reg);
                        let decremented = val.wrapping_sub(1);

                        self.registers.set_zero_flag(decremented == 0);
                        self.registers.set_subtract_flag(true);
                        self.registers.set_half_carry_flag(val.halfcarry_sub(1));

                        self.set_reg8_value(reg, decremented);
                    }
                    IncDecTarget::Reg16(reg) => {
                        let val = self.get_reg16_value(reg);
                        let decremented = val.wrapping_sub(1);

                        self.set_reg16_value(reg, decremented);
                    }
                    IncDecTarget::MemHL => {
                        let addr = self.registers.hl();
                        let val = mem.read8(addr)?;
                        let decremented = val.wrapping_sub(1);

                        self.registers.set_zero_flag(decremented == 0);
                        self.registers.set_subtract_flag(true);
                        self.registers.set_half_carry_flag(val.halfcarry_sub(1));

                        mem.write8(addr, decremented)?;
                    }
                };
                false
            }
            Instruction::RotLeftCircular(_) => instr_todo!(instr),
            Instruction::RotRightCircular(tgt) => {
                let pre = self.get_prefarith_tgt(mem, tgt)?;

                self.registers.set_carry_flag(pre.lsb_set());

                self.set_prefarith_tgt(mem, tgt, pre.rotate_right(1))?;

                false
            }
            Instruction::RotLeft(tgt) => {
                let init_val = self.get_prefarith_tgt(mem, tgt)?;
                let shifted = init_val.wrapping_shl(1);
                let result = shifted.set_lsb(self.registers.carry_flag());

                self.registers
                    .set_flags(result == 0, false, false, init_val.msb_set());

                self.set_prefarith_tgt(mem, tgt, result)?;

                false
            }
            Instruction::RotRight(_) => instr_todo!(instr),
            Instruction::ShiftLeftArith(_) => instr_todo!(instr),
            Instruction::ShiftRightArith(_) => instr_todo!(instr),
            Instruction::Swap(tgt) => {
                let val = self.get_prefarith_tgt(mem, tgt)?;
                let val_lower = val & 0xF;
                let val_upper = val & 0xF0;

                let swapped = (val_lower << 4) | (val_upper >> 4);

                self.set_prefarith_tgt(mem, tgt, swapped)?;

                self.registers.set_flags(swapped == 0, false, false, false);

                false
            }
            Instruction::ShiftRightLogic(_) => instr_todo!(instr),
            Instruction::Bit(bit, tgt) => {
                let val = match tgt {
                    PrefArithTarget::Reg(reg) => self.get_reg8_value(reg),
                    PrefArithTarget::MemHL => instr_todo!(instr),
                };

                let is_zero = val & (1 << (bit as usize)) == 0;

                self.registers.set_zero_flag(is_zero);
                self.registers.set_subtract_flag(false);
                self.registers.set_half_carry_flag(true);

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
            Instruction::LoadHLItoA => {
                let addr = self.registers.hl();
                let val = mem.read8(addr)?;

                self.registers.set_hl(addr + 1); // This increments HL
                self.registers.set_a(val);

                false
            }
            Instruction::LoadHLDtoA => {
                let addr = self.registers.hl();
                let val = mem.read8(addr)?;

                self.registers.set_hl(addr - 1); // This decrements HL
                self.registers.set_a(val);

                false
            }
            Instruction::LoadSPi8toHL(_) => instr_todo!(instr),
            Instruction::Jump(addr) => {
                self.registers.set_pc(addr);
                true
            }
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
            Instruction::ComplementAccumulator => {
                self.registers.set_a(!self.registers.a());
                self.registers.set_subtract_flag(true);
                self.registers.set_half_carry_flag(true);
                false
            }
            Instruction::SetCarryFlag => instr_todo!(instr),
            Instruction::ComplementCarry => instr_todo!(instr),
            Instruction::Rst(rsvec) => {
                let curr_addr = self.registers.pc();
                let return_addr = curr_addr + (instr.len() as u16);

                self.do_call(mem, return_addr, rsvec as u16)?;

                true
            }
            Instruction::RotLeftCircularA => instr_todo!(instr),
            Instruction::RotRightCircularA => {
                let pre = self.registers.a();

                self.registers.set_carry_flag(pre.lsb_set());

                self.registers.set_a(pre.rotate_right(1));

                false
            }
            Instruction::RotLeftA => {
                let cur_val = self.registers.a();
                let shifted = cur_val.wrapping_shl(1);
                let result = shifted.set_lsb(self.registers.carry_flag());

                self.registers
                    .set_flags(false, false, false, cur_val.msb_set());

                self.registers.set_a(result);

                false
            }
            Instruction::RotRightA => instr_todo!(instr),
            Instruction::IllegalInstruction(illegal) => {
                return Err(CpuErr::Illegal(illegal));
            }
        };

        if should_enable_interrupts {
            self.ei_queued = false;
            self.interrupts_master = true;
        }

        // Set PC to next instruction, if we didn't jump
        if !jumped {
            let instr_len = instr.len() as u16;

            self.registers.set_pc(self.registers.pc() + instr_len);
        }

        // Handle any interrupts.
        if self.interrupts_master {
            let enabled = mem.interrupts_enabled;
            let requested = mem.io_registers.interrupts_requested;
            let to_service: Interrupts = (u8::from(enabled) & u8::from(requested)).into();

            // We have an interrupt! Disable any following interrupts
            // and go to the handler. We check for zero
            // with the lower 5 bits, because the upper 3 are unused
            // and thus do not actually correspond to an interrupt
            if u8::from(to_service) & 0b00011111 != 0 {
                log::debug!("Handling interrupt! 0b{:b}", u8::from(to_service));
                self.interrupts_master = false;

                let handler_addr: u16 = if to_service.vblank() {
                    mem.io_registers.interrupts_requested.set_vblank(false);
                    0x40
                } else if to_service.lcd() {
                    mem.io_registers.interrupts_requested.set_lcd(false);
                    0x48
                } else if to_service.timer() {
                    mem.io_registers.interrupts_requested.set_timer(false);
                    0x50
                } else if to_service.serial() {
                    mem.io_registers.interrupts_requested.set_serial(false);
                    0x58
                } else if to_service.joypad() {
                    mem.io_registers.interrupts_requested.set_joypad(false);
                    0x60
                } else {
                    unreachable!("Not actually an interrupt");
                };

                // Return addr is just the current PC now, since we were interrupted before executing it
                self.do_call(mem, self.registers.pc(), handler_addr)?;
                self.cycles_remaining = 20; // Entire interrupt routine takes 20 cycles to complete
                return Ok(());
            }
        }

        // No interrupt was handled. Just continue execution as usual
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
