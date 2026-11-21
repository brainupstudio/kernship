//! Kernship's name, tagline and logo (Kerning + Spaceship: a command centre for your font galaxy).
use std::sync::Arc;

pub const NAME: &str = "Kernship";
pub const TAGLINE: &str = "A command centre for your font galaxy.";
/// The "k." mark, white on black, 4096 px square: the Dock icon and, as a template image, the
/// menu bar icon.
pub const MARK_PNG: &[u8] = include_bytes!("../assets/brand/kernship-k-white-on-black-4096.png");
/// Its black-on-white twin, for the Welcome window in the light appearance.
pub const MARK_LIGHT_PNG: &[u8] = include_bytes!("../assets/brand/kernship-k-black-on-white-4096.png");
/// Menu bar icon height in pixels at 2×: 14 pt, the height of the other menu bar icons'
/// artwork (the menu bar itself is taller; macOS centres the icon in it).
const MENUBAR_PX: u32 = 28;

/// The menu bar icon as PNG: the K mark in `rgb` on transparent, 28 px tall (14 pt at 2×).
/// Black is the version for the light menu bar, white the one for the dark menu bar; the app
/// gives macOS the mark as a template image, which it draws in the right colour by itself.
pub fn menubar_icon_png(rgb: [u8; 3]) -> Option<(Vec<u8>, u32, u32)> {
    let img = mark_image()?;
    let (w, h) = img.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            if p[0].max(p[1]).max(p[2]) > 60 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let crop = image::imageops::crop_imm(img, x0, y0, x1 - x0 + 1, y1 - y0 + 1).to_image();
    let scale = MENUBAR_PX as f32 / crop.height() as f32;
    let mw = ((crop.width() as f32 * scale).round() as u32).max(1);
    let small = image::imageops::resize(&crop, mw * 4, MENUBAR_PX * 4, image::imageops::FilterType::Lanczos3);
    let small = image::imageops::resize(&small, mw, MENUBAR_PX, image::imageops::FilterType::Triangle);
    // Brightness of the white-on-black artwork becomes the icon's opacity.
    let mut out = image::RgbaImage::new(mw, MENUBAR_PX);
    for (x, y, p) in small.enumerate_pixels() {
        let a = p[0].max(p[1]).max(p[2]);
        out.put_pixel(x, y, image::Rgba([rgb[0], rgb[1], rgb[2], a]));
    }
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(out).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).ok()?;
    Some((png, mw, MENUBAR_PX))
}

/// Decodes the mark once (it is 4096 px; the menu bar and the Dock both start from it).
fn mark_image() -> Option<&'static image::RgbaImage> {
    static MARK: std::sync::OnceLock<Option<image::RgbaImage>> = std::sync::OnceLock::new();
    MARK.get_or_init(|| image::load_from_memory_with_format(MARK_PNG, image::ImageFormat::Png).ok().map(|i| i.to_rgba8())).as_ref()
}

/// The Dock icon: the mark's own square (white "k." on black, as drawn) on the macOS icon
/// grid: an 824 px rounded square centred in 1024, the rest transparent.
pub fn icon() -> Option<Arc<egui::IconData>> {
    let img = mark_image()?;
    let size = 1024u32;
    let shape = 824u32;
    let inset = (size - shape) / 2;
    let radius = shape as f32 * 0.225;
    let art = image::imageops::resize(img, shape, shape, image::imageops::FilterType::CatmullRom);
    let mut out = image::RgbaImage::new(size, size);
    for y in 0..shape {
        for x in 0..shape {
            // Coverage of the rounded square at this pixel, antialiased over one pixel.
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let cx = fx.clamp(radius, shape as f32 - radius);
            let cy = fy.clamp(radius, shape as f32 - radius);
            let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() - radius;
            let a = (0.5 - d).clamp(0.0, 1.0);
            if a <= 0.0 {
                continue;
            }
            let p = art.get_pixel(x, y);
            out.put_pixel(inset + x, inset + y, image::Rgba([p[0], p[1], p[2], (a * 255.0).round() as u8]));
        }
    }
    Some(Arc::new(egui::IconData { rgba: out.into_raw(), width: size, height: size }))
}

/// The mark for the Welcome window, `px` square: white on black in the dark appearance,
/// black on white in the light one.
pub fn logo_image(light: bool, px: u32) -> Option<egui::ColorImage> {
    let decoded;
    let img = if light {
        decoded = image::load_from_memory_with_format(MARK_LIGHT_PNG, image::ImageFormat::Png).ok()?.to_rgba8();
        &decoded
    } else {
        mark_image()?
    };
    let small = image::imageops::resize(img, px, px, image::imageops::FilterType::CatmullRom);
    Some(egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], small.as_raw()))
}

#[cfg(test)]
mod tests {
    /// Writes the two menu bar icon versions into assets/ (black for the light menu bar,
    /// white for the dark one).
    #[test]
    fn write_menubar_icons() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        for (name, rgb) in [("menubar-icon-light.png", [0, 0, 0]), ("menubar-icon-dark.png", [255, 255, 255])] {
            let (png, w, h) = super::menubar_icon_png(rgb).expect("icon");
            assert_eq!(h, 28);
            assert!(w > 16 && w < 50);
            std::fs::write(dir.join(name), png).unwrap();
        }
    }

    /// The Dock icon keeps its corners transparent and the mark's black ground inside.
    #[test]
    fn dock_icon() {
        let icon = super::icon().expect("icon");
        assert_eq!((icon.width, icon.height), (1024, 1024));
        let px = |x: usize, y: usize| &icon.rgba[(y * 1024 + x) * 4..(y * 1024 + x) * 4 + 4];
        assert_eq!(px(0, 0)[3], 0);
        assert_eq!(px(120, 120)[3], 0);
        assert_eq!(px(512, 110), &[0, 0, 0, 255]);
        if let Ok(dir) = std::env::var("KERNSHIP_ICON_PREVIEW") {
            let img = image::RgbaImage::from_raw(1024, 1024, icon.rgba.clone()).unwrap();
            img.save(std::path::Path::new(&dir).join("dock-icon.png")).unwrap();
        }
    }
}
