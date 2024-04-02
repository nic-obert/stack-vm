use std::path::PathBuf;

use clap::Parser;


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// The input bytecode file to execute.
    #[clap(required = true)]
    pub input_file: PathBuf,

    /// Set the stack size in bytes.
    #[clap(default_value="1000000")]
    pub stack_size: usize,

    /// Execute in verbose mode.
    #[clap(short='v', long)]
    pub verbose: bool,

}

