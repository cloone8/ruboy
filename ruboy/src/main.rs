use ruboy_lib::{self, BoxedGBRam, Gameboy};

fn main() {
    let logconfig = simplelog::ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set logger time offset to local")
        .build();

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        logconfig,
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto
    ).expect("Could not initialize logger");

    log::info!("Starting Ruboy Emulator Frontend");

    let mut gameboy = Gameboy::<BoxedGBRam>::new();

    gameboy.start();
}
