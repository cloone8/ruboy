pub const fn get_tac_modulo(tac: u8) -> Option<usize> {
    if tac & 0b100 == 0 {
        None
    } else {
        let clock_select_val = tac & 0b11;

        let val = match clock_select_val {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => unreachable!(),
        };

        Some(val)
    }
}
