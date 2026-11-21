//! Icon fonts as icon sets (not fonts): every icon with its name, ready to copy, save or drag
//! out as SVG in the Icon Fonts view.
//!
//! Where sets come from (Kernship's own folder only):
//! - free icon sets downloaded on request with their names (FREE_SETS, Google's Material Icons);
//! - icon fonts dropped on the Icon Fonts view (.ttf/.otf, optionally with a .css).
use crate::google;
use crate::store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IconGlyph {
    pub name: String,
    pub ch: char,
}

#[derive(Clone, Debug)]
pub struct IconSet {
    pub id: String,
    pub name: String,
    pub path: Option<PathBuf>,
    pub index: u32,
    pub glyphs: Vec<IconGlyph>,
    /// Google set not downloaded yet.
    pub remote: Option<RemoteSet>,
}

#[derive(Clone, Debug)]
pub struct RemoteSet {
    pub family: String,
    pub variant: String,
    pub codepoints_url: String,
    /// Free sets from their published packages: the font file and its stylesheet of names.
    pub font_url: Option<String>,
    pub css_url: Option<String>,
}

const CDN: &str = "https://cdn.jsdelivr.net/npm/";

/// Free, openly licensed icon sets: (name, font path, stylesheet path) on the npm CDN.
pub const FREE_SETS: [(&str, &str, &str); 14] = [
    ("Font Awesome Solid", "@fortawesome/fontawesome-free@6.4.2/webfonts/fa-solid-900.ttf", "@fortawesome/fontawesome-free@6.4.2/css/all.css"),
    ("Font Awesome Regular", "@fortawesome/fontawesome-free@6.4.2/webfonts/fa-regular-400.ttf", "@fortawesome/fontawesome-free@6.4.2/css/all.css"),
    ("Font Awesome Brands", "@fortawesome/fontawesome-free@6.4.2/webfonts/fa-brands-400.ttf", "@fortawesome/fontawesome-free@6.4.2/css/all.css"),
    ("Remix Icon", "remixicon@4.2.0/fonts/remixicon.ttf", "remixicon@4.2.0/fonts/remixicon.css"),
    ("Boxicons", "boxicons@2.1.4/fonts/boxicons.ttf", "boxicons@2.1.4/css/boxicons.css"),
    ("Phosphor", "@phosphor-icons/web@2.1.1/src/regular/Phosphor.ttf", "@phosphor-icons/web@2.1.1/src/regular/style.css"),
    ("Phosphor Bold", "@phosphor-icons/web@2.1.1/src/bold/Phosphor-Bold.ttf", "@phosphor-icons/web@2.1.1/src/bold/style.css"),
    ("Phosphor Fill", "@phosphor-icons/web@2.1.1/src/fill/Phosphor-Fill.ttf", "@phosphor-icons/web@2.1.1/src/fill/style.css"),
    ("Ionicons", "ionicons@4.6.3/dist/fonts/ionicons.ttf", "ionicons@4.6.3/dist/css/ionicons.css"),
    ("Tabler Icons", "@tabler/icons-webfont@2.47.0/fonts/tabler-icons.ttf", "@tabler/icons-webfont@2.47.0/tabler-icons.css"),
    ("Typicons", "typicons.font@2.1.2/src/font/typicons.ttf", "typicons.font@2.1.2/src/font/typicons.css"),
    ("Weather Icons", "weather-icons@1.3.2/font/weathericons-regular-webfont.ttf", "weather-icons@1.3.2/css/weather-icons.css"),
    ("Devicon", "devicon@2.16.0/fonts/devicon.ttf", "devicon@2.16.0/devicon.min.css"),
    ("Material Design Icons", "@mdi/font@7.4.47/fonts/materialdesignicons-webfont.ttf", "@mdi/font@7.4.47/css/materialdesignicons.css"),
];

/// Class prefixes the sets put before every icon name.
const PREFIXES: [&str; 16] = ["icon-", "fa-", "ri-", "bxs-", "bxl-", "bx-", "ph-", "ion-", "ti-", "typcn-", "wi-", "devicon-", "la-", "mdi-", "lar-", "las-"];

