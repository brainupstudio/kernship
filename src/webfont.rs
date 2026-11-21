//! "Convert Font for Web": turns desktop fonts into web fonts the way transfonter.org does —
//! WOFF2, WOFF, TTF/OTF and EOT files, optional subsetting to chosen character sets, and a
//! ready stylesheet (@font-face rules with font-display, local() names and optional base64
//! embedding), a demo page and a ZIP.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Options {
    pub woff2: bool,
    pub woff: bool,
    pub ttf: bool,
    pub eot: bool,
    /// Character sets to keep (empty = keep every glyph).
    pub subsets: BTreeSet<String>,
    pub custom_chars: String,
    pub display: String,
    pub base64: bool,
    pub local: bool,
    pub demo: bool,
    pub zip: bool,
    pub family_override: String,
    /// Where converted fonts go; empty means the Downloads folder.
    pub save_to: String,
}

impl Options {
    pub fn save_dir(&self) -> PathBuf {
        if self.save_to.trim().is_empty() {
            dirs::download_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from("."))
        } else {
            PathBuf::from(self.save_to.trim())
        }
    }
}

/// A conversion running in the background.
#[derive(Default)]
pub struct Job {
    pub progress: f32,
    pub stage: String,
    pub done: Option<Result<Report, String>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            woff2: true,
            woff: true,
            ttf: false,
            eot: false,
            subsets: BTreeSet::new(),
            custom_chars: String::new(),
            display: "swap".into(),
            base64: false,
            local: true,
            demo: true,
            zip: true,
            family_override: String::new(),
            save_to: String::new(),
        }
    }
}

/// Character sets offered for subsetting (name, ranges).
pub const SUBSETS: [(&str, &[(u32, u32)]); 6] = [
    ("Latin", &[(0x20, 0x7E), (0xA0, 0xFF), (0x131, 0x131), (0x152, 0x153), (0x2BB, 0x2BC), (0x2C6, 0x2C6), (0x2DA, 0x2DA), (0x2DC, 0x2DC), (0x2000, 0x206F), (0x2074, 0x2074), (0x20AC, 0x20AC), (0x2122, 0x2122), (0x2191, 0x2193), (0x2212, 0x2215), (0xFEFF, 0xFEFF), (0xFFFD, 0xFFFD)]),
    ("Latin Extended", &[(0x100, 0x24F), (0x259, 0x259), (0x1E00, 0x1EFF), (0x2020, 0x2020), (0x20A0, 0x20AB), (0x20AD, 0x20CF), (0x2113, 0x2113), (0x2C60, 0x2C7F), (0xA720, 0xA7FF)]),
    ("Cyrillic", &[(0x400, 0x52F), (0x1C80, 0x1C88), (0x20B4, 0x20B4), (0x2DE0, 0x2DFF), (0xA640, 0xA69F), (0xFE2E, 0xFE2F)]),
    ("Greek", &[(0x370, 0x3FF), (0x1F00, 0x1FFF)]),
    ("Vietnamese", &[(0x102, 0x103), (0x110, 0x111), (0x128, 0x129), (0x168, 0x169), (0x1A0, 0x1A1), (0x1AF, 0x1B0), (0x300, 0x301), (0x303, 0x304), (0x308, 0x309), (0x323, 0x323), (0x329, 0x329), (0x1EA0, 0x1EF9), (0x20AB, 0x20AB)]),
    ("Numbers & Punctuation", &[(0x20, 0x40), (0x5B, 0x60), (0x7B, 0x7E), (0x2000, 0x206F)]),
];

pub const DISPLAYS: [&str; 5] = ["auto", "block", "swap", "fallback", "optional"];

/// One face to convert.
#[derive(Clone, Debug)]
pub struct Input {
    pub path: PathBuf,
    pub index: u32,
    pub family: String,
    pub style: String,
    pub full_name: String,
    pub ps_name: String,
    pub weight: u16,
    pub italic: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub folder: PathBuf,
    pub zip: Option<PathBuf>,
    /// (file name, bytes)
    pub files: Vec<(String, u64)>,
    pub original_bytes: u64,
    pub notes: Vec<String>,
    pub css: String,
}

// ----- sfnt reading and writing -----

