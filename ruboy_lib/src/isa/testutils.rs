pub(super) fn illegal_opcodes() -> Vec<u8> {
    vec![
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ]
}

pub(super) fn legal_instrs() -> impl Iterator<Item = [u8; 3]> {
    // All 8 bit opcodes
    let legal_opcodes = (0..=u8::MAX).filter(|x| !illegal_opcodes().contains(x));

    legal_opcodes.flat_map(|opcode| {
        (0..=u16::MAX)
            .map(|suffix| suffix.to_ne_bytes())
            .map(move |suffix_bytes| [opcode, suffix_bytes[0], suffix_bytes[1]])
    })
}
