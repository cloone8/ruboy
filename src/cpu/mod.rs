mod registers;
mod instructions;

use registers::Registers;

pub struct Cpu {
    registers: Registers
}
