//! Package pre-sized PNGs into ICO/ICNS containers without altering their pixels.
use std::{env, fs, io, path::Path};
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 4 {
        return Err(io::Error::other(
            "Usage: write-icons PNG_DIRECTORY OUTPUT.ico OUTPUT.icns",
        ));
    }
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let mut images = Vec::new();
    for size in sizes {
        let png = fs::read(Path::new(&args[1]).join(format!("{size}.png")))?;
        if png.len() < 24
            || &png[..8] != b"\x89PNG\r\n\x1a\n"
            || u32::from_be_bytes(png[16..20].try_into().unwrap()) != size
            || u32::from_be_bytes(png[20..24].try_into().unwrap()) != size
        {
            return Err(io::Error::other(format!("Expected a square {size}px PNG")));
        }
        images.push(png);
    }
    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0]);
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (size, png) in sizes.into_iter().zip(&images) {
        let dimension = if size == 256 { 0 } else { size as u8 };
        ico.extend_from_slice(&[dimension, dimension, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for png in images {
        ico.extend(png);
    }
    fs::write(&args[2], ico)?;
    let mut chunks = Vec::new();
    for (tag, size) in [
        (b"icp4", 16),
        (b"icp5", 32),
        (b"icp6", 64),
        (b"ic07", 128),
        (b"ic08", 256),
        (b"ic09", 512),
        (b"ic10", 1024),
        (b"ic11", 32),
        (b"ic12", 64),
        (b"ic13", 256),
        (b"ic14", 512),
    ] {
        let png = fs::read(Path::new(&args[1]).join(format!("{size}.png")))?;
        chunks.extend_from_slice(tag);
        chunks.extend_from_slice(&((png.len() + 8) as u32).to_be_bytes());
        chunks.extend(png);
    }
    let mut icns = b"icns".to_vec();
    icns.extend_from_slice(&((chunks.len() + 8) as u32).to_be_bytes());
    icns.extend(chunks);
    fs::write(&args[3], icns)
}
