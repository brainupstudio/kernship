//! Reads font files (TTF, OTF, TTC, OTC, DFONT where parseable) into `FontFace` records.
use crate::model::{Category, FontFace, Source};
use std::path::{Path, PathBuf};
use ttf_parser::{name_id, Face, Tag};

pub const FONT_EXTENSIONS: [&str; 5] = ["ttf", "otf", "ttc", "otc", "dfont"];

pub fn is_font_file(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| FONT_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn system_font_dirs() -> Vec<PathBuf> {
    let mut v = vec![
        PathBuf::from("/System/Library/Fonts"),
        PathBuf::from("/Library/Fonts"),
    ];
    if let Some(h) = dirs::home_dir() {
        v.push(h.join("Library/Fonts"));
    }
    #[cfg(target_os = "windows")]
    {
        v = vec![PathBuf::from("C:/Windows/Fonts")];
        if let Some(l) = dirs::data_local_dir() {
            v.push(l.join("Microsoft/Windows/Fonts"));
        }
    }
    #[cfg(target_os = "linux")]
    {
        v = vec![PathBuf::from("/usr/share/fonts"), PathBuf::from("/usr/local/share/fonts")];
        if let Some(h) = dirs::home_dir() {
            v.push(h.join(".local/share/fonts"));
            v.push(h.join(".fonts"));
        }
    }
    v
}

fn name(face: &Face, id: u16) -> Option<String> {
    let mut fallback = None;
    for n in face.names() {
        if n.name_id == id {
            if let Some(s) = n.to_string() {
                if n.language_id == 0x0409 || n.platform_id == ttf_parser::PlatformId::Unicode {
                    return Some(s);
                }
                if fallback.is_none() {
                    fallback = Some(s);
                }
            }
        }
    }
    fallback
}

fn classify(face: &Face, family: &str) -> (Category, bool) {
    let lower = family.to_lowercase();
    let mut pua = 0usize;
    let mut total = 0usize;
    if let Some(cmap) = face.tables().cmap {
        for st in cmap.subtables {
            if !st.is_unicode() {
                continue;
            }
            st.codepoints(|c| {
                total += 1;
                if (0xE000..=0xF8FF).contains(&c) || (0xF0000..=0xFFFFD).contains(&c) {
                    pua += 1;
                }
            });
            break;
        }
    }
    let icon = lower.contains("icon")
        || lower.contains("awesome")
        || lower.contains("material symbols")
        || (total > 20 && pua * 2 > total);
    if icon {
        return (Category::Symbol, true);
    }
    if face.is_monospaced() || lower.contains("mono") || lower.contains("code") {
        return (Category::Monospace, false);
    }
    if let Some(os2) = face.raw_face().table(Tag::from_bytes(b"OS/2")) {
        if os2.len() > 34 {
            let family_type = os2[32];
            let serif = os2[33];
            match family_type {
                3 => return (Category::Script, false),
                4 => return (Category::Display, false),
                5 => return (Category::Symbol, false),
                2 => {
                    if (11..=13).contains(&serif) {
                        return (Category::SansSerif, false);
                    } else if (6..=10).contains(&serif) {
                        return (Category::SlabSerif, false);
                    } else if (2..=5).contains(&serif) || serif == 14 || serif == 15 {
                        return (Category::Serif, false);
                    }
                }
                _ => {}
            }
        }
    }
    if lower.contains("sans") || lower.contains("grotesk") || lower.contains("gothic") {
        (Category::SansSerif, false)
    } else if lower.contains("serif") || lower.contains("roman") || lower.contains("garamond") {
        (Category::Serif, false)
    } else if lower.contains("script") || lower.contains("hand") {
        (Category::Script, false)
    } else if lower.contains("slab") {
        (Category::SlabSerif, false)
    } else if lower.contains("fraktur") || lower.contains("blackletter") {
        (Category::Blackletter, false)
    } else {
        (Category::Unknown, false)
    }
}

pub fn read_face(data: &[u8], index: u32, path: &Path, source: Source) -> Option<FontFace> {
    let face = Face::parse(data, index).ok()?;
    let family = name(&face, name_id::TYPOGRAPHIC_FAMILY).or_else(|| name(&face, name_id::FAMILY))?;
    if family.starts_with('.') {
        // Hidden system UI faces (".SF NS" etc.) are not listed.
        return None;
    }
    let style = name(&face, name_id::TYPOGRAPHIC_SUBFAMILY)
        .or_else(|| name(&face, name_id::SUBFAMILY))
        .unwrap_or_else(|| "Regular".into());
    let full = name(&face, name_id::FULL_NAME).unwrap_or_else(|| format!("{family} {style}"));
    let ps = name(&face, name_id::POST_SCRIPT_NAME).unwrap_or_else(|| full.replace(' ', ""));
    let (category, icon) = classify(&face, &family);
    let id = format!("{}#{}", path.display(), index);
    Some(FontFace {
        id,
        family,
        style,
        full_name: full,
        ps_name: ps,
        path: Some(path.to_path_buf()),
        index,
        weight: face.weight().to_number(),
        italic: face.is_italic() || face.is_oblique(),
        source,
        category,
        gf_variant: None,
        version: name(&face, name_id::VERSION).unwrap_or_default(),
        designer: name(&face, name_id::DESIGNER).unwrap_or_default(),
        copyright: name(&face, name_id::COPYRIGHT_NOTICE).unwrap_or_default(),
        license: name(&face, name_id::LICENSE).unwrap_or_default(),
        manufacturer: name(&face, name_id::MANUFACTURER).unwrap_or_default(),
        glyph_count: face.number_of_glyphs(),
        monospaced: face.is_monospaced(),
        icon_font: icon,
        variable: face.is_variable(),
        popularity: 0,
        date_added: String::new(),
        added: 0,
    })
}

pub fn read_file(path: &Path, source: Source) -> Vec<FontFace> {
    let Ok(data) = std::fs::read(path) else { return vec![] };
    let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
    (0..count)
        .filter_map(|i| read_face(&data, i, path, source.clone()))
        .collect()
}

pub fn scan_dirs(dirs: &[PathBuf], source: Source) -> Vec<FontFace> {
    let mut out = Vec::new();
    for d in dirs {
        for e in walkdir::WalkDir::new(d).follow_links(true).max_depth(6).into_iter().flatten() {
            if e.file_type().is_file() && is_font_file(e.path()) {
                out.extend(read_file(e.path(), source.clone()));
            }
        }
    }
    out
}
