use std::path::PathBuf;

use clap::Parser;


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// The input assembly file to assemble.
    #[clap(required = true)]
    pub input_file: PathBuf,

    /// The output bytecode file to generate.
    #[clap(required = false)]
    pub output_file: Option<PathBuf>,

    /// Execute in verbose mode.
    #[clap(short='v', long)]
    pub verbose: bool,

    /// List of paths to search for included libraries
    #[clap(short='L', value_delimiter=',')]
    pub include_paths: Vec<PathBuf>,

}

