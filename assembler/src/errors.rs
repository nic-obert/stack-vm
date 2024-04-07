use std::io;


pub fn io_error(err: io::Error) -> ! {
    eprintln!("IO error: {}", err);
    std::process::exit(1);
}

