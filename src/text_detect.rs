use std::io::Read;
use std::path::Path;

use anyhow::Result;

pub fn is_probably_text(path: &Path) -> Result<bool> {
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return Ok(false),
    };

    let mut buf = [0u8; 8192];
    let bytes_read = file.read(&mut buf)?;

    if bytes_read == 0 {
        return Ok(false);
    }

    Ok(!buf[..bytes_read].contains(&0))
}
