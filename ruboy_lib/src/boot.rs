use cfg_if::cfg_if;

#[cfg(target_os = "windows")]
macro_rules! path_sep {
    () => {
        "\\"
    };
}

#[cfg(not(target_os = "windows"))]
macro_rules! path_sep {
    () => {
        "/"
    };
}

cfg_if! {
    if #[cfg(feature = "boot_dmg0")] {
        pub const IMAGE_NAME: &str = "DMG0";
        pub const IMAGE: &'static [u8] = include_bytes!(concat!("..", path_sep!(), "boot", path_sep!(), "dmg0.bin"));
    } else if #[cfg(feature = "boot_dmg")] {
        pub const IMAGE_NAME: &str = "DMG";
        pub const IMAGE: &'static [u8] = include_bytes!(concat!("..", path_sep!(), "boot", path_sep!(), "dmg.bin"));
    } else if #[cfg(feature = "boot_mgb")] {
        pub const IMAGE_NAME: &str = "MGB";
        pub const IMAGE: &'static [u8] = include_bytes!(concat!("..", path_sep!(), "boot", path_sep!(), "mgb.bin"));
    }
}