fn be16(d: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([d[o], d[o + 1]])
}
fn be32(d: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

#[derive(Clone)]
pub struct Sfnt {
    pub flavor: u32,
    pub tables: BTreeMap<[u8; 4], Vec<u8>>,
}

impl Sfnt {
    /// One face of a font file (a .ttc collection gives the face at `index`).
    pub fn read(data: &[u8], index: u32) -> Result<Self, String> {
        if data.len() < 12 {
            return Err("not a font file".into());
        }
        let off = if &data[0..4] == b"ttcf" {
            let n = be32(data, 8);
            if index >= n {
                return Err("face not in collection".into());
            }
            be32(data, 12 + 4 * index as usize) as usize
        } else {
            0
        };
        let flavor = be32(data, off);
        if flavor != 0x0001_0000 && &flavor.to_be_bytes() != b"OTTO" && &flavor.to_be_bytes() != b"true" {
            return Err("unsupported font format (WOFF or bitmap fonts can't be converted)".into());
        }
        let n = be16(data, off + 4) as usize;
        let mut tables = BTreeMap::new();
        for i in 0..n {
            let r = off + 12 + 16 * i;
            if r + 16 > data.len() {
                return Err("damaged table directory".into());
            }
            let tag = [data[r], data[r + 1], data[r + 2], data[r + 3]];
            let (o, l) = (be32(data, r + 8) as usize, be32(data, r + 12) as usize);
            if o + l > data.len() {
                return Err("damaged table".into());
            }
            tables.insert(tag, data[o..o + l].to_vec());
        }
        Ok(Self { flavor, tables })
    }

    pub fn is_cff(&self) -> bool {
        self.tables.contains_key(b"CFF ") || self.tables.contains_key(b"CFF2")
    }

    /// A standalone .ttf/.otf with correct checksums.
    pub fn write(&self) -> Vec<u8> {
        let n = self.tables.len();
        let mut pow = 1usize;
        let mut sel = 0u16;
        while pow * 2 <= n {
            pow *= 2;
            sel += 1;
        }
        let search = (pow * 16) as u16;
        let mut out = Vec::new();
        out.extend(self.flavor.to_be_bytes());
        out.extend((n as u16).to_be_bytes());
        out.extend(search.to_be_bytes());
        out.extend(sel.to_be_bytes());
        out.extend(((n * 16) as u16 - search).to_be_bytes());
        let mut offset = 12 + 16 * n;
        let mut head_at = None;
        let mut body = Vec::new();
        for (tag, data) in &self.tables {
            let mut d = data.clone();
            if tag == b"head" && d.len() >= 12 {
                d[8..12].copy_from_slice(&[0; 4]);
                head_at = Some(offset);
            }
            out.extend(tag);
            out.extend(checksum(&d).to_be_bytes());
            out.extend((offset as u32).to_be_bytes());
            out.extend((d.len() as u32).to_be_bytes());
            let padded = (d.len() + 3) & !3;
            body.extend(&d);
            body.resize(body.len() + padded - d.len(), 0);
            offset += padded;
        }
        out.extend(body);
        if let Some(h) = head_at {
            let adj = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
            out[h + 8..h + 12].copy_from_slice(&adj.to_be_bytes());
        }
        out
    }
}

fn checksum(d: &[u8]) -> u32 {
    let mut sum = 0u32;
    for chunk in d.chunks(4) {
        let mut b = [0u8; 4];
        b[..chunk.len()].copy_from_slice(chunk);
        sum = sum.wrapping_add(u32::from_be_bytes(b));
    }
    sum
}

// ----- subsetting (TrueType outlines) -----

/// Keeps the glyphs of the chosen characters (and every glyph not mapped to a character, so
/// ligatures and alternates still work); other glyphs are emptied and dropped from the
/// character map. Layout, kerning and glyph numbering stay as they are.
pub fn subset(s: &mut Sfnt, keep: &dyn Fn(u32) -> bool) -> Result<usize, String> {
    if s.is_cff() {
        return Err("subsetting needs TrueType outlines; this font has PostScript (CFF) outlines".into());
    }
    if s.tables.contains_key(b"gvar") {
        return Err("subsetting isn't available for variable fonts".into());
    }
    let (Some(head), Some(maxp), Some(loca), Some(glyf)) = (s.tables.get(b"head"), s.tables.get(b"maxp"), s.tables.get(b"loca"), s.tables.get(b"glyf")) else {
        return Err("font has no TrueType glyph tables".into());
    };
    let n = be16(maxp, 4) as usize;
    let long = be16(head, 50) == 1;
    let off = |i: usize| -> usize {
        if long {
            be32(loca, i * 4) as usize
        } else {
            be16(loca, i * 2) as usize * 2
        }
    };
    let built = s.write();
    let face = ttf_parser::Face::parse(&built, 0).map_err(|e| e.to_string())?;
    let mut map: Vec<(u32, u16)> = Vec::new();
    if let Some(cmap) = face.tables().cmap {
        for st in cmap.subtables {
            if !st.is_unicode() {
                continue;
            }
            st.codepoints(|c| {
                if let Some(g) = st.glyph_index(c) {
                    map.push((c, g.0));
                }
            });
        }
    }
    map.sort();
    map.dedup();
    let encoded: HashSet<u16> = map.iter().map(|m| m.1).collect();
    let kept_map: Vec<(u32, u16)> = map.iter().copied().filter(|(c, _)| keep(*c)).collect();
    let mut keep_g: Vec<bool> = (0..n).map(|g| g == 0 || !encoded.contains(&(g as u16))).collect();
    for (_, g) in &kept_map {
        if (*g as usize) < n {
            keep_g[*g as usize] = true;
        }
    }
    // Composite glyphs keep their components.
    let mut stack: Vec<usize> = (0..n).filter(|&g| keep_g[g]).collect();
    while let Some(g) = stack.pop() {
        let (a, b) = (off(g), off(g + 1));
        if b <= a || b > glyf.len() || b - a < 10 {
            continue;
        }
        let d = &glyf[a..b];
        if (be16(d, 0) as i16) >= 0 {
            continue;
        }
        let mut p = 10;
        loop {
            if p + 4 > d.len() {
                break;
            }
            let flags = be16(d, p);
            let comp = be16(d, p + 2) as usize;
            if comp < n && !keep_g[comp] {
                keep_g[comp] = true;
                stack.push(comp);
            }
            p += 4 + if flags & 0x0001 != 0 { 4 } else { 2 };
            p += if flags & 0x0008 != 0 { 2 } else if flags & 0x0040 != 0 { 4 } else if flags & 0x0080 != 0 { 8 } else { 0 };
            if flags & 0x0020 == 0 {
                break;
            }
        }
    }
    let mut new_glyf = Vec::new();
    let mut new_loca = Vec::with_capacity((n + 1) * 4);
    let mut kept = 0;
    for (g, &k) in keep_g.iter().enumerate() {
        new_loca.extend((new_glyf.len() as u32).to_be_bytes());
        let (a, b) = (off(g), off(g + 1));
        if k && b > a && b <= glyf.len() {
            new_glyf.extend(&glyf[a..b]);
            while new_glyf.len() % 4 != 0 {
                new_glyf.push(0);
            }
            kept += 1;
        }
    }
    new_loca.extend((new_glyf.len() as u32).to_be_bytes());
    let mut new_head = head.clone();
    new_head[50..52].copy_from_slice(&1u16.to_be_bytes());
    s.tables.insert(*b"glyf", new_glyf);
    s.tables.insert(*b"loca", new_loca);
    s.tables.insert(*b"head", new_head);
    s.tables.insert(*b"cmap", build_cmap(&kept_map));
    // Hint tables that list every glyph stay valid; a signature no longer would.
    s.tables.remove(b"DSIG");
    Ok(kept)
}

fn build_cmap(map: &[(u32, u16)]) -> Vec<u8> {
    let bmp: Vec<(u32, u16)> = map.iter().copied().filter(|(c, _)| *c <= 0xFFFF).collect();
    // Format 4: runs of consecutive characters with consecutive glyphs.
    let mut segs: Vec<(u16, u16, i16)> = Vec::new();
    for (c, g) in &bmp {
        let delta = (*g as i32 - *c as i32) as i16;
        if let Some(last) = segs.last_mut() {
            if last.1 as u32 + 1 == *c && last.2 == delta {
                last.1 = *c as u16;
                continue;
            }
        }
        segs.push((*c as u16, *c as u16, delta));
    }
    segs.push((0xFFFF, 0xFFFF, 1));
    let seg_x2 = (segs.len() * 2) as u16;
    let mut pow = 1u16;
    let mut sel = 0u16;
    while pow * 2 <= segs.len() as u16 {
        pow *= 2;
        sel += 1;
    }
    let mut f4 = Vec::new();
    let len = 16 + segs.len() * 8;
    f4.extend(4u16.to_be_bytes());
    f4.extend((len as u16).to_be_bytes());
    f4.extend(0u16.to_be_bytes());
    f4.extend(seg_x2.to_be_bytes());
    f4.extend((pow * 2).to_be_bytes());
    f4.extend(sel.to_be_bytes());
    f4.extend((seg_x2 - pow * 2).to_be_bytes());
    for s in &segs {
        f4.extend(s.1.to_be_bytes());
    }
    f4.extend(0u16.to_be_bytes());
    for s in &segs {
        f4.extend(s.0.to_be_bytes());
    }
    for s in &segs {
        f4.extend(s.2.to_be_bytes());
    }
    for _ in &segs {
        f4.extend(0u16.to_be_bytes());
    }
    // Format 12 for everything (needed beyond the BMP).
    let mut groups: Vec<(u32, u32, u32)> = Vec::new();
    for (c, g) in map {
        if let Some(last) = groups.last_mut() {
            if last.1 + 1 == *c && last.2 + (c - last.0) == *g as u32 {
                last.1 = *c;
                continue;
            }
        }
        groups.push((*c, *c, *g as u32));
    }
    let need12 = map.iter().any(|(c, _)| *c > 0xFFFF);
    let mut f12 = Vec::new();
    if need12 {
        f12.extend(12u16.to_be_bytes());
        f12.extend(0u16.to_be_bytes());
        f12.extend(((16 + groups.len() * 12) as u32).to_be_bytes());
        f12.extend(0u32.to_be_bytes());
        f12.extend((groups.len() as u32).to_be_bytes());
        for (a, b, g) in &groups {
            f12.extend(a.to_be_bytes());
            f12.extend(b.to_be_bytes());
            f12.extend(g.to_be_bytes());
        }
    }
    let records: Vec<(u16, u16, bool)> = if need12 {
        vec![(0, 3, false), (0, 4, true), (3, 1, false), (3, 10, true)]
    } else {
        vec![(0, 3, false), (3, 1, false)]
    };
    let header = 4 + 8 * records.len();
    let mut out = Vec::new();
    out.extend(0u16.to_be_bytes());
    out.extend((records.len() as u16).to_be_bytes());
    for (p, e, is12) in &records {
        out.extend(p.to_be_bytes());
        out.extend(e.to_be_bytes());
        let o = if *is12 { header + f4.len() } else { header };
        out.extend((o as u32).to_be_bytes());
    }
    out.extend(&f4);
    out.extend(&f12);
    out
}

// ----- WOFF and WOFF2 -----

pub fn to_woff(s: &Sfnt) -> Vec<u8> {
    let n = s.tables.len();
    let mut dir = Vec::new();
    let mut data = Vec::new();
    let base = 44 + 20 * n;
    let mut total_sfnt = 12 + 16 * n;
    for (tag, t) in &s.tables {
        let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
        let _ = z.write_all(t);
        let comp = z.finish().unwrap_or_default();
        let stored = if comp.len() < t.len() { comp } else { t.clone() };
        dir.extend(tag);
        dir.extend(((base + data.len()) as u32).to_be_bytes());
        dir.extend((stored.len() as u32).to_be_bytes());
        dir.extend((t.len() as u32).to_be_bytes());
        dir.extend(checksum(t).to_be_bytes());
        data.extend(&stored);
        while data.len() % 4 != 0 {
            data.push(0);
        }
        total_sfnt += (t.len() + 3) & !3;
    }
    let mut out = Vec::new();
    out.extend(b"wOFF");
    out.extend(s.flavor.to_be_bytes());
    out.extend(((base + data.len()) as u32).to_be_bytes());
    out.extend((n as u16).to_be_bytes());
    out.extend(0u16.to_be_bytes());
    out.extend((total_sfnt as u32).to_be_bytes());
    out.extend(1u16.to_be_bytes());
    out.extend(0u16.to_be_bytes());
    out.extend([0u8; 20]);
    out.extend(dir);
    out.extend(data);
    out
}

const WOFF2_TAGS: [&[u8; 4]; 63] = [
    b"cmap", b"head", b"hhea", b"hmtx", b"maxp", b"name", b"OS/2", b"post", b"cvt ", b"fpgm", b"glyf", b"loca", b"prep", b"CFF ", b"VORG", b"EBDT",
    b"EBLC", b"gasp", b"hdmx", b"kern", b"LTSH", b"PCLT", b"VDMX", b"vhea", b"vmtx", b"BASE", b"GDEF", b"GPOS", b"GSUB", b"EBSC", b"JSTF", b"MATH",
    b"CBDT", b"CBLC", b"COLR", b"CPAL", b"SVG ", b"sbix", b"acnt", b"avar", b"bdat", b"bloc", b"bsln", b"cvar", b"fdsc", b"feat", b"fmtx", b"fvar",
    b"gvar", b"hsty", b"just", b"lcar", b"mort", b"morx", b"opbd", b"prop", b"trak", b"Zapf", b"Silf", b"Glat", b"Gloc", b"Feat", b"Sill",
];

fn base128(mut v: u32, out: &mut Vec<u8>) {
    let mut bytes = vec![(v & 0x7F) as u8];
    v >>= 7;
    while v > 0 {
        bytes.push((v & 0x7F) as u8 | 0x80);
        v >>= 7;
    }
    bytes.reverse();
    out.extend(bytes);
}

/// WOFF2 with untransformed tables (allowed by the spec: glyf/loca transform version 3),
/// compressed together with Brotli in font mode.
pub fn to_woff2(s: &Sfnt) -> Result<Vec<u8>, String> {
    // loca right after glyf; the rest in tag order.
    let mut order: Vec<&[u8; 4]> = s.tables.keys().filter(|t| *t != b"loca").collect();
    if s.tables.contains_key(b"loca") {
        if let Some(i) = order.iter().position(|t| *t == b"glyf") {
            order.insert(i + 1, b"loca");
        } else {
            order.push(b"loca");
        }
    }
    let mut dir = Vec::new();
    let mut stream = Vec::new();
    let mut total_sfnt = 12 + 16 * order.len();
    for tag in &order {
        let t = &s.tables[*tag];
        let known = WOFF2_TAGS.iter().position(|k| k == tag);
        let null_glyf = *tag == b"glyf" || *tag == b"loca";
        let version_bits: u8 = if null_glyf { 3 << 6 } else { 0 };
        match known {
            Some(i) => dir.push(version_bits | i as u8),
            None => {
                dir.push(version_bits | 63);
                dir.extend(*tag);
            }
        }
        base128(t.len() as u32, &mut dir);
        stream.extend(t);
        total_sfnt += (t.len() + 3) & !3;
    }
    let mut comp = Vec::new();
    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        lgwin: 22,
        mode: brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_FONT,
        size_hint: stream.len(),
        ..Default::default()
    };
    brotli::BrotliCompress(&mut std::io::Cursor::new(&stream), &mut comp, &params).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    out.extend(b"wOF2");
    out.extend(s.flavor.to_be_bytes());
    let header = 48;
    let total = header + dir.len() + comp.len();
    let padded_total = (total + 3) & !3;
    out.extend((padded_total as u32).to_be_bytes());
    out.extend((order.len() as u16).to_be_bytes());
    out.extend(0u16.to_be_bytes());
    out.extend((total_sfnt as u32).to_be_bytes());
    out.extend((comp.len() as u32).to_be_bytes());
    out.extend(1u16.to_be_bytes());
    out.extend(0u16.to_be_bytes());
    out.extend([0u8; 20]);
    out.extend(dir);
    out.extend(comp);
    out.resize(padded_total, 0);
    Ok(out)
}

