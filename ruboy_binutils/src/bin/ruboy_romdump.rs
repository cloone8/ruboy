use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};

use anyhow::{Context, Result};
use clap::Parser;
use ruboy_binutils::cli::romdump;
use ruboy_lib::rom::{CartridgeHardware, RomMeta};

fn seek_to_header_start(r: &mut BufReader<File>) -> Result<()> {
    let cur_pos = r
        .stream_position()
        .context("Could not determine reader start")?;

    let target_pos = RomMeta::OFFSET_HEADER_START;
    let offset = (target_pos - cur_pos as usize) as i64;

    r.seek_relative(offset)
        .context("Could not seek to header start")?;

    Ok(())
}

fn generate_checksum_string(valid: bool) -> &'static str {
    if valid {
        "valid"
    } else {
        "invalid"
    }
}

fn display_cartridge_hardware(hw: &CartridgeHardware) {
    if let Some(mapper) = hw.mapper() {
        println!("\t- Mapper: {}", mapper);
    }

    if hw.has_ram() {
        println!("\t- RAM");
    }
    if hw.has_battery() {
        println!("\t- Battery");
    }
    if hw.has_timer() {
        println!("\t- Timer");
    }
    if hw.has_rumble() {
        println!("\t- Rumble");
    }
    if hw.has_sensor() {
        println!("\t- Sensor");
    }
    if hw.has_camera() {
        println!("\t- Camera");
    }
}

fn display_rom_meta(meta: &RomMeta) {
    println!("Title: {}", meta.title());
    println!("Manufacturer: {}", meta.manufacturer());
    println!("CGB support: {}", meta.cgb_support());
    println!("Licensee: {}", meta.licensee());
    println!("SGB Support: {}", meta.sgb_support());

    println!("Cartridge hardware:");
    display_cartridge_hardware(&meta.cartridge_hardware());

    println!("ROM size: {}", meta.rom_size());
    println!("RAM size: {}", meta.ram_size());
    println!("Intented destination: {}", meta.destination());
    println!("Game version number: {}", meta.game_version());
    println!(
        "Header checksum: 0x{:x} ({})",
        meta.header_checksum(),
        generate_checksum_string(meta.header_checksum_valid())
    );
    println!("Global checksum: 0x{:x}", meta.global_checksum());
}

fn main() -> Result<()> {
    let args = romdump::CLIArgs::parse();

    let filepath = args.file;
    let file = File::open(filepath).context("Failed to open file")?;
    let mut reader = BufReader::new(file);

    seek_to_header_start(&mut reader)?;

    let mut header_bytes = [0u8; RomMeta::HEADER_LENGTH];

    reader.read_exact(&mut header_bytes)?;

    let meta = RomMeta::parse(&header_bytes).unwrap();

    display_rom_meta(&meta);

    Ok(())
}
