use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};

use anyhow::{Context, Result};
use clap::Parser;
use ruboy_binutils::cli::romdump;
use ruboy_lib::rom::RomMeta;

fn seek_to_header_start(r: &mut BufReader<File>) -> Result<()> {
    let cur_pos = r
        .stream_position()
        .context("Could not determine reader start")?;

    let target_pos = RomMeta::OFFSET_HEADER_START as usize;
    let offset = (target_pos - cur_pos as usize) as i64;

    r.seek_relative(offset)
        .context("Could not seek to header start")?;

    Ok(())
}

fn main() -> Result<()> {
    let args = romdump::CLIArgs::parse();

    let filepath = args.file;
    let file = File::open(filepath).context("Failed to open file")?;
    let mut reader = BufReader::new(file);

    seek_to_header_start(&mut reader)?;

    let mut header_bytes = [0u8; RomMeta::HEADER_LENGTH as usize];

    reader.read_exact(&mut header_bytes)?;

    let meta = RomMeta::parse(&header_bytes);

    println!("{:?}", meta);

    Ok(())
}
