use crate::GbInputs;

pub fn apply_input_to(cur_joypad_register: u8, cur_inputs: GbInputs) -> (u8, bool) {
    let select_buttons = cur_joypad_register & 0b00100000 == 0;
    let select_dpad = cur_joypad_register & 0b00010000 == 0;
    let selector_nibble = cur_joypad_register & 0b11110000;

    if select_buttons {
        let new_joypad_reg_value = selector_nibble | get_input_nibble_for_buttons(cur_inputs);
        let can_raise_interrupt =
            cur_joypad_register & 0x0F == 0x0F && new_joypad_reg_value & 0x0F != 0x0F;

        (new_joypad_reg_value, can_raise_interrupt)
    } else if select_dpad {
        let new_joypad_reg_value = selector_nibble | get_input_nibble_for_dpad(cur_inputs);
        let can_raise_interrupt =
            cur_joypad_register & 0x0F == 0x0F && new_joypad_reg_value & 0x0F != 0x0F;

        (new_joypad_reg_value, can_raise_interrupt)
    } else {
        (selector_nibble | 0b00001111, false)
    }
}

fn get_input_nibble_for_buttons(inputs: GbInputs) -> u8 {
    let start = if !inputs.start { 0b00001000 } else { 0 };

    let select = if !inputs.select { 0b00000100 } else { 0 };

    let b = if !inputs.b { 0b00000010 } else { 0 };

    let a = if !inputs.a { 0b00000001 } else { 0 };

    let res = start | select | b | a;

    debug_assert!(0b11110000 & res == 0);

    res
}

fn get_input_nibble_for_dpad(inputs: GbInputs) -> u8 {
    let down = if !inputs.down { 0b00001000 } else { 0 };

    let up = if !inputs.up { 0b00000100 } else { 0 };

    let left = if !inputs.left { 0b00000010 } else { 0 };

    let right = if !inputs.right { 0b00000001 } else { 0 };

    let res = down | left | up | right;

    debug_assert!(0b11110000 & res == 0);

    res
}
