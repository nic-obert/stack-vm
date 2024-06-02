use std::fs;
use std::path::Path;
use std::io;

use vmlib::ByteCode;


pub fn load_assembly(file_path: &Path) -> io::Result<String> {

    let file_content = fs::read_to_string(file_path)?;
    Ok(file_content)
}


fn generate_output_name(input_name: &Path) -> String {
    
    input_name.with_extension("out").to_str().unwrap().to_string()
}


pub fn save_byte_code(byte_code: ByteCode, input_file: &Path) -> io::Result<String> {

    let output_name = generate_output_name(input_file);

    fs::write(&output_name, byte_code)?;
    
    Ok(output_name)
}