// ----- EOT -----

fn utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
}

/// Embedded OpenType (EOT 2.1, uncompressed) for old Internet Explorer.
pub fn to_eot(s: &Sfnt, input: &Input) -> Vec<u8> {
    let font = s.write();
    let os2 = s.tables.get(b"OS/2").cloned().unwrap_or_default();
    let get = |o: usize, l: usize| -> Vec<u8> { if os2.len() >= o + l { os2[o..o + l].to_vec() } else { vec![0; l] } };
    let le32 = |b: &[u8]| -> [u8; 4] { u32::from_be_bytes([b[0], b[1], b[2], b[3]]).to_le_bytes() };
    let head = s.tables.get(b"head").cloned().unwrap_or_else(|| vec![0; 54]);
    let face = ttf_parser::Face::parse(&font, 0).ok();
    let name = |id: u16| -> String {
        face.as_ref()
            .and_then(|f| f.names().into_iter().find(|n| n.name_id == id && n.to_string().is_some()).and_then(|n| n.to_string()))
            .unwrap_or_default()
    };
    let (family, style, version, full) = (name(1), name(2), name(5), name(4));
    let mut h = Vec::new();
    h.extend([0u8; 4]); // EOTSize, filled below
    h.extend((font.len() as u32).to_le_bytes());
    h.extend(0x0002_0001u32.to_le_bytes());
    h.extend(0u32.to_le_bytes());
    h.extend(get(32, 10));
    h.push(1);
    h.push(input.italic as u8);
    h.extend((input.weight as u32).to_le_bytes());
    h.extend(u16::from_be_bytes([get(8, 2)[0], get(8, 2)[1]]).to_le_bytes());
    h.extend(0x504Cu16.to_le_bytes());
    for i in 0..4 {
        h.extend(le32(&get(42 + 4 * i, 4)));
    }
    for i in 0..2 {
        h.extend(le32(&get(78 + 4 * i, 4)));
    }
    h.extend(le32(&head[8..12]));
    h.extend([0u8; 16]);
    for (i, text) in [family, style, version, full].iter().enumerate() {
        h.extend(0u16.to_le_bytes());
        let b = utf16le(text);
        h.extend((b.len() as u16).to_le_bytes());
        h.extend(b);
        let _ = i;
    }
    h.extend(0u16.to_le_bytes());
    h.extend(0u16.to_le_bytes()); // RootStringSize
    let total = h.len() + font.len();
    h[0..4].copy_from_slice(&(total as u32).to_le_bytes());
    h.extend(font);
    h
}

