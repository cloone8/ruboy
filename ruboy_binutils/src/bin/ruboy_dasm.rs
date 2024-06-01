use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek},
};

use anyhow::{Context, Result};
use clap::Parser;
use ruboy_binutils::{cli::dasm, ListOutput};
use ruboy_lib::isa::{
    decoder::{decode, DecoderReadable},
    Instruction,
};

// TODO: Pull into main lab and properly implement Read and Seek traits
struct SmartReader<R: Read + Seek + ?Sized> {
    pos: Cell<usize>,
    reader: RefCell<BufReader<R>>,
}

impl<R: Read + Seek> SmartReader<R> {
    pub fn new(read: R) -> Self {
        let mut bufreader = BufReader::new(read);
        let init_pos = bufreader.stream_position().unwrap();

        Self {
            pos: Cell::new(init_pos as usize),
            reader: RefCell::new(bufreader),
        }
    }
}

impl<R: Read + Seek + ?Sized> DecoderReadable for SmartReader<R> {
    fn read_at(&self, idx: usize) -> Option<u8> {
        let mut reader = self.reader.borrow_mut();
        let cur_pos = self.pos.get();
        let offset = (idx - cur_pos) as isize;

        reader.seek_relative(offset as i64).ok()?;
        let mut buf: [u8; 1] = [0; 1];

        reader.read_exact(&mut buf).ok()?;
        self.pos.replace(idx + 1);

        Some(buf[0])
    }
}

fn display_output(instructions: &HashMap<usize, Instruction>) {
    let mut sorted: Vec<(usize, Instruction)> = instructions
        .iter()
        .map(|(&addr, &instr)| (addr, instr))
        .collect();

    sorted.sort_by(|x, y| usize::cmp(&x.0, &y.0));

    let mut output = ListOutput::new();

    for (addr, instr) in sorted {
        output.add_single(format!("0x{:x}", addr), instr);
    }

    println!("{}", output);
}

fn main() -> Result<()> {
    let args = dasm::CLIArgs::parse();

    let filepath = args.file;
    let file = File::open(filepath).context("Failed to open file")?;

    let reader = SmartReader::new(file);

    let mut instructions: HashMap<usize, Instruction> = HashMap::new();

    let mut cur_addr: usize = 0x0;

    while let Ok(instr) = decode(&reader, cur_addr as u16) {
        let existing = instructions.insert(cur_addr, instr);

        assert!(existing.is_none());

        if let Instruction::IllegalInstruction(_) = instr {
            cur_addr += 1;
        } else {
            cur_addr += instr.len() as usize;
        }
    }

    display_output(&instructions);

    Ok(())
}
