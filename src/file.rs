use std::fs::File;
use std::io::prelude::*;

pub fn write(response: &str) -> std::io::Result<()> {
    let mut file = File::create("test/output.txt")?;
    file.write_all(response.as_bytes())?;
    Ok(())
}