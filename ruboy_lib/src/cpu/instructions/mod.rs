use crate::memcontroller::MemController;
use crate::{GBAllocator, RomReader};

use super::nums::{GbBits, GbHalfCarry};
use super::{Cpu, CpuErr, IncDecTarget, Instruction, Ld16Dst, Ld16Src, Ld8Dst, Ld8Src};

macro_rules! instr_todo {
    ($instr:expr) => {
        todo!("{}", $instr)
    };
}

impl Cpu {
    /// Runs the given CPU instruction
    pub fn execute_instruction(
        &mut self,
        mem: &mut MemController<impl GBAllocator, impl RomReader>,
        instr: Instruction,
    ) -> Result<bool, CpuErr> {
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
            Instruction::AddCarry(src) => {
                let base = self.registers.a();
                let val = self.get_arith_src(mem, src)?;
                let cur_carry = if self.registers.carry_flag() { 1 } else { 0 };

                let (res, new_carry) = base.overflowing_add(val + cur_carry);

                self.registers.set_flags(
                    res == 0,
                    false,
                    base.halfcarry_add(val + cur_carry),
                    new_carry,
                );

                self.registers.set_a(res);

                false
            }
            Instruction::AddHL(reg) => {
                let base = self.registers.hl();
                let val = self.get_reg16_value(reg);

                let (res, carry) = base.overflowing_add(val);

                self.registers
                    .set_flags(res == 0, false, base.halfcarry_add(val), carry);

                self.registers.set_hl(res);

                false
            }
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
            Instruction::RotRight(tgt) => {
                let init_val = self.get_prefarith_tgt(mem, tgt)?;
                let shifted = init_val.wrapping_shr(1);
                let result = shifted.set_msb(self.registers.carry_flag());

                self.registers
                    .set_flags(result == 0, false, false, init_val.lsb_set());

                self.set_prefarith_tgt(mem, tgt, result)?;

                false
            }
            Instruction::ShiftLeftArith(tgt) => {
                let init_val = self.get_prefarith_tgt(mem, tgt)?;
                let shifted = init_val.wrapping_shl(1);

                self.registers
                    .set_flags(shifted == 0, false, false, init_val.msb_set());

                self.set_prefarith_tgt(mem, tgt, shifted)?;

                false
            }
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
            Instruction::ShiftRightLogic(tgt) => {
                let val = self.get_prefarith_tgt(mem, tgt)?;

                let carry = val.lsb_set();

                let res = val.wrapping_shr(1);

                self.registers.set_flags(res == 0, false, false, carry);

                self.set_prefarith_tgt(mem, tgt, res)?;

                false
            }
            Instruction::Bit(bit, tgt) => {
                let val = self.get_prefarith_tgt(mem, tgt)?;

                let is_zero = val & (1 << (bit as usize)) == 0;

                self.registers.set_zero_flag(is_zero);
                self.registers.set_subtract_flag(false);
                self.registers.set_half_carry_flag(true);

                false
            }
            Instruction::Res(bit, tgt) => {
                let val = self.get_prefarith_tgt(mem, tgt)?;

                let bit: u8 = 0b1 << bit as usize;

                self.set_prefarith_tgt(mem, tgt, val & (!bit))?;

                false
            }
            Instruction::Set(bit, tgt) => {
                let val = self.get_prefarith_tgt(mem, tgt)?;

                let bit: u8 = 0b1 << bit as usize;

                self.set_prefarith_tgt(mem, tgt, val | bit)?;

                false
            }
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
            Instruction::JumpHL => {
                self.registers.set_pc(self.registers.hl());
                true
            }
            Instruction::JumpIf(addr, cond) => {
                if self.check_condition(cond) {
                    self.registers.set_pc(addr);
                    true
                } else {
                    false
                }
            }
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
            Instruction::Reti => {
                // TODO: Not entirely sure if the order and timings
                // of enabling/disabling interrupts is correct.
                let ret_addr = self.do_pop16(mem)?;
                self.registers.set_pc(ret_addr);
                self.interrupts_master = true; // This is definitely not correct

                true
            }
            Instruction::RetIf(cond) => {
                if self.check_condition(cond) {
                    let ret_addr = self.do_pop16(mem)?;
                    self.registers.set_pc(ret_addr);

                    true
                } else {
                    false
                }
            }
            Instruction::Pop(reg) => {
                let val = self.do_pop16(mem)?;
                self.set_reg16_value(reg, val);

                false
            }
            Instruction::Push(reg) => {
                self.do_push16(mem, self.get_reg16_value(reg))?;

                false
            }
            Instruction::DecimalAdjust => {
                let mut a = self.registers.a();
                let cflag = self.registers.carry_flag();
                let hflag = self.registers.half_carry_flag();

                match self.registers.subtract_flag() {
                    false => {
                        if cflag || a > 0x99 {
                            a = a.wrapping_add(0x60);
                            self.registers.set_carry_flag(true);
                        }
                        if hflag || (a & 0x0F) > 0x09 {
                            a = a.wrapping_add(0x6);
                        }
                    }
                    true => {
                        if cflag {
                            a = a.wrapping_sub(0x60);
                        }
                        if hflag {
                            a = a.wrapping_sub(0x6);
                        }
                    }
                }

                self.registers.set_zero_flag(a == 0);
                self.registers.set_half_carry_flag(false);
                self.registers.set_a(a);

                false
            }
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
            Instruction::RotLeftCircularA => {
                let pre = self.registers.a();

                self.registers.set_carry_flag(pre.msb_set());

                self.registers.set_a(pre.rotate_left(1));

                false
            }
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

        Ok(jumped)
    }
}