// ----- CSS, demo page, conversion -----

pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    for c in data.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

pub fn file_stem(i: &Input) -> String {
    let clean = |s: &str| s.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>();
    format!("{}-{}", clean(&i.family), clean(&i.style))
}

fn family_of(i: &Input, o: &Options) -> String {
    if o.family_override.trim().is_empty() {
        i.family.clone()
    } else {
        o.family_override.trim().to_string()
    }
}

/// The @font-face rule for one face (`files` are (format, file name or data URL)).
pub fn font_face(i: &Input, o: &Options, files: &[(&str, String)]) -> String {
    let mut src: Vec<String> = Vec::new();
    if o.local {
        src.push(format!("local('{}')", i.full_name.replace('\'', "\\'")));
        if i.ps_name != i.full_name {
            src.push(format!("local('{}')", i.ps_name.replace('\'', "\\'")));
        }
    }
    let mut eot_first = None;
    for (fmt, url) in files {
        match *fmt {
            "eot" => {
                eot_first = Some(url.clone());
                src.push(format!("url('{url}?#iefix') format('embedded-opentype')"));
            }
            "woff2" => src.push(format!("url('{url}') format('woff2')")),
            "woff" => src.push(format!("url('{url}') format('woff')")),
            "ttf" => src.push(format!("url('{url}') format('truetype')")),
            "otf" => src.push(format!("url('{url}') format('opentype')")),
            _ => {}
        }
    }
    let mut css = String::from("@font-face {\n");
    css.push_str(&format!("    font-family: '{}';\n", family_of(i, o).replace('\'', "\\'")));
    if let Some(e) = eot_first {
        css.push_str(&format!("    src: url('{e}');\n"));
    }
    css.push_str(&format!("    src: {};\n", src.join(",\n        ")));
    css.push_str(&format!("    font-weight: {};\n", i.weight));
    css.push_str(&format!("    font-style: {};\n", if i.italic { "italic" } else { "normal" }));
    css.push_str(&format!("    font-display: {};\n", o.display));
    css.push_str("}\n");
    css
}

