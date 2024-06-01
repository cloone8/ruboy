use std::{
    fmt::format,
    fs::File,
    io::{BufReader, Read, Seek},
};

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use ruboy_binutils::{cli::romdump, ListOutput};
use ruboy_lib::rom::meta::RomMeta;

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

fn generate_checksum_string(valid: bool) -> ColoredString {
    if valid {
        "valid".green()
    } else {
        "invalid".red()
    }
}

fn display_rom_meta(meta: &RomMeta) {
    let mut output = ListOutput::new();
    output.add_single("Title", meta.title());
    output.add_single("Manufacturer", meta.manufacturer());
    output.add_single("CGB Support", meta.cgb_support());
    output.add_single("Licensee", meta.licensee());
    output.add_single("SGB Support", meta.sgb_support());

    let hw = meta.cartridge_hardware();

    let mut hw_strs: Vec<String> = Vec::new();

    if let Some(mapper) = hw.mapper() {
        hw_strs.push(format!("Mapper: {}", mapper));
    }

    if hw.has_ram() {
        hw_strs.push("RAM".into());
    }
    if hw.has_battery() {
        hw_strs.push("Battery".into());
    }
    if hw.has_timer() {
        hw_strs.push("Timer".into());
    }
    if hw.has_rumble() {
        hw_strs.push("Rumble".into());
    }
    if hw.has_sensor() {
        hw_strs.push("Sensor".into());
    }
    if hw.has_camera() {
        hw_strs.push("Camera".into());
    }

    output.add_multiple("Cartridge hardware", hw_strs);

    output.add_single("ROM size", meta.rom_size());
    output.add_single("RAM size", meta.ram_size());
    output.add_single("Intended destination", meta.destination());
    output.add_single("Game version number", meta.game_version());
    output.add_single(
        "Header checksum",
        format!(
            "0x{:x} ({})",
            meta.header_checksum(),
            generate_checksum_string(meta.header_checksum_valid())
        ),
    );
    output.add_single("Global checksum", format!("0x{:x}", meta.global_checksum()));

    println!("{}", output);
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
