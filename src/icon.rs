//! The application's own icon.
//!
//! The artwork lives under `packaging/` as the PNG renditions the desktop
//! entry names, so the launcher, the AppImage and the window all wear the
//! same picture. The 256px one is embedded in the binary and handed to the
//! desktop as the window's icon; the rest are there for the icon theme to
//! pick from when the application is installed.

use egui::IconData;

/// The 256px rendition, decoded at build time, as the window's icon.
///
/// This is the same file the desktop entry resolves `Icon=musiccraft` to, so
/// there is no second copy of the picture to keep in step.
pub fn window_icon() -> IconData {
    const PNG: &[u8] = include_bytes!("../packaging/icons/hicolor/256x256/apps/musiccraft.png");

    let image = image::load_from_memory(PNG)
        .expect("the bundled icon is a PNG — it is one of the files in packaging/icons")
        .into_rgba8();
    let (width, height) = image.dimensions();

    IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The icon decodes, and at the size its file name promises: a build whose
    /// embedded picture were corrupt would open a window with no icon at all,
    /// which is the kind of failure nobody notices until the taskbar is empty.
    #[test]
    fn the_bundled_icon_decodes_at_its_named_size() {
        let icon = window_icon();

        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4, "not four bytes to a pixel");
    }

    /// The picture is the artwork, not a blank field: it has ink in it, and
    /// its ink is more than one colour.
    #[test]
    fn the_icon_carries_its_artwork() {
        let icon = window_icon();

        let (pixels, leftover) = icon.rgba.as_chunks::<4>();
        assert!(leftover.is_empty(), "the bytes do not divide into pixels");

        let first = pixels[0];
        assert!(
            pixels.iter().any(|pixel| *pixel != first),
            "the icon is one flat colour"
        );
    }

    /// The corners are cut away. The artwork's tile is a rounded square, and
    /// the renditions carry that as transparency rather than as white canvas,
    /// which is what keeps the icon from reading as a white block on a dark
    /// bar. An unmasked square re-added to `packaging/` fails here.
    #[test]
    fn the_icon_keeps_its_rounded_corners() {
        let icon = window_icon();

        assert_eq!(icon.rgba[3], 0, "the top-left corner is opaque");
        assert_eq!(
            icon.rgba[(256 * 256 - 1) * 4 + 3],
            0,
            "the bottom-right corner is opaque"
        );

        // The middle of the tile is opaque — only the corners are cut.
        let middle = ((256 / 2) * 256 + 256 / 2) * 4;
        assert_eq!(
            icon.rgba[middle + 3],
            255,
            "the middle of the icon is see-through"
        );
    }
}
