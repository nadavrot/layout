//! This is a collection of useful utilities.

#[cfg(feature = "log")]
use log;
use std::fs::File;
use std::io::{Error, Read, Seek, Write};

pub fn save_to_file(filename: &str, content: &str) -> Result<(), Error> {
    let f = File::create(filename)?;
    let _ = write!(&f, "{}", content);
    #[cfg(feature = "log")]
    log::info!("Wrote {}", filename);
    Result::Ok(())
}

pub(crate) fn get_image_size(filename: &str) -> Result<(u32, u32), Error> {
    if let Ok(image_size) = get_png_size(filename) {
        return Ok(image_size);
    }

    // TODO: Add support for other image formats (e.g., JPEG, SVG) following graphviz specs

    Err(Error::new(
        std::io::ErrorKind::InvalidData,
        "Unsupported image format",
    ))
}

fn get_png_size(filename: &str) -> Result<(u32, u32), Error> {
    let mut f = File::open(filename)?;
    let mut signature = [0; 8];

    f.read_exact(&mut signature)?;

    if signature != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "Not a valid PNG file",
        ));
    }
    f.seek(std::io::SeekFrom::Current(4))?;

    let mut chunk_type = [0; 4];
    f.read_exact(&mut chunk_type)?;
    if &chunk_type != b"IHDR" {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "Missing IHDR chunk",
        ));
    }

    let mut width_bytes = [0; 4];
    f.read_exact(&mut width_bytes)?;
    let mut height_bytes = [0; 4];
    f.read_exact(&mut height_bytes)?;

    let width = u32::from_be_bytes(width_bytes);
    let height = u32::from_be_bytes(height_bytes);

    Ok((width, height))
}
