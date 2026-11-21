//! Google Fonts: the public family catalog, per-style file addresses and downloads.
//!
//! Catalog: https://fonts.google.com/metadata/fonts (families, styles, category, designers,
//! popularity). Files: the CSS2 API answers a plain client with one TrueType file per style.
use crate::model::{Category, FontFace, Source};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub const CATALOG_URL: &str = "https://fonts.google.com/metadata/fonts";
pub const CSS_URL: &str = "https://fonts.googleapis.com/css2";
const AGENT: &str = "Wget/1.21";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GFamily {
    pub family: String,
    pub category: String,
    pub designers: Vec<String>,
    pub popularity: u32,
    pub date_added: String,
    pub variants: Vec<String>,
    pub variable: bool,
    pub subsets: Vec<String>,
}

pub fn fetch(url: &str, dest: &Path, timeout: u32) -> bool {
    if let Some(p) = dest.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let tmp = dest.with_extension("part");
    let ok = Command::new("curl")
        .args(["-sSfL", "--max-time", &timeout.to_string(), "-A", AGENT, "-o"])
        .arg(&tmp)
        .arg(url)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        std::fs::rename(&tmp, dest).is_ok()
    } else {
        let _ = std::fs::remove_file(&tmp);
        false
    }
}

fn fetch_text(url: &str) -> Option<String> {
    let out = Command::new("curl")
        .args(["-sSfL", "--max-time", "30", "-A", AGENT])
        .arg(url)
        .output()
        .ok()?;
    if out.status.success() {
        String::from_utf8(out.stdout).ok()
    } else {
        None
    }
}

/// Loads the catalog from the local copy, refreshing it from Google when older than a week.
pub fn load_catalog(dir: &Path, force: bool, max_age_days: u32) -> Result<Vec<GFamily>, String> {
    let file = dir.join("catalog.json");
    let fresh = std::fs::metadata(&file)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|e| e.as_secs() < (max_age_days.max(1) as u64) * 86400)
        .unwrap_or(false);
    if force || !fresh {
        // 300 s for the catalog: it is large.
        let _ = fetch(CATALOG_URL, &file, 300);
    }
    let text = std::fs::read_to_string(&file).map_err(|_| "Couldn't reach Google Fonts".to_string())?;
    parse_catalog(&text)
}

