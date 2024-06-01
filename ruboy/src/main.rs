use std::fs::File;
use std::io::BufReader;

use anyhow::{Context, Result};
use clap::Parser;
use ruboy_lib::allocator::StackAllocator;
use ruboy_lib::Gameboy;

use crate::args::CLIArgs;

mod args;

fn main() -> Result<()> {
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

    let romfile = File::open(args.rom).context("Could not open file at provided path")?;
    let reader = BufReader::new(romfile);

    let gameboy =
        Gameboy::<StackAllocator, _>::new(reader).context("Could not initialize Gaemboy")?;

    gameboy.start();

    Ok(())
}
