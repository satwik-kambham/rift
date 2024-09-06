use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
};

/// Read file at path to string
pub fn read_file_content(path: &str) -> Result<String, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut buf = String::new();

    let _ = f.read_to_string(&mut buf)?;

    Ok(buf)
}

/// Override file at path with new content
pub fn override_file_content(path: &str, buf: String) -> Result<(), Box<dyn Error>> {
    let mut f = File::create(path)?;
    f.write_all(buf.as_bytes())?;

    Ok(())
}
