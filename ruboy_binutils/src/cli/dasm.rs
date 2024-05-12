use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, about, version)]
pub struct CLIArgs {
    pub file: PathBuf,
}