/// Google's Material icon families and the name lists published with them.
pub const GOOGLE_SETS: [(&str, &str); 5] = [
    ("Material Icons", "https://raw.githubusercontent.com/google/material-design-icons/master/font/MaterialIcons-Regular.codepoints"),
    ("Material Icons Outlined", "https://raw.githubusercontent.com/google/material-design-icons/master/font/MaterialIconsOutlined-Regular.codepoints"),
    ("Material Icons Round", "https://raw.githubusercontent.com/google/material-design-icons/master/font/MaterialIconsRound-Regular.codepoints"),
    ("Material Icons Sharp", "https://raw.githubusercontent.com/google/material-design-icons/master/font/MaterialIconsSharp-Regular.codepoints"),
    ("Material Icons Two Tone", "https://raw.githubusercontent.com/google/material-design-icons/master/font/MaterialIconsTwoTone-Regular.codepoints"),
];

pub fn own_dir() -> PathBuf {
    let d = store::libraries_root().join(format!("Icon Fonts.{}", store::LIBRARY_EXT)).join("fonts");
    let _ = std::fs::create_dir_all(&d);
    d
}

/// `.name:before { content: "\e900" }` pairs from an icon font's stylesheet.
pub fn parse_css(css: &str) -> Vec<IconGlyph> {
    let mut out = Vec::new();
    for block in css.split('}') {
        let Some(sel_end) = block.find('{') else { continue };
        let selectors = &block[..sel_end];
        let body = &block[sel_end + 1..];
        let Some(c) = body.find("content") else { continue };
        let rest = &body[c..];
        let Some(q1) = rest.find(['"', '\'']) else { continue };
        let rest = &rest[q1 + 1..];
        let Some(q2) = rest.find(['"', '\'']) else { continue };
        let val = &rest[..q2];
        let ch = if let Some(hex) = val.strip_prefix('\\') {
            u32::from_str_radix(hex.trim(), 16).ok().and_then(char::from_u32)
        } else {
            val.chars().next()
        };
        let Some(ch) = ch else { continue };
        for sel in selectors.split(',') {
            let sel = sel.trim();
            let class = sel.split(':').next().unwrap_or("").trim();
            // ".ph.ph-acorn" → "ph-acorn"; then the set's prefix comes off.
            let class = class.rsplit('.').next().unwrap_or("").trim();
            let name = PREFIXES.iter().find_map(|p| class.strip_prefix(p)).unwrap_or(class);
            if !name.is_empty() && !name.contains(' ') {
                out.push(IconGlyph { name: name.to_string(), ch });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out.dedup_by(|a, b| a.name == b.name);
    out
}

/// `name codepoint` lines (Material Icons .codepoints files).
pub fn parse_codepoints(text: &str) -> Vec<IconGlyph> {
    let mut out: Vec<IconGlyph> = text
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let name = it.next()?;
            let cp = u32::from_str_radix(it.next()?, 16).ok()?;
            Some(IconGlyph { name: name.to_string(), ch: char::from_u32(cp)? })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out.dedup_by(|a, b| a.name == b.name);
    out
}

/// Icons straight from the font: mapped characters in the private-use area (or all symbols),
/// named by the font's glyph names when it has them.
pub fn glyphs_from_font(data: &[u8], index: u32) -> Vec<IconGlyph> {
    let Ok(face) = ttf_parser::Face::parse(data, index) else { return vec![] };
    let mut map: Vec<(u32, ttf_parser::GlyphId)> = Vec::new();
    if let Some(cmap) = face.tables().cmap {
        for st in cmap.subtables {
            if !st.is_unicode() {
                continue;
            }
            st.codepoints(|c| {
                if let Some(g) = st.glyph_index(c) {
                    map.push((c, g));
                }
            });
        }
    }
    map.sort();
    map.dedup_by_key(|x| x.0);
    let pua: Vec<_> = map.iter().copied().filter(|(c, _)| (0xE000..=0xF8FF).contains(c) || *c >= 0xF0000).collect();
    let list = if pua.len() > 8 { pua } else { map.into_iter().filter(|(c, _)| *c > 0x20).collect() };
    list.into_iter()
        .filter_map(|(c, g)| {
            let ch = char::from_u32(c)?;
            let name = face
                .glyph_name(g)
                .filter(|n| !n.starts_with("uni") && !n.starts_with("glyph") && *n != ".notdef")
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("u{c:04X}"));
            Some(IconGlyph { name, ch })
        })
        .collect()
}

fn font_files(dir: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file() && crate::scan::is_font_file(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn set_from_file(path: &Path, id_prefix: &str) -> Option<IconSet> {
    let data = std::fs::read(path).ok()?;
    let css = path.with_extension("css");
    let codepoints = path.with_extension("codepoints");
    let glyphs = if let Ok(t) = std::fs::read_to_string(&css) {
        parse_css(&t)
    } else if let Ok(t) = std::fs::read_to_string(&codepoints) {
        parse_codepoints(&t)
    } else {
        glyphs_from_font(&data, 0)
    };
    let face = ttf_parser::Face::parse(&data, 0).ok()?;
    // One stylesheet can serve several fonts (Font Awesome): keep what this font draws, once each.
    let mut seen_ch = std::collections::HashSet::new();
    let glyphs: Vec<IconGlyph> = glyphs.into_iter().filter(|g| face.glyph_index(g.ch).is_some() && seen_ch.insert(g.ch)).collect();
    if glyphs.is_empty() {
        return None;
    }
    let family = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::FAMILY && n.to_string().is_some())
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().to_string());
    Some(IconSet {
        id: format!("{id_prefix}:{}", path.display()),
        name: family,
        path: Some(path.to_path_buf()),
        index: 0,
        glyphs,
        remote: None,
    })
}

/// Finds every icon set on this computer, plus the Google sets that can be downloaded.
pub fn discover() -> Vec<IconSet> {
    let mut sets: Vec<IconSet> = Vec::new();
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut add = |s: IconSet, sets: &mut Vec<IconSet>| {
        if seen.insert(s.name.to_lowercase(), ()).is_none() {
            sets.push(s);
        }
    };
    for f in font_files(&own_dir()) {
        if let Some(s) = set_from_file(&f, "own") {
            add(s, &mut sets);
        }
    }
    for (family, url) in GOOGLE_SETS {
        let key = family.to_lowercase();
        if sets.iter().any(|s| s.name.to_lowercase() == key) {
            continue;
        }
        sets.push(IconSet {
            id: format!("google:{family}"),
            name: family.to_string(),
            path: None,
            index: 0,
            glyphs: vec![],
            remote: Some(RemoteSet { family: family.to_string(), variant: "400".into(), codepoints_url: url.to_string(), font_url: None, css_url: None }),
        });
    }
    for (name, font, css) in FREE_SETS {
        let file = font.rsplit('/').next().unwrap_or(font);
        if let Some(existing) = sets.iter_mut().find(|s| s.path.as_ref().and_then(|p| p.file_name()).map(|f| f.to_string_lossy() == file).unwrap_or(false)) {
            existing.name = name.to_string();
            continue;
        }
        sets.push(IconSet {
            id: format!("free:{name}"),
            name: name.to_string(),
            path: None,
            index: 0,
            glyphs: vec![],
            remote: Some(RemoteSet {
                family: name.to_string(),
                variant: String::new(),
                codepoints_url: String::new(),
                font_url: Some(format!("{CDN}{font}")),
                css_url: Some(format!("{CDN}{css}")),
            }),
        });
    }
    sets.sort_by_key(|a| a.name.to_lowercase());
    sets
}

/// Downloads a Google icon family and its names into Kernship's Icon Fonts library.
pub fn download(remote: &RemoteSet) -> Result<PathBuf, String> {
    let dir = own_dir();
    if let (Some(font_url), Some(css_url)) = (&remote.font_url, &remote.css_url) {
        let file = font_url.rsplit('/').next().unwrap_or("icons.ttf");
        let font = dir.join(file);
        if !font.exists() && !google::fetch(font_url, &font, 600) {
            return Err(format!("Couldn't download {}", remote.family));
        }
        let css = font.with_extension("css");
        if !css.exists() {
            let _ = google::fetch(css_url, &css, 120);
        }
        return Ok(font);
    }
    let font = google::ensure_file(&dir, &remote.family, &remote.variant, std::slice::from_ref(&remote.variant))
        .ok_or_else(|| format!("Couldn't download {}", remote.family))?;
    let cp = font.with_extension("codepoints");
    if !cp.exists() {
        let _ = google::fetch(&remote.codepoints_url, &cp, 120);
    }
    Ok(font)
}

struct PathSink {
    d: String,
    sx: f32,
    x0: f32,
    y1: f32,
}

impl PathSink {
    fn p(&self, x: f32, y: f32) -> String {
        let px = (x - self.x0) * self.sx;
        let py = (self.y1 - y) * self.sx;
        format!("{} {}", trim(px), trim(py))
    }
}

fn trim(v: f32) -> String {
    let s = format!("{:.2}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s == "-0" { "0".into() } else { s }
}

impl ttf_parser::OutlineBuilder for PathSink {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.d.push_str(&format!("M{p}"));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.d.push_str(&format!("L{p}"));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let a = self.p(x1, y1);
        let b = self.p(x, y);
        self.d.push_str(&format!("Q{a} {b}"));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let a = self.p(x1, y1);
        let b = self.p(x2, y2);
        let c = self.p(x, y);
        self.d.push_str(&format!("C{a} {b} {c}"));
    }
    fn close(&mut self) {
        self.d.push('Z');
    }
}

/// The icon as SVG: the outline moved to the icon's top-left corner and turned
/// upright, at `size` points.
pub fn svg(font_data: &[u8], index: u32, ch: char, size: f32, color: &str) -> Option<String> {
    let face = ttf_parser::Face::parse(font_data, index).ok()?;
    let gid = face.glyph_index(ch)?;
    let upem = face.units_per_em() as f32;
    let bbox = face.glyph_bounding_box(gid)?;
    let scale = size / upem;
    let mut sink = PathSink { d: String::new(), sx: scale, x0: bbox.x_min as f32, y1: bbox.y_max as f32 };
    face.outline_glyph(gid, &mut sink)?;
    let w = (bbox.x_max - bbox.x_min) as f32 * scale;
    let h = (bbox.y_max - bbox.y_min) as f32 * scale;
    Some(format!(
        "<svg style=\"\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\n<path d=\"{}\" style=\"fill:{color}\"/>\n</svg>",
        trim(w),
        trim(h),
        trim(w),
        trim(h),
        sink.d
    ))
}

/// A file name that is safe on disk ("<name>.svg").
pub fn file_name(name: &str) -> String {
    let clean: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    format!("{clean}.svg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_set_names() {
        let tmp = std::env::temp_dir().join("kernship-icon-test");
        let _ = std::fs::create_dir_all(&tmp);
        for (name, font, css) in [FREE_SETS[5], FREE_SETS[0]] {
            let f = tmp.join(font.rsplit('/').next().unwrap());
            let c = f.with_extension("css");
            if !f.exists() {
                google::fetch(&format!("{CDN}{font}"), &f, 120);
            }
            if !c.exists() {
                google::fetch(&format!("{CDN}{css}"), &c, 120);
            }
            let set = set_from_file(&f, "t").expect("set");
            println!("{name}: {} icons, first {:?}", set.glyphs.len(), &set.glyphs.iter().take(3).map(|g| g.name.clone()).collect::<Vec<_>>());
            assert!(set.glyphs.len() > 100);
            assert!(!set.glyphs[0].name.starts_with('u'));
        }
    }

    #[test]
    fn css_names_and_svg() {
        let css = ".activity:before {\n content: \"\\e900\"\n}\n.airplay:before { content: \"\\e901\" }";
        let g = parse_css(css);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].name, "activity");
        assert_eq!(g[0].ch as u32, 0xE900);
        let font = Path::new("/System/Library/Fonts/Supplemental/Arial.ttf");
        if let Ok(data) = std::fs::read(font) {
            let s = svg(&data, 0, 'A', 32.0, "#000000").expect("svg");
            assert!(s.starts_with("<svg") && s.contains("<path d=\"M"));
            println!("{s}");
        }
    }
}
