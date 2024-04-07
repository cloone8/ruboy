
pub mod decoder;

#[derive(Debug, Copy, Clone)]
pub enum RawInstruction {
    Single(u8),
    Double(u16)
}

#[derive(Debug, Copy, Clone)]
pub enum Register8 {
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
pub enum Register16 {
    BC,
    DE,
    HL,
    SP
}

#[derive(Debug, Copy, Clone)]
pub enum MemLoc {
    Reg(Register16),
    Imm(u16)
}

#[derive(Debug, Copy, Clone)]
pub enum ArithSrc {
    Reg(Register8),
    Imm(u8),
    Mem(MemLoc)
}

#[derive(Debug, Copy, Clone)]
pub enum Ld8Src {
    Reg(Register8),
    Mem(MemLoc),
    Imm(u8),
}

#[derive(Debug, Copy, Clone)]
pub enum Ld8Dst {
    Mem(MemLoc),
    Reg(Register8)
}

#[derive(Debug, Copy, Clone)]
pub enum Ld16Src {
    Reg(Register16),
    Imm(u16),
}

#[derive(Debug, Copy, Clone)]
pub enum Ld16Dst {
    Mem(MemLoc),
    Reg(Register16)
}

#[derive(Debug, Copy, Clone)]
pub enum IncDecTarget {
    Reg8(Register8),
    Reg16(Register16),
    Mem(MemLoc)
}

#[derive(Debug, Copy, Clone)]
pub enum Condition {
    Zero,
    NotZero,
    Carry,
    NotCarry
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    /// No operation
    Nop,

    /// Enter low power mode
    Stop,

    /// Enter low power mode until interrupt
    Halt,

    /// Add value from source to register A, store result in A
    Add(ArithSrc),

    /// Add value from source plus carry flag to register A, store result in A
    AddCarry(ArithSrc),

    /// Add value from source to register HL, store result in HL
    AddHL(Register16),

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

    /// Jump to address
    Jump(u16),

    /// Jump to address stored in HL
    JumlHL,

    /// Jump to address if condition is met
    JumpIf(u16, Condition),

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
    Pop(Register16),

    /// Push value from register onto stack
    Push(Register16),

    /// Illegal instruction, stop CPU
    IllegalInstruction(RawInstruction)
}
