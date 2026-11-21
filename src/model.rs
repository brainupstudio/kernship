//! Fonts, families, libraries, tags, smart filters and preview samples.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Where a font comes from:
/// 0 default library, 1 user library (folder), 2 icon fonts, 3 Google Fonts.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    System,
    Library(String),
    Google,
}

/// Classification codes (0 none, 1 sans, 2 serif, 3 slab, 4 script,
/// 5 blackletter, 7 display, 8 symbols, 9 monospaced).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    SansSerif,
    Serif,
    SlabSerif,
    Script,
    Blackletter,
    Display,
    Symbol,
    Monospace,
    Unknown,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::SansSerif => "Sans Serif",
            Category::Serif => "Serif",
            Category::SlabSerif => "Slab Serif",
            Category::Script => "Script",
            Category::Blackletter => "Blackletter",
            Category::Display => "Display",
            Category::Symbol => "Symbols",
            Category::Monospace => "Monospaced",
            Category::Unknown => "Uncategorized",
        }
    }
    pub fn chip(self) -> &'static str {
        match self {
            Category::SansSerif => "Sans",
            Category::Serif => "Serif",
            Category::SlabSerif => "Slab",
            Category::Script => "Script",
            Category::Blackletter => "Blackletter",
            Category::Display => "Display",
            Category::Symbol => "Symbols",
            Category::Monospace => "Mono",
            Category::Unknown => "None",
        }
    }
    pub fn code(self) -> u8 {
        match self {
            Category::Unknown => 0,
            Category::SansSerif => 1,
            Category::Serif => 2,
            Category::SlabSerif => 3,
            Category::Script => 4,
            Category::Blackletter => 5,
            Category::Display => 7,
            Category::Symbol => 8,
            Category::Monospace => 9,
        }
    }
    /// Google's catalog categories mapped onto these codes.
    pub fn from_google(s: &str) -> Self {
        match s {
            "Sans Serif" => Category::SansSerif,
            "Serif" => Category::Serif,
            "Display" => Category::Display,
            "Handwriting" => Category::Script,
            "Monospace" => Category::Monospace,
            _ => Category::Unknown,
        }
    }
    pub const ALL: [Category; 9] = [
        Category::Unknown,
        Category::SansSerif,
        Category::Serif,
        Category::SlabSerif,
        Category::Script,
        Category::Blackletter,
        Category::Display,
        Category::Symbol,
        Category::Monospace,
    ];
    /// Order of the filter chips: Sans, Serif, Slab, Script, Display, Blackletter, Mono, Symbols, None.
    pub const CHIPS: [Category; 9] = [
        Category::SansSerif,
        Category::Serif,
        Category::SlabSerif,
        Category::Script,
        Category::Display,
        Category::Blackletter,
        Category::Monospace,
        Category::Symbol,
        Category::Unknown,
    ];
}

