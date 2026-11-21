//! What the app remembers between launches, kept as JSON in its own folders.
use crate::activate::Scope;
use crate::model::{default_samples, ActiveEntry, Category, FontFace, Library, Sample, SmartFilter, Tag};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Grid,
    List,
}

/// Sort By menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Name,       // 0
    DateAdded,  // 1
    Popular,    // 2
    Enabled,    // 4
    Styles,     // 5
    Starred,    // 6
    Format,     // 7
    Category,   // 8
    Designer,   // 11
    Random,     // 14
}

impl SortBy {
    pub const ALL: [(SortBy, &'static str); 10] = [
        (SortBy::Random, "Random"),
        (SortBy::Enabled, "Enabled"),
        (SortBy::Name, "Name"),
        (SortBy::Styles, "Styles"),
        (SortBy::Starred, "Starred"),
        (SortBy::Format, "Format"),
        (SortBy::Category, "Category"),
        (SortBy::Designer, "Designer"),
        (SortBy::DateAdded, "Date Added"),
        (SortBy::Popular, "Popular"),
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextCase {
    None,
    Upper,
    Lower,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// What the right-hand pane shows: the font's details or "Convert Font for Web".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InfoMode {
    #[default]
    Info,
    Web,
}

/// Settings › General › Appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub appearance: Appearance,
    /// "Run Kernship in background": keep the icon in the menu bar when the window is closed
    pub run_in_background: bool,
    // General
    pub launch_on_login: bool,
    // Auto Activation
    pub enable_auto_activation: bool,
    pub auto_activation_apps: BTreeMap<String, bool>,
    pub activate_entire_family: bool,
    pub silent_mode: bool,
    pub deactivate_auto_on_quit: bool,
    pub deactivate_manual_on_quit: bool,
    // Import
    pub skip_duplicates: bool,
    pub activate_after_import: bool,
    pub copy_into_library: bool,
    // Sidebar ("Show these items in the sidebar")
    pub show_all_fonts: bool,
    pub show_system: bool,
    pub show_starred: bool,
    pub show_recents: bool,
    pub show_google: bool,
    pub show_icon_fonts: bool,
    pub show_smart_filters: bool,
    // Updates
    pub catalog_refresh_days: u32,
    // Import: "Create sets from folder structure" (IncludingFolderStructureWhenImportingFonts)
    pub sets_from_folders: bool,
    // Manual Check: "Shortcut to check missing fonts" and the recorded shortcut (e.g. "⌃⌥M").
    pub check_shortcut_enabled: bool,
    pub check_shortcut: Option<String>,
    // Updates: "Check when Kernship opens", and what the last check found.
    pub update_check: bool,
    /// Unix seconds of the last successful check (0: never).
    pub update_checked: i64,
    pub update_latest: Option<String>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            appearance: Appearance::System,
            run_in_background: true,
            launch_on_login: false,
            enable_auto_activation: true,
            auto_activation_apps: BTreeMap::new(),
            activate_entire_family: false,
            silent_mode: false,
            deactivate_auto_on_quit: false,
            deactivate_manual_on_quit: false,
            // Skipping duplicates and activating after import are both off by default.
            skip_duplicates: false,
            activate_after_import: false,
            copy_into_library: true,
            show_all_fonts: true,
            show_system: true,
            // Starred and Recents are shown by default.
            show_starred: true,
            show_recents: true,
            show_google: true,
            show_icon_fonts: true,
            show_smart_filters: true,
            catalog_refresh_days: 7,
            sets_from_folders: false,
            check_shortcut_enabled: false,
            check_shortcut: None,
            update_check: true,
            update_checked: 0,
            update_latest: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub libraries: Vec<Library>,
    pub tags: Vec<Tag>,
    pub smart_filters: Vec<SmartFilter>,
    /// Font ids that are switched on, with the registered file and how they were switched on.
    pub active: BTreeMap<String, ActiveEntry>,
    pub starred: BTreeSet<String>,
    pub categories: BTreeMap<String, Category>,
    /// Fonts in user libraries (imported), kept so the list shows without rescanning.
    pub library_fonts: Vec<FontFace>,
    pub samples: Vec<Sample>,
    pub preview_text: String,
    pub grid_size: f32,
    pub list_size: f32,
    pub view_mode: ViewMode,
    pub sort_by: SortBy,
    pub sort_ascending: bool,
    pub expand_families: bool,
    pub sidebar_visible: bool,
    pub info_visible: bool,
    pub show_style_bar: bool,
    pub align: Align,
    pub case: TextCase,
    pub letter_spacing: f32,
    pub float_on_top: bool,
    pub prefs: Prefs,
    pub used: BTreeMap<String, u64>,
    pub welcomed: bool,
    pub demo_sets_added: bool,
    /// Widths of the sidebar and info pane, changed by dragging their dividers.
    pub sidebar_width: f32,
    pub info_width: f32,
    /// Icon Fonts slider: 24–128, default 32.
    pub icon_size: f32,
    pub info_mode: InfoMode,
    /// "Convert Font for Web" choices (formats, subsets, CSS options, where to save).
    pub webfont: crate::webfont::Options,
}

impl Default for State {
    fn default() -> Self {
        Self {
            libraries: Vec::new(),
            tags: ["Calligraphic", "Display", "Funny"]
                .iter()
                .map(|n| Tag { id: crate::model::new_id(), name: n.to_string(), font_ids: vec![] })
                .collect(),
            smart_filters: Vec::new(),
            active: BTreeMap::new(),
            starred: BTreeSet::new(),
            categories: BTreeMap::new(),
            library_fonts: Vec::new(),
            samples: default_samples(),
            preview_text: String::new(),
            // Preview size 24 in the grid, 16 in the list.
            grid_size: 24.0,
            list_size: 16.0,
            view_mode: ViewMode::Grid,
            sort_by: SortBy::Name,
            sort_ascending: true,
            expand_families: true,
            sidebar_visible: true,
            info_visible: false,
            show_style_bar: false,
            align: Align::Left,
            case: TextCase::None,
            letter_spacing: 0.0,
            float_on_top: false,
            prefs: Prefs::default(),
            used: BTreeMap::new(),
            welcomed: false,
            demo_sets_added: false,
            sidebar_width: 210.0,
            info_width: 300.0,
            icon_size: 32.0,
            info_mode: InfoMode::Info,
            webfont: Default::default(),
        }
    }
}

pub const APP_NAME: &str = "Kernship";
/// Kernship's own library packages: a folder with the fonts and a manifest.
pub const LIBRARY_EXT: &str = "kernshiplibrary";
/// The manifest of a library package (JSON: uuid, name, kind, created, modified).
pub const MANIFEST: &str = "manifest.json";

pub fn data_dir() -> PathBuf {
    let d = dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join(APP_NAME);
    let _ = std::fs::create_dir_all(&d);
    d
}

/// ~/Kernship: libraries and samples.
pub fn home_root() -> PathBuf {
    let d = dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(APP_NAME);
    let _ = std::fs::create_dir_all(&d);
    d
}

/// ~/Kernship/Libraries.
pub fn libraries_root() -> PathBuf {
    let d = home_root().join("Libraries");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn google_library_dir() -> PathBuf {
    let d = libraries_root().join(format!("Google Fonts.{LIBRARY_EXT}")).join("fonts");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn google_cache_dir() -> PathBuf {
    let d = data_dir().join("Google Fonts Cache");
    let _ = std::fs::create_dir_all(&d);
    d
}

impl State {
    pub fn load() -> Self {
        let p = data_dir().join("state.json");
        std::fs::read_to_string(p)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let p = data_dir().join("state.json");
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let tmp = p.with_extension("tmp");
            if std::fs::write(&tmp, t).is_ok() {
                let _ = std::fs::rename(tmp, p);
            }
        }
    }

    pub fn preview_size(&self) -> f32 {
        match self.view_mode {
            ViewMode::Grid => self.grid_size,
            ViewMode::List => self.list_size,
        }
    }

    pub fn set_preview_size(&mut self, v: f32) {
        match self.view_mode {
            ViewMode::Grid => self.grid_size = v.clamp(SIZE_MIN, GRID_MAX),
            ViewMode::List => self.list_size = v.clamp(SIZE_MIN, LIST_MAX),
        }
    }

    pub fn size_max(&self) -> f32 {
        match self.view_mode {
            ViewMode::Grid => GRID_MAX,
            ViewMode::List => LIST_MAX,
        }
    }

    pub fn _scope(&self) -> Scope {
        Scope::Persistent
    }
}

/// Size slider range.
pub const SIZE_MIN: f32 = 8.0;
pub const ICON_MIN: f32 = 24.0;
pub const ICON_MAX: f32 = 128.0;
pub const GRID_MAX: f32 = 256.0;
pub const LIST_MAX: f32 = 128.0;
