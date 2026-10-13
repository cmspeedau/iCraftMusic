//! Files the tests write for themselves.
//!
//! The crate ships no audio fixtures, and the two places that need a real file
//! — the tag writer and the editor that drives it — need the *same* one, so it
//! is built here once. Nothing in this module is compiled into the application:
//! it exists for `#[cfg(test)]` only.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Prefix of every scratch directory this module makes.
const SCRATCH: &str = "musiccraft-tests";

/// How many scratch directories have been made, so two tests asking for the
/// same name never share one.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// A directory of this process's own, emptied and recreated.
///
/// Tests run in parallel and the temporary directory is shared, and several of
/// them build the same fixture: the name a caller gives is therefore only the
/// readable part of the path, and a counter makes it unique — without it, two
/// tests would take turns deleting each other's file mid-run.
#[must_use]
pub fn scratch(name: &str) -> PathBuf {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let directory =
        std::env::temp_dir().join(format!("{SCRATCH}-{}-{serial}-{name}", std::process::id()));
    std::fs::remove_dir_all(&directory).ok();
    std::fs::create_dir_all(&directory).expect("the scratch directory should be creatable");

    directory
}

/// Write `bytes` into a scratch directory of its own and return the file's path.
#[must_use]
pub fn scratch_file(name: &str, bytes: &[u8]) -> PathBuf {
    let path = scratch(name).join(name);
    std::fs::write(&path, bytes).expect("the scratch file should be writable");

    path
}

/// A minimal, valid, silent WAV file.
///
/// Tag writing can only be tested against a real audio file, and an 8-bit mono
/// PCM WAV is a header and a run of bytes: exactly the kind of file `lofty` can
/// read, tag and hand back. `wav` is also a format whose tags this build can
/// write, so it is the sample the happy paths use.
#[must_use]
pub fn silent_wav(samples: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + samples).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&8000u32.to_le_bytes()); // sample rate
    bytes.extend_from_slice(&8000u32.to_le_bytes()); // byte rate
    bytes.extend_from_slice(&1u16.to_le_bytes()); // block align
    bytes.extend_from_slice(&8u16.to_le_bytes()); // bits per sample
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&samples.to_le_bytes());
    bytes.extend(std::iter::repeat_n(0x80, samples as usize)); // silence

    bytes
}

/// A tiny PNG, encoded by the `image` crate the album view already depends on.
#[must_use]
pub fn png() -> Vec<u8> {
    let mut pixels = image::RgbaImage::new(2, 2);
    pixels.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));

    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(pixels)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("the encoder should write a PNG");

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scratch_file_is_written_where_it_says_it_is() {
        let path = scratch_file("hello.txt", b"hi");

        assert!(path.starts_with(std::env::temp_dir()));
        assert_eq!(std::fs::read(&path).expect("readable"), b"hi");
    }

    #[test]
    fn the_silent_wav_is_the_size_its_header_promises() {
        let wav = silent_wav(64);
        let declared = u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]);

        assert_eq!(declared as usize + 8, wav.len());
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn the_png_is_a_png() {
        let png = png();

        assert_eq!(&png[..4], b"\x89PNG");
        assert!(crate::ui::artwork::decode(&png).is_some());
    }
}