/// How a font was switched on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivatedBy {
    Manual,
    Temporary,
    Auto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveEntry {
    pub path: PathBuf,
    pub by: ActivatedBy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontFace {
    pub id: String,
    pub family: String,
    pub style: String,
    pub full_name: String,
    pub ps_name: String,
    pub path: Option<PathBuf>,
    pub index: u32,
    pub weight: u16,
    pub italic: bool,
    pub source: Source,
    pub category: Category,
    /// Google Fonts variant key such as "400" or "700i".
    pub gf_variant: Option<String>,
    pub version: String,
    pub designer: String,
    pub copyright: String,
    pub license: String,
    pub manufacturer: String,
    pub glyph_count: u16,
    pub monospaced: bool,
    pub icon_font: bool,
    pub variable: bool,
    pub popularity: u32,
    pub date_added: String,
    /// Seconds since 1970 when it was imported (for Recents).
    #[serde(default)]
    pub added: u64,
}

impl FontFace {
    pub fn display_name(&self) -> String {
        if self.full_name.is_empty() {
            format!("{} {}", self.family, self.style)
        } else {
            self.full_name.clone()
        }
    }
    pub fn is_google(&self) -> bool {
        self.source == Source::Google
    }
    pub fn format_name(&self) -> &'static str {
        let ext = self
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "ttf".into());
        match ext.as_str() {
            "otf" => "OpenType PostScript",
            "ttc" | "otc" => "TrueType Collection",
            "dfont" => "DFONT",
            _ => "OpenType TrueType",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
    pub font_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Library {
    pub id: String,
    pub name: String,
    /// Folder holding the library's font files (a `.kernshiplibrary` package).
    pub location: PathBuf,
    pub collections: Vec<Collection>,
    #[serde(default)]
    pub expanded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub font_ids: Vec<String>,
}

/// Smart filter keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleField {
    Name,
    Family,
    Style,
    Manufacturer,
    Designer,
    Copyright,
    License,
    Kind,
    Category,
    Activation,
    Starred,
    Variable,
    Libraries,
    Tags,
}

impl RuleField {
    pub const ALL: [RuleField; 14] = [
        RuleField::Name,
        RuleField::Family,
        RuleField::Style,
        RuleField::Manufacturer,
        RuleField::Designer,
        RuleField::Copyright,
        RuleField::License,
        RuleField::Kind,
        RuleField::Category,
        RuleField::Activation,
        RuleField::Starred,
        RuleField::Variable,
        RuleField::Libraries,
        RuleField::Tags,
    ];
    pub fn label(self) -> &'static str {
        match self {
            RuleField::Name => "Name",
            RuleField::Family => "Family",
            RuleField::Style => "Style",
            RuleField::Manufacturer => "Manufacturer",
            RuleField::Designer => "Designer",
            RuleField::Copyright => "Copyright",
            RuleField::License => "License",
            RuleField::Kind => "Kind",
            RuleField::Category => "Category",
            RuleField::Activation => "Activation",
            RuleField::Starred => "Starred",
            RuleField::Variable => "Variable",
            RuleField::Libraries => "Libraries",
            RuleField::Tags => "Tags",
        }
    }
    /// Fixed choices for keys that have them.
    pub fn choices(self) -> Option<Vec<&'static str>> {
        match self {
            RuleField::Activation => Some(vec!["Activated", "Deactivated", "Temporarily Activated", "Auto Activated"]),
            RuleField::Starred | RuleField::Variable => Some(vec!["Yes", "No"]),
            RuleField::Category => Some(Category::ALL.iter().map(|c| c.label()).collect()),
            RuleField::Kind => Some(vec!["OpenType PostScript", "OpenType TrueType", "TrueType", "TrueType Collection", "DFONT"]),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleOp {
    Is,
    IsNot,
    Contains,
    BeginsWith,
    EndsWith,
    Greater,
    Less,
}

impl RuleOp {
    pub fn label(self) -> &'static str {
        match self {
            RuleOp::Contains => "contains",
            RuleOp::BeginsWith => "begins with",
            RuleOp::EndsWith => "ends with",
            RuleOp::Is => "is",
            RuleOp::IsNot => "is not",
            RuleOp::Greater => "is greater than",
            RuleOp::Less => "is less than",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub field: RuleField,
    pub op: RuleOp,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SmartFilter {
    pub id: String,
    pub name: String,
    pub match_all: bool,
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub name: String,
    pub text: String,
}

/// The eight preview samples Kernship starts with.
pub fn default_samples() -> Vec<Sample> {
    let s = |n: &str, t: &str| Sample { name: n.into(), text: t.into() };
    vec![
        s("123", "0123456789"),
        s("ABC", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        s("abc", "abcdefghijklmnopqrstuvwxyz"),
        s("deutsch", "Victor jagt zwölf Boxkämpfer quer über den großen Sylter Deich."),
        s("fox", "The quick brown fox jumps over the lazy dog"),
        s("français", "Voix ambiguë d'un cœur qui au zéphyr préfère les jattes de kiwis"),
        s("lorem", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt"),
        s("symbols", "¿ ? ¡ ! & @ ‘ ’ “ ” « » % * ^ # $ £ € ¢ / ( ) [ ] { } . , ® ©"),
    ]
}

/// The sidebar entry being shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Nav {
    AllFonts,
    SystemFonts,
    /// 0 System, 1 All Users, 2 Current User.
    SystemFolder(u8),
    Starred,
    Recents,
    /// 0 Today, 1 Yesterday, 2 This Week, 3 This Month.
    RecentsPeriod(u8),
    GoogleFonts,
    IconFonts,
    IconFamily(String),
    /// Built-in smart lists: Activated, Auto Activated, Deactivated, Temporarily Activated.
    BuiltinSmart(u8),
    SmartFilter(String),
    Library(String),
    Collection(String, String),
    Tag(String),
}

pub const BUILTIN_SMART: [&str; 4] = ["Activated Fonts", "Auto Activated", "Deactivated Fonts", "Temporarily Activated"];

pub fn weight_name(w: u16) -> &'static str {
    match w {
        0..=149 => "Thin",
        150..=249 => "ExtraLight",
        250..=349 => "Light",
        350..=449 => "Regular",
        450..=549 => "Medium",
        550..=649 => "SemiBold",
        650..=749 => "Bold",
        750..=849 => "ExtraBold",
        _ => "Black",
    }
}

pub fn new_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let c = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("{:X}-{:X}", n, c)
}

/// Case-insensitive natural sort key used for family ordering (digits first).
pub fn sort_key(s: &str) -> String {
    s.to_lowercase()
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
