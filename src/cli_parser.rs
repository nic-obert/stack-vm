use clap::Parser;


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// The input bytecode file to execute.
    #[clap(value_parser, required = true)]
    input_file: String,

    /// Set the stack size in bytes.
    #[clap(default_value="1000000")]
    stack_size: usize,

    /// Execute in verbose mode.
    #[clap(short='v', long)]
    verbose: bool,

}

