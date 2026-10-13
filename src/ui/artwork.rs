//! Cover art for the Library view.
//!
//! The scanner writes the pictures it extracts to a cache directory as ordinary
//! image files; this module turns one of those files into an `egui` texture,
//! once, and hands the same texture to every frame that asks for it. Decoding
//! is the expensive part, so a failure is remembered as well as a success: a
//! corrupt cover is not retried on every repaint.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::{ColorImage, Context, TextureHandle, TextureOptions};

/// How many covers are kept resident before the cache starts over.
///
/// A library can hold thousands of albums, and a texture per album would grow
/// without bound. The view only ever draws the tiles that fit on screen, so
/// emptying the cache when it fills up costs one re-decode per visible tile and
/// keeps memory flat.
pub const MAX_TEXTURES: usize = 256;

/// Decoded covers, keyed by the path they were cached at.
#[derive(Default)]
pub struct ArtworkCache {
    textures: HashMap<PathBuf, Option<TextureHandle>>,
}

impl ArtworkCache {
    /// An empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many covers have been looked at, successfully or not.
    #[must_use]
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// Whether nothing has been loaded yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// Forget every cover, so the next frame decodes them again.
    pub fn clear(&mut self) {
        self.textures.clear();
    }

    /// Forget one cover, so the next frame decodes it again.
    ///
    /// The cache file for a track keeps its name when the user chooses a
    /// different picture, so the decoded texture has to be dropped or the view
    /// would keep drawing the cover that was just replaced.
    pub fn forget(&mut self, path: &Path) {
        self.textures.remove(path);
    }

    /// The texture for the cover at `path`, decoding it the first time.
    ///
    /// Returns `None` when the file is missing, unreadable, or in a format the
    /// build cannot decode; the album view draws its placeholder instead.
    pub fn texture(&mut self, ctx: &Context, path: &Path) -> Option<TextureHandle> {
        if let Some(cached) = self.textures.get(path) {
            return cached.clone();
        }

        if self.textures.len() >= MAX_TEXTURES {
            self.clear();
        }

        let texture = load(ctx, path);
        self.textures.insert(path.to_path_buf(), texture.clone());

        texture
    }
}

/// Read and decode one cover file into a texture.
fn load(ctx: &Context, path: &Path) -> Option<TextureHandle> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::debug!("cover art at {} could not be read: {error}", path.display());
            return None;
        }
    };

    let image = decode(&bytes)?;
    let name = path.to_string_lossy().into_owned();

    Some(ctx.load_texture(name, image, TextureOptions::LINEAR))
}

/// Decode encoded image bytes into an `egui` image, or `None` when the data is
/// not a format this build can read.
///
/// The decoded pixels are unmultiplied, which is what `egui` expects from a
/// JPEG or a PNG.
#[must_use]
pub fn decode(bytes: &[u8]) -> Option<ColorImage> {
    let decoded = image::load_from_memory(bytes).ok()?;
    let rgba = decoded.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];

    Some(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    /// A tiny PNG of our own, so the test does not need a fixture file.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut pixels = image::RgbaImage::new(width, height);
        pixels.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));

        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(pixels)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("the encoder should write a PNG");

        bytes
    }

    /// A scratch directory of our own, so tests never share state.
    fn scratch(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("musiccraft-artwork-ui-{name}"));
        std::fs::remove_dir_all(&directory).ok();
        std::fs::create_dir_all(&directory).expect("the scratch directory should be creatable");

        directory
    }

    #[test]
    fn a_png_decodes_to_its_pixel_size() {
        let image = decode(&png(3, 2)).expect("a PNG should decode");

        assert_eq!(image.size, [3, 2]);
    }

    #[test]
    fn data_that_is_not_an_image_decodes_to_nothing() {
        assert!(decode(b"not an image at all").is_none());
        assert!(decode(&[]).is_none());
    }

    #[test]
    fn a_cover_is_decoded_once_and_reused() {
        let directory = scratch("decode-once");
        let path = directory.join("cover.png");
        std::fs::write(&path, png(2, 2)).expect("the cover should be writable");

        let ctx = Context::default();
        let mut cache = ArtworkCache::new();

        let first = cache.texture(&ctx, &path).expect("the cover should load");
        let second = cache.texture(&ctx, &path).expect("the cover is cached");

        assert_eq!(first.id(), second.id());
        assert_eq!(
            cache.len(),
            1,
            "the second call does not store anything new"
        );

        std::fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn a_missing_or_broken_cover_is_remembered_as_unusable() {
        let directory = scratch("broken");
        let missing = directory.join("nothing.png");
        let broken = directory.join("broken.png");
        std::fs::write(&broken, b"definitely not a png").expect("the file should be writable");

        let ctx = Context::default();
        let mut cache = ArtworkCache::new();

        assert!(cache.texture(&ctx, &missing).is_none());
        assert!(cache.texture(&ctx, &broken).is_none());
        assert_eq!(cache.len(), 2, "both failures are remembered");

        cache.clear();
        assert!(cache.is_empty());

        std::fs::remove_dir_all(&directory).ok();
    }
}
