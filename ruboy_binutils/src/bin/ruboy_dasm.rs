use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Read, Seek},
};

use anyhow::{Context, Result};
use clap::Parser;
use ruboy_binutils::{
    cli::dasm::{self, CLIArgs},
    ListOutput,
};
use ruboy_lib::isa::{
    decoder::{decode, DecoderReadable},
    display::{DisplayableInstruction, FormatOpts, ImmediateFormat},
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
    type Err = io::Error;
    fn read_at(&self, idx: usize) -> Result<u8, Self::Err> {
        let mut reader = self.reader.borrow_mut();
        let cur_pos = self.pos.get();
        let offset = idx.wrapping_sub(cur_pos) as isize;

        reader.seek_relative(offset as i64)?;
        let mut buf: [u8; 1] = [0; 1];

        reader.read_exact(&mut buf)?;
        self.pos.replace(idx + 1);

        Ok(buf[0])
    }
}

fn display_output(instructions: &HashMap<usize, String>) {
    let mut sorted: Vec<(usize, _)> = instructions
        .iter()
        .map(|(&addr, instr)| (addr, instr))
        .collect();

    sorted.sort_by(|x, y| usize::cmp(&x.0, &y.0));

    let mut output = ListOutput::new();

    for (addr, instr) in sorted {
        output.add_single(format!("0x{:x}", addr), instr);
    }

    println!("{}", output);
}

fn to_format_opts(args: &CLIArgs) -> FormatOpts {
    let mut opts = FormatOpts::rgdbs();

    if let Some(case) = args.mnemonic_case {
        opts.mnemonic_case = case.into();
    }

    if let Some(case) = args.register_case {
        opts.reg_case = case.into();
    }

    if let Some(hlid_signs) = args.hlid_signs {
        opts.hlid_as_signs = hlid_signs;
    }

    if let Ok(imm_format) = ImmediateFormat::try_from(args.immediate_format.clone()) {
        opts.imm_format = imm_format;
    }

    if let Some(op_order) = args.first_operand {
        opts.operand_order = op_order.into();
    }

    opts
}

fn format_instruction(instr: Instruction, opts: &FormatOpts) -> String {
    let displayable = DisplayableInstruction::from(instr);

    displayable.with_format(opts)
}

fn main() -> Result<()> {
    let args = dasm::CLIArgs::parse();
    let format_opts = to_format_opts(&args);
    let filepath = args.file.clone();
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

    let instructions_formatted: HashMap<_, _> = instructions
        .into_iter()
        .map(|(addr, instr)| (addr, format_instruction(instr, &format_opts)))
        .collect();

    display_output(&instructions_formatted);

    Ok(())
}
