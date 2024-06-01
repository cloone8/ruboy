use clap::Parser;
use ruboy_lib::allocator::StackAllocator;
use ruboy_lib::Gameboy;

use crate::args::CLIArgs;

mod args;

fn main() {
    let args = CLIArgs::parse();

    let logconfig = simplelog::ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set logger time offset to local")
        .build();

    simplelog::TermLogger::init(
        args.verbosity.into(),
        logconfig,
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .expect("Could not initialize logger");

    log::info!("Starting Ruboy Emulator Frontend");

    let mut gameboy = Gameboy::<StackAllocator>::new();

    gameboy.start();
}
