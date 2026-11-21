//! Font properties for the search panel's Properties and Language filters, measured from the font
//! file:
//! - x-height ratio = xHeight / ascent (0.5 when the ascent is 0);
//!   Low ≤ 0.4 < Regular < 0.6 ≤ High.
//! - old-style ratio = (ascent − y₂) / (ascent − y₉) from the bounding boxes of "2" and "9" (1.0 when
//!   missing); Old Style ≤ 0.9 < Lining.
//! - contrast = |thick − thin| / mean of the "o" strokes;
//!   Low ≤ 0.25 < Regular ≤ 0.55 < High.
//! - width from OS/2 usWidthClass: < 4 Narrow, > 5 Expanded.
//! - colour: COLR, sbix, CBDT or SVG glyphs (`font.hasColorGlyphs`).
//! - languages: the characters each language needs are all in the character map
//!   (`font.supportedLanguages LIKE '%lang,%'`).
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FontProps {
    pub xheight: f32,
    pub oldstyle: f32,
    pub contrast: f32,
    pub width_class: u16,
    pub color: bool,
    /// Bit i set = supports LANGUAGES[i].
    pub langs: u32,
    /// false when only the catalog was known (no file to measure).
    pub measured: bool,
}

/// (code, name, characters the language needs, Google Fonts subset that covers it).
pub const LANGUAGES: [(&str, &str, &str, &str); 30] = [
    ("en", "English", "AZaz", "latin"),
    ("fr", "French", "éèàçœÉ", "latin"),
    ("de", "German", "äöüßÄ", "latin"),
    ("es", "Spanish", "ñ¿¡áÑ", "latin"),
    ("it", "Italian", "àèéìòù", "latin"),
    ("pt", "Portuguese", "ãõçâ", "latin"),
    ("nl", "Dutch", "éëïĳ", "latin"),
    ("sv", "Swedish", "åäöÅ", "latin"),
    ("ca", "Catalan", "àçèíl·", "latin"),
    ("cs", "Czech", "čřšžůě", "latin-ext"),
    ("hu", "Hungarian", "őűŐ", "latin-ext"),
    ("pl", "Polish", "ąęłńśźż", "latin-ext"),
    ("ro", "Romanian", "ăâșț", "latin-ext"),
    ("sl", "Slovenian", "čšžČ", "latin-ext"),
    ("lt", "Lithuanian", "ąčęėįšųūž", "latin-ext"),
    ("mt", "Maltese", "ċġħż", "latin-ext"),
    ("tr", "Turkish", "ğışİ", "latin-ext"),
    ("vi", "Vietnamese", "ơưạảấ", "vietnamese"),
    ("el", "Greek", "αβγΩ", "greek"),
    ("ru", "Russian", "абвЖЯ", "cyrillic"),
    ("hy", "Armenian", "աբգԱ", "armenian"),
    ("he", "Hebrew", "אבג", "hebrew"),
    ("ar", "Arabic", "ابت", "arabic"),
    ("deva", "Devanagari", "कखग", "devanagari"),
    ("bn", "Bengali", "অআক", "bengali"),
    ("gu", "Gujarati", "અઆક", "gujarati"),
    ("th", "Thai", "กขค", "thai"),
    ("ja", "Japanese", "あア日", "japanese"),
    ("ko", "Korean", "한글", "korean"),
    ("zh", "Chinese", "中文字", "chinese-simplified"),
];

/// What the Google Fonts catalog says, for styles not downloaded yet.
pub fn from_catalog(style: &str, subsets: &[String]) -> FontProps {
    let s = style.to_lowercase();
    let width_class = if ["condensed", "narrow", "compressed"].iter().any(|w| s.contains(w)) {
        3
    } else if ["expanded", "extended", "wide"].iter().any(|w| s.contains(w)) {
        7
    } else {
        5
    };
    let mut langs = 0u32;
    for (i, l) in LANGUAGES.iter().enumerate() {
        if subsets.iter().any(|x| x == l.3) {
            langs |= 1 << i;
        }
    }
    FontProps { width_class, langs, ..Default::default() }
}

