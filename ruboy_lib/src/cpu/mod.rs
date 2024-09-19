mod instructions;
mod nums;
mod registers;
mod timer;

use core::num::Wrapping;

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
    timer_cycles: Wrapping<usize>,
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

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            timer_cycles: Wrapping(0),
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

    fn handle_timers(&mut self, mem: &mut MemController<impl GBAllocator, impl RomReader>) {
        if self.timer_cycles.0 % 256 == 0 {
            mem.io_registers.timer_div += 1;
        }

        if let Some(tac_frequency) = timer::get_tac_modulo(mem.io_registers.timer_control) {
            if self.timer_cycles.0 % tac_frequency == 0 {
                let (incremented, overflown) = mem.io_registers.timer_counter.overflowing_add(1);

                if overflown {
                    mem.io_registers.timer_counter = mem.io_registers.timer_modulo;
                    mem.io_registers.interrupts_requested.set_timer(true);
                } else {
                    mem.io_registers.timer_counter = incremented;
                }
            }
        }

        self.timer_cycles += 1;
    }

    pub fn run_cycle(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
    ) -> Result<(), CpuErr> {
        self.handle_timers(mem);

        if self.cycles_remaining != 0 {
            // Still executing, continue later
            self.cycles_remaining -= 1;
            return Ok(());
        }

        let instr = decoder::decode(mem, self.registers.pc())?;

        log::trace!("Running 0x{:x}: {}", self.registers.pc(), instr);

        let should_enable_interrupts = self.ei_queued;

        // Actually run the instruction here
        let jumped = self.execute_instruction(mem, instr)?;

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
