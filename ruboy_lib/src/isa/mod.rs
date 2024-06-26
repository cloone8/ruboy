use std::fmt::Display;

mod cycles;
pub mod decoder;
mod len;

#[cfg(test)]
mod testutils;

pub use cycles::*;

#[cfg(feature = "isa_display")]
pub mod display;

#[derive(Debug, Copy, Clone)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Debug, Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, Copy, Clone)]
pub enum MemLoc {
    /// 0xFF00 + u8
    HighMemReg(Reg8),
    Reg(Reg16),
    /// 0xFF00 + u8
    HighMemImm(u8),
    Imm(u16),
}

#[derive(Debug, Copy, Clone)]
pub enum ArithSrc {
    Reg(Reg8),
    Imm(u8),
    Mem(MemLoc),
}

#[derive(Debug, Copy, Clone)]
pub enum Ld8Src {
    Reg(Reg8),
    Mem(MemLoc),
    Imm(u8),
}

#[derive(Debug, Copy, Clone)]
pub enum Ld8Dst {
    Mem(MemLoc),
    Reg(Reg8),
}

#[derive(Debug, Copy, Clone)]
pub enum Ld16Src {
    Reg(Reg16),
    Imm(u16),
}

impl Ld16Src {
    const fn op_size(&self) -> u8 {
        match self {
            Ld16Src::Reg(_) => 0,
            Ld16Src::Imm(_) => 2,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Ld16Dst {
    Mem(MemLoc),
    Reg(Reg16),
}

#[derive(Debug, Copy, Clone)]
pub enum IncDecTarget {
    Reg8(Reg8),
    Reg16(Reg16),
    MemHL,
}

#[derive(Debug, Copy, Clone)]
pub enum PrefArithTarget {
    Reg(Reg8),

    /// Memory location in HL
    MemHL,
}

#[derive(Debug, Copy, Clone)]
pub enum Bit {
    B0 = 0,
    B1 = 1,
    B2 = 2,
    B3 = 3,
    B4 = 4,
    B5 = 5,
    B6 = 6,
    B7 = 7,
}

#[derive(Debug, Copy, Clone)]
pub enum Condition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

#[derive(Debug, Copy, Clone)]
pub enum RsVec {
    Rst0 = 0x00,
    Rst1 = 0x08,
    Rst2 = 0x10,
    Rst3 = 0x18,
    Rst4 = 0x20,
    Rst5 = 0x28,
    Rst6 = 0x30,
    Rst7 = 0x38,
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    /// No operation
    Nop,

    /// Enter low power mode
    Stop(u8),

    /// Enter low power mode until interrupt
    Halt,

    /// Enable interrupts _after_ instruction following this one by setting IME
    EI,

    /// Disable interrupts by clearing IME
    DI,

    /// Add value from source to register A, store result in A
    Add(ArithSrc),

    /// Add value from source plus carry flag to register A, store result in A
    AddCarry(ArithSrc),

    /// Add value from source to register HL, store result in HL
    AddHL(Reg16),

    /// Add signed value to SP
    AddSP(i8),

    /// Subtract value from source from register A, store result in A
    Sub(ArithSrc),

    /// Subtract value from source plus carry flag from register A, store result in A
    SubCarry(ArithSrc),

    /// Bitwise AND of register A and source, store result in A
    And(ArithSrc),

    /// Bitwise OR of register A and source, store result in A
    Or(ArithSrc),

    /// Bitwise XOR of register A and source, store result in A    
    Xor(ArithSrc),

    /// Subtract value from source from register A, set flags but don't store result
    Cmp(ArithSrc),

    /// Increment value at target
    Inc(IncDecTarget),

    /// Decrement value at target
    Dec(IncDecTarget),

    /// Rotate left
    RotLeftCircular(PrefArithTarget),

    /// Rotate A left
    RotLeftCircularA,

    /// Rotate right
    RotRightCircular(PrefArithTarget),

    /// Rotate A right
    RotRightCircularA,

    /// Rotate left through carry
    RotLeft(PrefArithTarget),

    /// Rotate A left through carry
    RotLeftA,

    /// Rotate right through carry
    RotRight(PrefArithTarget),

    /// Rotate A right through carry
    RotRightA,

    /// Shift left arithmetically
    ShiftLeftArith(PrefArithTarget),

    /// Shift right arithmetically
    ShiftRightArith(PrefArithTarget),

    /// Swap upper and lower 4 bits
    Swap(PrefArithTarget),

    /// Shift right locically
    ShiftRightLogic(PrefArithTarget),

    /// Set zero flag if bit is 0
    Bit(Bit, PrefArithTarget),

    /// Set bit to 0 (RESET)
    Res(Bit, PrefArithTarget),

    /// Set bit to 1 (SET)
    Set(Bit, PrefArithTarget),

    /// Load 8 bit value from source to destination
    Load8(Ld8Dst, Ld8Src),

    /// Load 16 bit value from source to destination
    Load16(Ld16Dst, Ld16Src),

    /// Load value from A into address stored in HL, increment HL afterwards
    LoadAtoHLI,

    /// Load value from A into address stored in HL, decrement HL afterwards
    LoadAtoHLD,

    /// Load value from address stored in HL into A, increment HL afterwards
    LoadHLItoA,

    /// Load value from address stored in HL into A, decrement HL afterwards
    LoadHLDtoA,

    /// Add SP to signed immediate value, store result in HL
    LoadSPi8toHL(i8),

    /// Jump to address
    Jump(u16),

    /// Jump to relative address
    JumpRel(i8),

    /// Jump to address stored in HL
    JumpHL,

    /// Jump to address if condition is met
    JumpIf(u16, Condition),

    /// Jump to relative address if condition is met
    JumpRelIf(i8, Condition),

    /// Call subroutine at address
    Call(u16),

    /// Call subroutine at address if condition is met
    CallIf(u16, Condition),

    /// Return from subroutine, AKA pop PC from stack
    Ret,

    /// Same as [Instruction::Ret], but enables interrupts before returning
    Reti,

    /// Return from subroutine if condition is met, AKA pop PC from stack if condition is met
    RetIf(Condition),

    /// Pop value from stack into register
    Pop(Reg16),

    /// Push value from register onto stack
    Push(Reg16),

    /// Decimal Adjust Accumulator to get correct BCD representation after an arithmetic
    /// instruction
    /// TODO: What does that actually mean
    DecimalAdjust,

    /// Complement A (A = ~A)
    ComplementAccumulator,

    /// Set carry flag to 1
    SetCarryFlag,

    /// Inverts carry flag
    ComplementCarry,

    /// Call address contained in this instruction.
    Rst(RsVec),

    /// Illegal instruction, stop CPU. Opcode is provided for debugging
    IllegalInstruction(u8),
}

impl Display for Instruction {
    #[cfg(feature = "isa_display")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_displayable = display::DisplayableInstruction::from(*self);

        write!(
            f,
            "{}",
            as_displayable.with_format(&display::FormatOpts::rgdbs())
        )
    }

    #[cfg(not(feature = "isa_display"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
