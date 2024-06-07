#[derive(Debug, Clone, Copy)]
pub struct OldLicensee {
    pub code: u8,
    pub names: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct NewLicensee {
    /// Two ascii chars
    pub code: [u8; 2],
    pub names: &'static [&'static str],
}

pub const OLD_LICENSEES: &[OldLicensee] = &[
    OldLicensee {
        code: 0x01,
        names: &["Nintendo"],
    },
    OldLicensee {
        code: 0x08,
        names: &["Capcom"],
    },
    OldLicensee {
        code: 0x09,
        names: &["HOT-B"],
    },
    OldLicensee {
        code: 0x0A,
        names: &["Jaleco"],
    },
    OldLicensee {
        code: 0x0B,
        names: &["Coconuts Japan"],
    },
    OldLicensee {
        code: 0x0C,
        names: &["Elite Systems"],
    },
    OldLicensee {
        code: 0x13,
        names: &["EA (Electronic Arts)"],
    },
    OldLicensee {
        code: 0x18,
        names: &["Hudson Soft"],
    },
    OldLicensee {
        code: 0x19,
        names: &["ITC Entertainment"],
    },
    OldLicensee {
        code: 0x1A,
        names: &["Yanoman"],
    },
    OldLicensee {
        code: 0x1D,
        names: &["Japan Clary"],
    },
    OldLicensee {
        code: 0x1F,
        names: &[
            "Virgin Games Ltd.",
            "Virgin Mastertronic Ltd.",
            "Virgin Interactive Entertainment",
            "Avalon Interactive Group Ltd.",
        ],
    },
    OldLicensee {
        code: 0x24,
        names: &["PCM Complete"],
    },
    OldLicensee {
        code: 0x25,
        names: &["San-X"],
    },
    OldLicensee {
        code: 0x28,
        names: &["Kemco"],
    },
    OldLicensee {
        code: 0x29,
        names: &["SETA Corporation"],
    },
    OldLicensee {
        code: 0x30,
        names: &["Infogrames", "Atari SA"],
    },
    OldLicensee {
        code: 0x31,
        names: &["Nintendo"],
    },
    OldLicensee {
        code: 0x32,
        names: &["Bandai"],
    },
    OldLicensee {
        code: 0x34,
        names: &["Konami"],
    },
    OldLicensee {
        code: 0x35,
        names: &["HectorSoft"],
    },
    OldLicensee {
        code: 0x38,
        names: &["Capcom"],
    },
    OldLicensee {
        code: 0x39,
        names: &["Banpresto"],
    },
    OldLicensee {
        code: 0x3C,
        names: &[".Entertainment i"],
    },
    OldLicensee {
        code: 0x3E,
        names: &["Gremlin"],
    },
    OldLicensee {
        code: 0x41,
        names: &["Ubi Soft", "Ubisoft"],
    },
    OldLicensee {
        code: 0x42,
        names: &["Atlus"],
    },
    OldLicensee {
        code: 0x44,
        names: &["Malibu Interactive"],
    },
    OldLicensee {
        code: 0x46,
        names: &["Angel"],
    },
    OldLicensee {
        code: 0x47,
        names: &["Spectrum Holoby"],
    },
    OldLicensee {
        code: 0x49,
        names: &["Irem"],
    },
    OldLicensee {
        code: 0x4A,
        names: &[
            "Virgin Games Ltd.",
            "Virgin Mastertronic Ltd.",
            "Virgin Interactive Entertainment",
            "Avalon Interactive Group Ltd.",
        ],
    },
    OldLicensee {
        code: 0x4D,
        names: &["Malibu Interactive"],
    },
    OldLicensee {
        code: 0x4F,
        names: &["U.S. Gold"],
    },
    OldLicensee {
        code: 0x50,
        names: &["Absolute"],
    },
    OldLicensee {
        code: 0x51,
        names: &["Acclaim Entertainment"],
    },
    OldLicensee {
        code: 0x52,
        names: &["Activision"],
    },
    OldLicensee {
        code: 0x53,
        names: &["Sammy USA Corporation"],
    },
    OldLicensee {
        code: 0x54,
        names: &["GameTek"],
    },
    OldLicensee {
        code: 0x55,
        names: &["Park Place"],
    },
    OldLicensee {
        code: 0x56,
        names: &["LJN"],
    },
    OldLicensee {
        code: 0x57,
        names: &["Matchbox"],
    },
    OldLicensee {
        code: 0x59,
        names: &["Milton Bradley Company"],
    },
    OldLicensee {
        code: 0x5A,
        names: &["Mindscape"],
    },
    OldLicensee {
        code: 0x5B,
        names: &["Romstar"],
    },
    OldLicensee {
        code: 0x5C,
        names: &["Naxat Soft", "Kaga Create"],
    },
    OldLicensee {
        code: 0x5D,
        names: &["Tradewest"],
    },
    OldLicensee {
        code: 0x60,
        names: &["Titus Interactive"],
    },
    OldLicensee {
        code: 0x61,
        names: &[
            "Virgin Games Ltd.",
            "Virgin Mastertronic Ltd.",
            "Virgin Interactive Entertainment",
            "Avalon Interactive Group Ltd.",
        ],
    },
    OldLicensee {
        code: 0x67,
        names: &["Ocean Software"],
    },
    OldLicensee {
        code: 0x69,
        names: &["EA (Electronic Arts)"],
    },
    OldLicensee {
        code: 0x6E,
        names: &["Elite Systems"],
    },
    OldLicensee {
        code: 0x6F,
        names: &["Electro Brain"],
    },
    OldLicensee {
        code: 0x70,
        names: &["Infogrames", "Atari SA"],
    },
    OldLicensee {
        code: 0x71,
        names: &["Interplay Entertainment"],
    },
    OldLicensee {
        code: 0x72,
        names: &["Broderbund"],
    },
    OldLicensee {
        code: 0x73,
        names: &["Sculptured Software", "Iguana Entertainment"],
    },
    OldLicensee {
        code: 0x75,
        names: &[
            "The Sales Curve Limited",
            "SCi",
            "SCi Entertainment Group plc",
            "Eidos",
        ],
    },
    OldLicensee {
        code: 0x78,
        names: &["THQ"],
    },
    OldLicensee {
        code: 0x79,
        names: &["Accolade"],
    },
    OldLicensee {
        code: 0x7A,
        names: &["Triffix Entertainment"],
    },
    OldLicensee {
        code: 0x7C,
        names: &["Microprose"],
    },
    OldLicensee {
        code: 0x7F,
        names: &["Kemco"],
    },
    OldLicensee {
        code: 0x80,
        names: &["Misawa Entertainment"],
    },
    OldLicensee {
        code: 0x83,
        names: &["Lozc"],
    },
    OldLicensee {
        code: 0x86,
        names: &["Tokuma Shoten"],
    },
    OldLicensee {
        code: 0x8B,
        names: &["Bullet-Proof Software", "Blue Planet Software"],
    },
    OldLicensee {
        code: 0x8C,
        names: &["Vic Tokai"],
    },
    OldLicensee {
        code: 0x8E,
        names: &["Ape"],
    },
    OldLicensee {
        code: 0x8F,
        names: &["I'Max"],
    },
    OldLicensee {
        code: 0x91,
        names: &["Chunsoft Co.", "Spike Chunsoft Co., Ltd."],
    },
    OldLicensee {
        code: 0x92,
        names: &["Video System"],
    },
    OldLicensee {
        code: 0x93,
        names: &["Tsubaraya Productions"],
    },
    OldLicensee {
        code: 0x95,
        names: &["Varie"],
    },
    OldLicensee {
        code: 0x96,
        names: &["Yonezawa", "S'Pal"],
    },
    OldLicensee {
        code: 0x97,
        names: &["Kemco"],
    },
    OldLicensee {
        code: 0x99,
        names: &["Arc"],
    },
    OldLicensee {
        code: 0x9A,
        names: &["Nihon Bussan"],
    },
    OldLicensee {
        code: 0x9B,
        names: &["Tecmo"],
    },
    OldLicensee {
        code: 0x9C,
        names: &["Imagineer"],
    },
    OldLicensee {
        code: 0x9D,
        names: &["Banpresto"],
    },
    OldLicensee {
        code: 0x9F,
        names: &["Nova"],
    },
    OldLicensee {
        code: 0xA1,
        names: &["Hori Electric"],
    },
    OldLicensee {
        code: 0xA2,
        names: &["Bandai"],
    },
    OldLicensee {
        code: 0xA4,
        names: &["Konami"],
    },
    OldLicensee {
        code: 0xA6,
        names: &["Kawada"],
    },
    OldLicensee {
        code: 0xA7,
        names: &["Takara"],
    },
    OldLicensee {
        code: 0xA9,
        names: &["Technos Japan"],
    },
    OldLicensee {
        code: 0xAA,
        names: &["Broderbund"],
    },
    OldLicensee {
        code: 0xAC,
        names: &["Toei Animation"],
    },
    OldLicensee {
        code: 0xAD,
        names: &["Toho"],
    },
    OldLicensee {
        code: 0xAF,
        names: &["Namco"],
    },
    OldLicensee {
        code: 0xB0,
        names: &["Acclaim Entertainment"],
    },
    OldLicensee {
        code: 0xB1,
        names: &["ASCII Corporation", "Nexoft"],
    },
    OldLicensee {
        code: 0xB2,
        names: &["Bandai"],
    },
    OldLicensee {
        code: 0xB4,
        names: &["Square Enix"],
    },
    OldLicensee {
        code: 0xB6,
        names: &["HAL Laboratory"],
    },
    OldLicensee {
        code: 0xB7,
        names: &["SNK"],
    },
    OldLicensee {
        code: 0xB9,
        names: &["Pony Canyon"],
    },
    OldLicensee {
        code: 0xBA,
        names: &["Culture Brain"],
    },
    OldLicensee {
        code: 0xBB,
        names: &["Sunsoft"],
    },
    OldLicensee {
        code: 0xBD,
        names: &["Sony Imagesoft"],
    },
    OldLicensee {
        code: 0xBF,
        names: &["Sammy Corporation"],
    },
    OldLicensee {
        code: 0xC0,
        names: &["Taito"],
    },
    OldLicensee {
        code: 0xC2,
        names: &["Kemco"],
    },
    OldLicensee {
        code: 0xC3,
        names: &["Square"],
    },
    OldLicensee {
        code: 0xC4,
        names: &["Tokuma Shoten"],
    },
    OldLicensee {
        code: 0xC5,
        names: &["Data East"],
    },
    OldLicensee {
        code: 0xC6,
        names: &["Tonkinhouse"],
    },
    OldLicensee {
        code: 0xC8,
        names: &["Koei"],
    },
    OldLicensee {
        code: 0xC9,
        names: &["UFL"],
    },
    OldLicensee {
        code: 0xCA,
        names: &["Ultra"],
    },
    OldLicensee {
        code: 0xCB,
        names: &["Vap"],
    },
    OldLicensee {
        code: 0xCC,
        names: &["Use Corporation"],
    },
    OldLicensee {
        code: 0xCD,
        names: &["Meldac"],
    },
    OldLicensee {
        code: 0xCE,
        names: &["Pony Canyon"],
    },
    OldLicensee {
        code: 0xCF,
        names: &["Angel"],
    },
    OldLicensee {
        code: 0xD0,
        names: &["Taito"],
    },
    OldLicensee {
        code: 0xD1,
        names: &["Sofel"],
    },
    OldLicensee {
        code: 0xD2,
        names: &["Quest"],
    },
    OldLicensee {
        code: 0xD3,
        names: &["Sigma Enterprises"],
    },
    OldLicensee {
        code: 0xD4,
        names: &["ASK Kodansha Co."],
    },
    OldLicensee {
        code: 0xD6,
        names: &["Naxat Soft", "Kaga Create"],
    },
    OldLicensee {
        code: 0xD7,
        names: &["Copya System"],
    },
    OldLicensee {
        code: 0xD9,
        names: &["Banpresto"],
    },
    OldLicensee {
        code: 0xDA,
        names: &["Tomy"],
    },
    OldLicensee {
        code: 0xDB,
        names: &["LJN"],
    },
    OldLicensee {
        code: 0xDD,
        names: &["NCS"],
    },
    OldLicensee {
        code: 0xDE,
        names: &["Human"],
    },
    OldLicensee {
        code: 0xDF,
        names: &["Altron"],
    },
    OldLicensee {
        code: 0xE0,
        names: &["Jaleco"],
    },
    OldLicensee {
        code: 0xE1,
        names: &["Towa Chiki"],
    },
    OldLicensee {
        code: 0xE2,
        names: &["Yutaka"],
    },
    OldLicensee {
        code: 0xE3,
        names: &["Varie"],
    },
    OldLicensee {
        code: 0xE5,
        names: &["Epcoh"],
    },
    OldLicensee {
        code: 0xE7,
        names: &["Athena"],
    },
    OldLicensee {
        code: 0xE8,
        names: &["Asmik Ace Entertainment"],
    },
    OldLicensee {
        code: 0xE9,
        names: &["Natsume"],
    },
    OldLicensee {
        code: 0xEA,
        names: &["King Records"],
    },
    OldLicensee {
        code: 0xEB,
        names: &["Atlus"],
    },
    OldLicensee {
        code: 0xEC,
        names: &["Epic", "Sony Records"],
    },
    OldLicensee {
        code: 0xEE,
        names: &["IGS"],
    },
    OldLicensee {
        code: 0xF0,
        names: &["A Wave"],
    },
    OldLicensee {
        code: 0xF3,
        names: &["Extreme Entertainment"],
    },
    OldLicensee {
        code: 0xFF,
        names: &["LJN"],
    },
];
pub const NEW_LICENSEES: &[NewLicensee] = &[
    NewLicensee {
        code: [b'0', b'0'],
        names: &["None"],
    },
    NewLicensee {
        code: [b'0', b'1'],
        names: &["Nintendo Research & Development 1"],
    },
    NewLicensee {
        code: [b'0', b'8'],
        names: &["Capcom"],
    },
    NewLicensee {
        code: [b'1', b'3'],
        names: &["EA (Electronic Arts)"],
    },
    NewLicensee {
        code: [b'1', b'8'],
        names: &["Hudson Soft"],
    },
    NewLicensee {
        code: [b'1', b'9'],
        names: &["B-AI"],
    },
    NewLicensee {
        code: [b'2', b'0'],
        names: &["KSS"],
    },
    NewLicensee {
        code: [b'2', b'2'],
        names: &["Planning Office WADA"],
    },
    NewLicensee {
        code: [b'2', b'4'],
        names: &["PCM Complete"],
    },
    NewLicensee {
        code: [b'2', b'5'],
        names: &["San-X"],
    },
    NewLicensee {
        code: [b'2', b'8'],
        names: &["Kemco"],
    },
    NewLicensee {
        code: [b'2', b'9'],
        names: &["SETA Corporation"],
    },
    NewLicensee {
        code: [b'3', b'0'],
        names: &["Viacom"],
    },
    NewLicensee {
        code: [b'3', b'1'],
        names: &["Nintendo"],
    },
    NewLicensee {
        code: [b'3', b'2'],
        names: &["Bandai"],
    },
    NewLicensee {
        code: [b'3', b'3'],
        names: &["Ocean Software", "Acclaim Entertainment"],
    },
    NewLicensee {
        code: [b'3', b'4'],
        names: &["Konami"],
    },
    NewLicensee {
        code: [b'3', b'5'],
        names: &["HectorSoft"],
    },
    NewLicensee {
        code: [b'3', b'7'],
        names: &["Taito"],
    },
    NewLicensee {
        code: [b'3', b'8'],
        names: &["Hudson Soft"],
    },
    NewLicensee {
        code: [b'3', b'9'],
        names: &["Banpresto"],
    },
    NewLicensee {
        code: [b'4', b'1'],
        names: &["Ubi Soft", "Ubisoft"],
    },
    NewLicensee {
        code: [b'4', b'2'],
        names: &["Atlus"],
    },
    NewLicensee {
        code: [b'4', b'4'],
        names: &["Malibu Interactive"],
    },
    NewLicensee {
        code: [b'4', b'6'],
        names: &["Angel"],
    },
    NewLicensee {
        code: [b'4', b'7'],
        names: &["Bullet-Proof Software", "Blue Planet Software"],
    },
    NewLicensee {
        code: [b'4', b'9'],
        names: &["Irem"],
    },
    NewLicensee {
        code: [b'5', b'0'],
        names: &["Absolute"],
    },
    NewLicensee {
        code: [b'5', b'1'],
        names: &["Acclaim Entertainment"],
    },
    NewLicensee {
        code: [b'5', b'2'],
        names: &["Activision"],
    },
    NewLicensee {
        code: [b'5', b'3'],
        names: &["Sammy USA Corporation"],
    },
    NewLicensee {
        code: [b'5', b'4'],
        names: &["Konami"],
    },
    NewLicensee {
        code: [b'5', b'5'],
        names: &["Hi Tech Expressions"],
    },
    NewLicensee {
        code: [b'5', b'6'],
        names: &["LJN"],
    },
    NewLicensee {
        code: [b'5', b'7'],
        names: &["Matchbox"],
    },
    NewLicensee {
        code: [b'5', b'8'],
        names: &["Mattel"],
    },
    NewLicensee {
        code: [b'5', b'9'],
        names: &["Milton Bradley Company"],
    },
    NewLicensee {
        code: [b'6', b'0'],
        names: &["Titus Interactive"],
    },
    NewLicensee {
        code: [b'6', b'1'],
        names: &[
            "Virgin Games Ltd.",
            "Virgin Mastertronic Ltd.",
            "Virgin Interactive Entertainment",
            "Avalon Interactive Group, Ltd.",
        ],
    },
    NewLicensee {
        code: [b'6', b'4'],
        names: &["Lucasfilm Games", "LucasArts"],
    },
    NewLicensee {
        code: [b'6', b'7'],
        names: &["Ocean Software"],
    },
    NewLicensee {
        code: [b'6', b'9'],
        names: &["EA (Electronic Arts)"],
    },
    NewLicensee {
        code: [b'7', b'0'],
        names: &["Infogrames", "Atari SA"],
    },
    NewLicensee {
        code: [b'7', b'1'],
        names: &["Interplay Entertainment"],
    },
    NewLicensee {
        code: [b'7', b'2'],
        names: &["Broderbund"],
    },
    NewLicensee {
        code: [b'7', b'3'],
        names: &["Sculptured Software", "Iguana Entertainment"],
    },
    NewLicensee {
        code: [b'7', b'5'],
        names: &[
            "The Sales Curve Limited",
            "SCi",
            "SCi Entertainment Group",
            "Eidos",
        ],
    },
    NewLicensee {
        code: [b'7', b'8'],
        names: &["THQ"],
    },
    NewLicensee {
        code: [b'7', b'9'],
        names: &["Accolade"],
    },
    NewLicensee {
        code: [b'8', b'0'],
        names: &["Misawa Entertainment"],
    },
    NewLicensee {
        code: [b'8', b'3'],
        names: &["lozc"],
    },
    NewLicensee {
        code: [b'8', b'6'],
        names: &["Tokuma Shoten"],
    },
    NewLicensee {
        code: [b'8', b'7'],
        names: &["Tsukuda Original"],
    },
    NewLicensee {
        code: [b'9', b'1'],
        names: &["Chunsoft Co.", "Spike Chunsoft Co., Ltd."],
    },
    NewLicensee {
        code: [b'9', b'2'],
        names: &["Video System"],
    },
    NewLicensee {
        code: [b'9', b'3'],
        names: &["Ocean Software", "Acclaim Entertainment"],
    },
    NewLicensee {
        code: [b'9', b'5'],
        names: &["Varie"],
    },
    NewLicensee {
        code: [b'9', b'6'],
        names: &["Yonezawa", "s'pal"],
    },
    NewLicensee {
        code: [b'9', b'7'],
        names: &["Kaneko"],
    },
    NewLicensee {
        code: [b'9', b'9'],
        names: &["Pack-In-Video"],
    },
    NewLicensee {
        code: [b'9', b'H'],
        names: &["Bottom Up"],
    },
    NewLicensee {
        code: [b'A', b'4'],
        names: &["Konami (Yu-Gi-Oh!)"],
    },
    NewLicensee {
        code: [b'B', b'L'],
        names: &["MTO"],
    },
    NewLicensee {
        code: [b'D', b'K'],
        names: &["Kodansha"],
    },
];

pub fn find_old(code: u8) -> Option<OldLicensee> {
    OLD_LICENSEES.iter().find(|l| l.code == code).copied()
}

pub fn find_new(code: [u8; 2]) -> Option<NewLicensee> {
    NEW_LICENSEES.iter().find(|l| l.code == code).copied()
}

#[cfg(test)]
mod tests {
    use super::{NEW_LICENSEES, OLD_LICENSEES};

    #[test]
    fn ensure_old_unique() {
        let mut found: Vec<u8> = Vec::new();

        for licensee in OLD_LICENSEES {
            assert!(
                !found.contains(&licensee.code),
                "Licensee code 0x{:x} is duplicate",
                licensee.code
            );
            found.push(licensee.code);
        }
    }

    #[test]
    fn ensure_new_unique() {
        let mut found: Vec<u16> = Vec::new();

        for licensee in NEW_LICENSEES {
            let as16 = u16::from_ne_bytes(licensee.code);

            assert!(
                !found.contains(&as16),
                "Licensee code [0x{:x}, 0x{:x}] is duplicate",
                licensee.code[0],
                licensee.code[1]
            );

            found.push(as16);
        }
    }
}