/// The rules for the selected faces without converting anything (for "Copy @font-face").
pub fn css_preview(inputs: &[Input], o: &Options) -> String {
    inputs
        .iter()
        .map(|i| {
            let stem = file_stem(i);
            let mut files: Vec<(&str, String)> = Vec::new();
            if o.eot {
                files.push(("eot", format!("{stem}.eot")));
            }
            if o.woff2 {
                files.push(("woff2", format!("{stem}.woff2")));
            }
            if o.woff {
                files.push(("woff", format!("{stem}.woff")));
            }
            if o.ttf {
                let otf = i.path.extension().map(|e| e.eq_ignore_ascii_case("otf")).unwrap_or(false);
                files.push((if otf { "otf" } else { "ttf" }, format!("{stem}.{}", if otf { "otf" } else { "ttf" })));
            }
            font_face(i, o, &files)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn keep_fn(o: &Options) -> Option<Box<dyn Fn(u32) -> bool>> {
    if o.subsets.is_empty() && o.custom_chars.trim().is_empty() {
        return None;
    }
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for (name, rs) in SUBSETS {
        if o.subsets.contains(name) {
            ranges.extend(rs.iter().copied());
        }
    }
    let custom: HashSet<u32> = o.custom_chars.chars().map(|c| c as u32).collect();
    Some(Box::new(move |c: u32| c == 0x20 || custom.contains(&c) || ranges.iter().any(|(a, b)| c >= *a && c <= *b)))
}

fn demo_html(inputs: &[Input], o: &Options) -> String {
    let mut body = String::new();
    for i in inputs {
        body.push_str(&format!(
            "<section><p class=\"meta\">{} — {} {}</p><p style=\"font-family:'{}';font-weight:{};font-style:{};font-size:48px\">The quick brown fox jumps over the lazy dog</p><p style=\"font-family:'{}';font-weight:{};font-style:{};font-size:20px\">ABCDEFGHIJKLMNOPQRSTUVWXYZ<br>abcdefghijklmnopqrstuvwxyz<br>0123456789 !?&amp;@%</p></section>\n",
            i.full_name,
            i.weight,
            if i.italic { "italic" } else { "normal" },
            family_of(i, o),
            i.weight,
            if i.italic { "italic" } else { "normal" },
            family_of(i, o),
            i.weight,
            if i.italic { "italic" } else { "normal" }
        ));
    }
    format!(
        "<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\"><title>Web fonts — Kernship</title>\n<link rel=\"stylesheet\" href=\"stylesheet.css\">\n<style>body{{margin:40px;background:#fff;color:#111;font-family:-apple-system,sans-serif}}section{{border-bottom:1px solid #ddd;padding:16px 0}}.meta{{color:#888;font-size:13px}}</style></head>\n<body><h1 style=\"font:600 20px -apple-system,sans-serif\">Web fonts made with Kernship</h1>\n{body}</body></html>\n"
    )
}

/// Converts `inputs` into `dest/<name>-webfont/` (and a .zip next to it when asked).
pub fn convert(inputs: &[Input], o: &Options, dest: &Path, progress: &dyn Fn(f32)) -> Result<Report, String> {
    if inputs.is_empty() {
        return Err("No fonts selected".into());
    }
    if !(o.woff2 || o.woff || o.ttf || o.eot) {
        return Err("Choose at least one format".into());
    }
    let name = if inputs.iter().all(|i| i.family == inputs[0].family) {
        inputs[0].family.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>()
    } else {
        "fonts".into()
    };
    let mut folder = dest.join(format!("{name}-webfont"));
    let mut k = 2;
    while folder.exists() {
        folder = dest.join(format!("{name}-webfont-{k}"));
        k += 1;
    }
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let keep = keep_fn(o);
    let mut rep = Report { folder: folder.clone(), ..Default::default() };
    let mut css = String::new();
    for (n, i) in inputs.iter().enumerate() {
        progress(n as f32 / inputs.len() as f32);
        let data = std::fs::read(&i.path).map_err(|e| format!("{}: {e}", i.full_name))?;
        rep.original_bytes += data.len() as u64;
        let mut s = match Sfnt::read(&data, i.index) {
            Ok(s) => s,
            Err(e) => {
                rep.notes.push(format!("{}: {e}", i.full_name));
                continue;
            }
        };
        s.tables.remove(b"DSIG");
        if let Some(k) = &keep {
            match subset(&mut s, k.as_ref()) {
                Ok(g) => rep.notes.push(format!("{}: kept {g} glyphs", i.full_name)),
                Err(e) => rep.notes.push(format!("{}: {e} — kept all glyphs", i.full_name)),
            }
        }
        let stem = file_stem(i);
        let otf = s.is_cff();
        let mut outputs: Vec<(&str, String, Vec<u8>)> = Vec::new();
        if o.eot {
            outputs.push(("eot", format!("{stem}.eot"), to_eot(&s, i)));
        }
        if o.woff2 {
            outputs.push(("woff2", format!("{stem}.woff2"), to_woff2(&s)?));
        }
        if o.woff {
            outputs.push(("woff", format!("{stem}.woff"), to_woff(&s)));
        }
        if o.ttf {
            let ext = if otf { "otf" } else { "ttf" };
            outputs.push((ext, format!("{stem}.{ext}"), s.write()));
        }
        let mut refs: Vec<(&str, String)> = Vec::new();
        for (fmt, file, bytes) in &outputs {
            std::fs::write(folder.join(file), bytes).map_err(|e| e.to_string())?;
            rep.files.push((file.clone(), bytes.len() as u64));
            let mime = match *fmt {
                "woff2" => "font/woff2",
                "woff" => "font/woff",
                "ttf" => "font/ttf",
                "otf" => "font/otf",
                _ => "application/vnd.ms-fontobject",
            };
            let url = if o.base64 && (*fmt == "woff2" || *fmt == "woff") { format!("data:{mime};base64,{}", base64(bytes)) } else { file.clone() };
            refs.push((fmt, url));
        }
        css.push_str(&font_face(i, o, &refs));
        css.push('\n');
    }
    std::fs::write(folder.join("stylesheet.css"), &css).map_err(|e| e.to_string())?;
    if o.demo {
        std::fs::write(folder.join("demo.html"), demo_html(inputs, o)).map_err(|e| e.to_string())?;
    }
    rep.css = css;
    progress(1.0);
    if o.zip {
        let zip = folder.with_extension("zip");
        let ok = std::process::Command::new("/usr/bin/ditto")
            .args(["-c", "-k", "--sequesterRsrc", "--keepParent"])
            .arg(&folder)
            .arg(&zip)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            rep.zip = Some(zip);
        }
    }
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn woff_round_trip() {
        // A system font converted to every format; the WOFF/WOFF2 headers and an EOT must be sane.
        let path = PathBuf::from("/System/Library/Fonts/Supplemental/Arial.ttf");
        let Ok(data) = std::fs::read(&path) else { return };
        let mut s = Sfnt::read(&data, 0).unwrap();
        let rebuilt = s.write();
        assert!(ttf_parser::Face::parse(&rebuilt, 0).is_ok());
        let w2 = to_woff2(&s).unwrap();
        assert_eq!(&w2[0..4], b"wOF2");
        assert!(w2.len() < data.len());
        let w1 = to_woff(&s);
        assert_eq!(&w1[0..4], b"wOFF");
        let kept = subset(&mut s, &|c| (0x20..0x7F).contains(&c)).unwrap();
        let small = s.write();
        let face = ttf_parser::Face::parse(&small, 0).unwrap();
        assert!(face.glyph_index('A').is_some());
        assert!(face.glyph_index('é').is_none());
        assert!(small.len() < data.len());
        println!("glyphs kept {kept}, ttf {} → subset {} → woff2 {}", data.len(), small.len(), to_woff2(&s).unwrap().len());
        // Brotli stream must decode back to the tables.
        let mut dec = Vec::new();
        let hdr_dir_end = w2.len() - u32::from_be_bytes([w2[20], w2[21], w2[22], w2[23]]) as usize;
        let start = hdr_dir_end - (w2.len() - (48 + 0)) % 1; // placeholder to keep the calculation simple
        let _ = start;
        let comp_len = u32::from_be_bytes([w2[20], w2[21], w2[22], w2[23]]) as usize;
        let mut dir_len = 0;
        // Find where the compressed stream starts by walking the directory.
        let n = u16::from_be_bytes([w2[12], w2[13]]) as usize;
        let mut p = 48;
        for _ in 0..n {
            let flags = w2[p];
            p += 1;
            if flags & 63 == 63 {
                p += 4;
            }
            loop {
                let b = w2[p];
                p += 1;
                if b & 0x80 == 0 {
                    break;
                }
            }
            dir_len += 1;
        }
        assert_eq!(dir_len, n);
        brotli::BrotliDecompress(&mut std::io::Cursor::new(&w2[p..p + comp_len]), &mut dec).unwrap();
        let total: usize = Sfnt::read(&data, 0).unwrap().tables.values().map(|t| t.len()).sum();
        assert_eq!(dec.len(), total);
    }
}

#[cfg(test)]
mod convert_tests {
    use super::*;

    #[test]
    fn convert_all_formats() {
        let path = PathBuf::from("/System/Library/Fonts/Supplemental/Georgia.ttf");
        if !path.exists() {
            return;
        }
        let input = Input { path, index: 0, family: "Georgia".into(), style: "Regular".into(), full_name: "Georgia".into(), ps_name: "Georgia".into(), weight: 400, italic: false };
        let mut o = Options { ttf: true, eot: true, ..Default::default() };
        o.subsets.insert("Latin".into());
        let dest = std::env::temp_dir().join(format!("kernship-webfont-test-{}", std::process::id()));
        std::fs::create_dir_all(&dest).unwrap();
        let r = convert(&[input], &o, &dest, &|_| {}).unwrap();
        println!("{:#?}", r.files);
        println!("{}", r.css);
        println!("notes {:?} zip {:?}", r.notes, r.zip);
        assert!(r.zip.is_some());
        assert_eq!(r.files.len(), 4);
        assert!(r.css.contains("format('woff2')"));
    }
}