pub fn parse_catalog(text: &str) -> Result<Vec<GFamily>, String> {
    let start = text.find('{').ok_or("catalog is empty")?;
    let v: serde_json::Value = serde_json::from_str(&text[start..]).map_err(|e| e.to_string())?;
    let list = v["familyMetadataList"].as_array().ok_or("no families in catalog")?;
    let mut out = Vec::with_capacity(list.len());
    for f in list {
        let family = f["family"].as_str().unwrap_or_default().to_string();
        if family.is_empty() {
            continue;
        }
        let mut variants: Vec<String> = f["fonts"]
            .as_object()
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        variants.sort_by_key(|v| variant_sort(v));
        out.push(GFamily {
            family,
            category: f["category"].as_str().unwrap_or("").to_string(),
            designers: f["designers"]
                .as_array()
                .map(|a| a.iter().filter_map(|d| d.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            popularity: f["popularity"].as_u64().unwrap_or(0) as u32,
            date_added: f["dateAdded"].as_str().unwrap_or("").to_string(),
            variants,
            variable: f["axes"].as_array().map(|a| !a.is_empty()).unwrap_or(false),
            subsets: f["subsets"]
                .as_array()
                .map(|a| a.iter().filter_map(|d| d.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        });
    }
    out.sort_by_key(|a| a.family.to_lowercase());
    Ok(out)
}

pub fn variant_parts(v: &str) -> (u16, bool) {
    let italic = v.ends_with('i');
    let w = v.trim_end_matches('i').parse().unwrap_or(400);
    (w, italic)
}

fn variant_sort(v: &str) -> (bool, u16) {
    let (w, i) = variant_parts(v);
    (i, w)
}

pub fn style_name(v: &str) -> String {
    let (w, italic) = variant_parts(v);
    let base = match w {
        100 => "Thin",
        200 => "ExtraLight",
        300 => "Light",
        400 => "Regular",
        500 => "Medium",
        600 => "SemiBold",
        700 => "Bold",
        800 => "ExtraBold",
        900 => "Black",
        _ => "",
    };
    match (base, italic) {
        ("", false) => format!("W{w}"),
        ("", true) => format!("W{w} Italic"),
        ("Regular", true) => "Italic".into(),
        (b, true) => format!("{b} Italic"),
        (b, false) => b.into(),
    }
}

pub fn face_id(family: &str, variant: &str) -> String {
    format!("gf:{family}:{variant}")
}

/// One `FontFace` per style.
pub fn faces(cat: &[GFamily]) -> Vec<FontFace> {
    let mut out = Vec::new();
    for f in cat {
        let category = Category::from_google(&f.category);
        let icon = f.family.contains("Icons") || f.family.starts_with("Material Symbols");
        for v in &f.variants {
            let (w, italic) = variant_parts(v);
            let style = style_name(v);
            out.push(FontFace {
                id: face_id(&f.family, v),
                family: f.family.clone(),
                style: style.clone(),
                full_name: format!("{} {}", f.family, style),
                ps_name: format!("{}-{}", f.family.replace(' ', ""), style.replace(' ', "")),
                path: None,
                index: 0,
                weight: w,
                italic,
                source: Source::Google,
                category: if icon { Category::Symbol } else { category },
                gf_variant: Some(v.clone()),
                version: String::new(),
                designer: f.designers.join(", "),
                copyright: String::new(),
                license: "SIL Open Font License / Apache 2.0 (Google Fonts)".into(),
                manufacturer: "Google Fonts".into(),
                glyph_count: 0,
                monospaced: category == Category::Monospace,
                icon_font: icon,
                variable: f.variable,
                popularity: f.popularity,
                date_added: f.date_added.clone(),
                added: 0,
            });
        }
    }
    out
}

fn url_cache() -> &'static Mutex<HashMap<String, HashMap<String, String>>> {
    static C: OnceLock<Mutex<HashMap<String, HashMap<String, String>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn css_query(family: &str, variants: &[String]) -> String {
    let mut tuples: Vec<(u8, u16)> = variants
        .iter()
        .map(|v| {
            let (w, i) = variant_parts(v);
            (i as u8, w)
        })
        .collect();
    tuples.sort();
    tuples.dedup();
    let spec = tuples.iter().map(|(i, w)| format!("{i},{w}")).collect::<Vec<_>>().join(";");
    format!("{CSS_URL}?family={}:ital,wght@{spec}", family.replace(' ', "+"))
}

/// Asks the CSS2 API for each style's TrueType address.
pub fn resolve_urls(family: &str, variants: &[String]) -> HashMap<String, String> {
    if let Some(m) = url_cache().lock().get(family) {
        return m.clone();
    }
    let mut map = HashMap::new();
    let text = fetch_text(&css_query(family, variants))
        .or_else(|| fetch_text(&format!("{CSS_URL}?family={}", family.replace(' ', "+"))));
    if let Some(css) = text {
        for block in css.split("@font-face").skip(1) {
            let italic = block.contains("font-style: italic");
            let weight = block
                .split("font-weight:")
                .nth(1)
                .and_then(|s| s.trim().split(';').next())
                .and_then(|s| s.trim().split(' ').next().map(|x| x.to_string()))
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(400);
            if let Some(url) = block.split("url(").nth(1).and_then(|s| s.split(')').next()) {
                let key = format!("{weight}{}", if italic { "i" } else { "" });
                map.insert(key, url.to_string());
            }
        }
    }
    if !map.is_empty() {
        url_cache().lock().insert(family.to_string(), map.clone());
    }
    map
}

pub fn file_name(family: &str, variant: &str) -> String {
    format!("{}-{}.ttf", family.replace(' ', ""), style_name(variant).replace(' ', ""))
}

/// Makes sure the style's file is on disk in `dir`, downloading it when needed.
pub fn ensure_file(dir: &Path, family: &str, variant: &str, all_variants: &[String]) -> Option<PathBuf> {
    let dest = dir.join(file_name(family, variant));
    if dest.exists() {
        return Some(dest);
    }
    let urls = resolve_urls(family, all_variants);
    let url = urls.get(variant).or_else(|| {
        // A variable family answers with the nearest weight; take the closest one.
        let (w, i) = variant_parts(variant);
        urls.iter()
            .filter(|(k, _)| variant_parts(k).1 == i)
            .min_by_key(|(k, _)| (variant_parts(k).0 as i32 - w as i32).abs())
            .map(|(_, u)| u)
    })?;
    // 600 s per file: some families are large.
    if fetch(url, &dest, 600) {
        Some(dest)
    } else {
        None
    }
}

/// The `<link>` and CSS a designer pastes into a web page.
pub fn embed_code(family: &str, variants: &[String]) -> String {
    format!(
        "<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n<link href=\"{}&display=swap\" rel=\"stylesheet\">\n\nfont-family: '{}';",
        css_query(family, variants),
        family
    )
}