pub fn measure(path: &Path, index: u32) -> Option<FontProps> {
    let data = std::fs::read(path).ok()?;
    let face = ttf_parser::Face::parse(&data, index).ok()?;
    let asc = face.ascender() as f32;
    let bbox = |c: char| face.glyph_index(c).and_then(|g| face.glyph_bounding_box(g));
    let xh = face.x_height().map(|v| v as f32).filter(|v| *v > 0.0).or_else(|| bbox('x').map(|b| b.y_max as f32)).unwrap_or(0.0);
    let xheight = if asc > 0.0 { xh / asc } else { 0.5 };
    let oldstyle = match (bbox('2'), bbox('9')) {
        (Some(two), Some(nine)) if asc > 0.0 && asc - nine.y_min as f32 != 0.0 => ((asc - two.y_min as f32) / (asc - nine.y_min as f32)).abs(),
        _ => 1.0,
    };
    let width_class = face.width().to_number();
    let raw = face.raw_face();
    let has = |t: &[u8; 4]| raw.table(ttf_parser::Tag::from_bytes(t)).is_some();
    let color = has(b"COLR") || has(b"sbix") || has(b"CBDT") || has(b"SVG ");
    let mut langs = 0u32;
    for (i, l) in LANGUAGES.iter().enumerate() {
        if l.2.chars().filter(|c| *c != '·').all(|c| face.glyph_index(c).is_some()) {
            langs |= 1 << i;
        }
    }
    Some(FontProps { xheight, oldstyle, contrast: contrast(&data, index), width_class, color, langs, measured: true })
}

/// Stroke contrast of "o": the side stroke (scanning the middle row) against the top stroke
/// (scanning the middle column) of the glyph drawn 160 px tall.
fn contrast(data: &[u8], index: u32) -> f32 {
    use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
    let Ok(font) = FontRef::try_from_slice_and_index(data, index) else { return 0.0 };
    let id = font.glyph_id('o');
    if id.0 == 0 {
        return 0.0;
    }
    let scaled = font.as_scaled(PxScale::from(160.0));
    let Some(g) = scaled.outline_glyph(id.with_scale(PxScale::from(160.0))) else { return 0.0 };
    let b = g.px_bounds();
    let (w, h) = (b.width().ceil() as usize + 1, b.height().ceil() as usize + 1);
    if w < 6 || h < 6 {
        return 0.0;
    }
    let mut cov = vec![0f32; w * h];
    g.draw(|x, y, c| {
        let (x, y) = (x as usize, y as usize);
        if x < w && y < h {
            cov[y * w + x] = c;
        }
    });
    let run = |cells: &mut dyn Iterator<Item = f32>| -> f32 {
        // Length of the first inked run.
        let mut started = false;
        let mut len = 0.0;
        for c in cells {
            if c > 0.5 {
                started = true;
                len += 1.0;
            } else if started {
                break;
            }
        }
        len
    };
    let side = run(&mut (0..w).map(|x| cov[(h / 2) * w + x]));
    let top = run(&mut (0..h).map(|y| cov[y * w + w / 2]));
    if side <= 0.0 || top <= 0.0 {
        return 0.0;
    }
    (side - top).abs() / ((side + top) * 0.5)
}

pub fn cache_file() -> std::path::PathBuf {
    crate::store::data_dir().join("props.json")
}

pub fn load_cache() -> HashMap<String, FontProps> {
    std::fs::read_to_string(cache_file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn save_cache(map: &HashMap<String, FontProps>) {
    if let Ok(t) = serde_json::to_string(map) {
        let p = cache_file();
        let tmp = p.with_extension("tmp");
        if std::fs::write(&tmp, t).is_ok() {
            let _ = std::fs::rename(tmp, p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn measures_system_fonts() {
        for (p, i) in [
            ("/System/Library/Fonts/Supplemental/Georgia.ttf", 0),
            ("/System/Library/Fonts/Supplemental/Times New Roman.ttf", 0),
            ("/System/Library/Fonts/Supplemental/Arial.ttf", 0),
            ("/System/Library/Fonts/Supplemental/Didot.ttc", 0),
            ("/System/Library/Fonts/Apple Color Emoji.ttc", 0),
            ("/System/Library/Fonts/Supplemental/Arial Narrow.ttf", 0),
        ] {
            if let Some(m) = measure(Path::new(p), i) {
                println!("{p}: xh {:.2} old {:.2} contrast {:.2} width {} color {} langs {:b}", m.xheight, m.oldstyle, m.contrast, m.width_class, m.color, m.langs);
            }
        }
    }
}
