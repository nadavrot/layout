//! This is a collection of useful utilities.

use log;
use std::fs::File;
use std::io::{Error, Write};

pub fn save_to_file(filename: &str, content: &str) -> Result<(), Error> {
    let f = File::create(filename)?;
    let _ = write!(&f, "{}", content);
    log::info!("Wrote {}", filename);
    Result::Ok(())
}
