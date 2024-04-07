use std::path::PathBuf;

use clap::Parser;


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// The input bytecode file to execute.
    #[clap(required = true)]
    pub input_file: PathBuf,

    /// Set the stack size in bytes.
    #[clap()]
    pub stack_size: Option<usize>,

    /// Execute in verbose mode.
    #[clap(short='v', long)]
    pub verbose: bool,

}

