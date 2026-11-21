//! App state and everything that isn't drawing: loading fonts, filtering, activation, downloads.
use crate::activate::{self, Scope};
use crate::design_apps::{self, DesignWatcher};
use crate::google::{self, GFamily};
use crate::model::*;
use crate::render::Renderer;
use crate::scan;
use crate::store::{self, SortBy, State};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

pub struct Group {
    pub family: String,
    pub faces: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Dialog {
    None,
    NewLibrary { name: String, folder: Option<PathBuf> },
    NewCollection { library: String, parent: Option<String>, name: String },
    NewTag { name: String },
    Rename { what: RenameWhat, name: String },
    SmartFilter { filter: SmartFilter, is_new: bool },
    NewSample { name: String, text: String },
    Preferences { tab: PrefTab },
    Details { font_id: String },
    MissingFonts { app: String, names: Vec<String> },
    ConfirmDelete { ids: Vec<String> },
    /// Asks before removing a library, set, tag or smart filter from the sidebar.
    ConfirmRemove { what: RemoveWhat, name: String },
    Welcome,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RemoveWhat {
    Library(String),
    Collection(String, String),
    Tag(String),
    SmartFilter(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenameWhat {
    Library(String),
    Tag(String),
    Collection(String, String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefTab {
    General,
    Sidebar,
    Import,
    AutoActivation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct QuickFilter {
    pub starred: bool,
    pub variable: bool,
    pub activated: bool,
    pub deactivated: bool,
    pub color: bool,
    pub duplicate: bool,
}

/// The search panel's Properties row: within a group any chosen value
/// matches, across groups all must.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PropFilter {
    /// Light, Medium, Bold
    pub weight: [bool; 3],
    /// Narrow, Normal, Expanded
    pub width: [bool; 3],
    /// Low, Regular, High
    pub xheight: [bool; 3],
    /// Old Style, Lining
    pub figures: [bool; 2],
    /// High, Regular, Low
    pub contrast: [bool; 3],
    /// Index into props::LANGUAGES.
    pub language: Option<usize>,
}

impl PropFilter {
    pub fn needs_measuring(&self) -> bool {
        self.xheight.iter().any(|b| *b) || self.figures.iter().any(|b| *b) || self.contrast.iter().any(|b| *b)
    }
}

#[derive(Default)]
pub struct Loaded {
    pub system: Option<Vec<FontFace>>,
    pub catalog: Option<Result<Vec<GFamily>, String>>,
    pub imported: Vec<(String, Vec<FontFace>, bool)>,
    pub downloaded: Vec<(String, ActivatedBy, Result<PathBuf, String>)>,
    pub messages: Vec<String>,
    pub icon_sets: Option<Vec<crate::icons::IconSet>>,
    pub icon_downloaded: Vec<(String, Result<PathBuf, String>)>,
    pub new_sets: Vec<(String, String, Vec<String>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchBy {
    All,
    Name,
    Family,
    Style,
    Manufacturer,
    Designer,
    Copyright,
    License,
}

impl SearchBy {
    pub const ALL: [(SearchBy, &'static str); 8] = [
        (SearchBy::All, "All"),
        (SearchBy::Name, "Name"),
        (SearchBy::Family, "Family"),
        (SearchBy::Style, "Style"),
        (SearchBy::Manufacturer, "Manufacturer"),
        (SearchBy::Designer, "Designer"),
        (SearchBy::Copyright, "Copyright"),
        (SearchBy::License, "License"),
    ];
}

pub struct KernshipApp {
    pub st: State,
    pub all: Vec<FontFace>,
    pub by_id: HashMap<String, usize>,
    pub catalog: Vec<GFamily>,
    pub catalog_by_family: HashMap<String, usize>,
    pub catalog_error: Option<String>,
    pub loading_system: bool,
    pub loading_catalog: bool,
    pub loaded: Arc<Mutex<Loaded>>,

    pub nav: Nav,
    pub search: String,
    pub search_by: SearchBy,
    pub category_filter: HashSet<Category>,
    pub quick: QuickFilter,
    pub selected: Vec<String>,
    pub expanded: HashSet<String>,
    pub groups: Vec<Group>,
    pub dirty: bool,
    pub downloading: HashSet<String>,

    pub renderer: Renderer,
    pub dialog: Dialog,
    pub inspector_tab: u8,
    pub toasts: Vec<(String, f64)>,
    pub watcher: DesignWatcher,
    pub screenshot: Option<PathBuf>,
    pub frame: u64,
    pub shot_requested: bool,
    pub save_due: Option<f64>,
    pub focus_search: bool,
    pub focus_preview: bool,
    pub random_seed: u64,

    // Icon Fonts: icon sets, not fonts.
    pub icon_sets: Vec<crate::icons::IconSet>,
    pub loading_icons: bool,
    pub icon_downloads: HashSet<String>,
    /// Selected icons as (set id, character).
    pub icon_selected: Vec<(String, char)>,
    pub ns_view: usize,
    pub menubar: Option<crate::menubar::MenuBar>,
    /// Settings window: recording the missing-fonts shortcut.
    pub recording_shortcut: bool,
    pub app_icons: HashMap<String, Option<egui::TextureHandle>>,
    pub counts: std::cell::RefCell<HashMap<String, usize>>,
    pub menu_checks: Option<(store::ViewMode, bool, bool, bool)>,
    pub status_item_on: Option<bool>,
    /// The check for a newer Kernship (Settings › General › Updates).
    pub update: crate::update::Checker,
    /// Frames left in which the grid keeps the selected font centred (layout is changing).
    pub center_selected: u32,
    pub list_collapsed: HashSet<String>,
    pub dates: std::cell::RefCell<HashMap<String, String>>,
    /// "Convert Font for Web": the running conversion and the last result.
    pub web_job: Option<Arc<Mutex<crate::webfont::Job>>>,
    pub web_result: Option<Result<crate::webfont::Report, String>>,
    /// Search panel (opens when the search field gets focus; pinned keeps it open).
    pub props_filter: PropFilter,
    pub search_pop: bool,
    pub search_pinned: bool,
    /// Measured font properties by font id, filled in the background (props.json).
    pub props: Arc<Mutex<HashMap<String, crate::props::FontProps>>>,
    pub props_running: Arc<std::sync::atomic::AtomicBool>,
    pub props_queued: usize,
    pub props_seen: Arc<std::sync::atomic::AtomicUsize>,
    pub props_applied: usize,
}

impl KernshipApp {
    pub fn new(cc: &eframe::CreationContext, screenshot: Option<PathBuf>) -> Self {
        crate::theme::install_fonts(&cc.egui_ctx);
        // A screenshot run must not touch the person's saved state or their activated fonts.
        let st = if screenshot.is_some() {
            State::default()
        } else {
            State::load()
        };
        let mut app = Self {
            renderer: Renderer::new(&cc.egui_ctx),
            st,
            all: Vec::new(),
            by_id: HashMap::new(),
            catalog: Vec::new(),
            catalog_by_family: HashMap::new(),
            catalog_error: None,
            loading_system: true,
            loading_catalog: true,
            loaded: Arc::new(Mutex::new(Loaded::default())),
            nav: Nav::GoogleFonts,
            search: String::new(),
            search_by: SearchBy::All,
            category_filter: HashSet::new(),
            quick: QuickFilter::default(),
            selected: Vec::new(),
            expanded: HashSet::new(),
            groups: Vec::new(),
            dirty: true,
            downloading: HashSet::new(),
            dialog: Dialog::None,
            inspector_tab: 4,
            toasts: Vec::new(),
            watcher: DesignWatcher::new(),
            screenshot,
            frame: 0,
            shot_requested: false,
            save_due: None,
            focus_search: false,
            focus_preview: false,
            random_seed: 1,
            icon_sets: Vec::new(),
            loading_icons: true,
            icon_downloads: HashSet::new(),
            icon_selected: Vec::new(),
            ns_view: 0,
            menubar: None,
            recording_shortcut: false,
            app_icons: HashMap::new(),
            counts: std::cell::RefCell::new(HashMap::new()),
            menu_checks: None,
            status_item_on: None,
            update: Default::default(),
            center_selected: 0,
            list_collapsed: HashSet::new(),
            dates: std::cell::RefCell::new(HashMap::new()),
            web_job: None,
            web_result: None,
            props_filter: PropFilter::default(),
            search_pop: false,
            search_pinned: false,
            props: Arc::new(Mutex::new(crate::props::load_cache())),
            props_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            props_queued: 0,
            props_seen: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            props_applied: 0,
        };
        if app.st.libraries.is_empty() {
            // A first start creates one local library named "Font Library".
            let id = new_id();
            let location = store::libraries_root().join(format!("Font Library.{}", store::LIBRARY_EXT));
            write_manifest(&location, &id, "Font Library");
            app.st.libraries.push(Library { id, name: "Font Library".into(), location, collections: vec![], expanded: false });
        }
        if !app.st.demo_sets_added {
            app.add_demo_sets();
        }
        for a in design_apps::APPS.iter() {
            app.st.prefs.auto_activation_apps.entry(a.bundle_id.to_string()).or_insert(a.default_on);
        }
        app.start_loading(false);
        app.load_icon_sets();
        if app.screenshot.is_none() {
            app.menubar = crate::menubar::MenuBar::install(cc.egui_ctx.clone());
            crate::macos::decorate_menus(crate::menubar::ICONS);
            app.reactivate_all();
            app.apply_watcher_settings();
            app.watcher.start(cc.egui_ctx.clone());
            if !app.st.welcomed {
                app.dialog = Dialog::Welcome;
            }
        }
        // Test pictures of other screens: KERNSHIP_SHOT_VIEW=icons|settings|menu (screenshot runs only).
        if app.screenshot.is_some() {
            match std::env::var("KERNSHIP_SHOT_VIEW").as_deref() {
                Ok("icons") => {
                    app.nav = Nav::IconFonts;
                    app.expanded.insert("icons".into());
                }
                Ok("settings") => app.dialog = Dialog::Preferences { tab: PrefTab::AutoActivation },
                Ok("settings-general") => app.dialog = Dialog::Preferences { tab: PrefTab::General },
                // As if a check had found a newer version (the sidebar row and Settings).
                Ok("update") | Ok("update-settings") => {
                    app.st.prefs.update_latest = Some("9.0.0".into());
                    app.st.prefs.update_checked = 1_795_000_000;
                    if std::env::var("KERNSHIP_SHOT_VIEW").as_deref() == Ok("update-settings") {
                        app.dialog = Dialog::Preferences { tab: PrefTab::General };
                    }
                    // KERNSHIP_SHOT_UPDATE=ready|downloading: the later states of the same update
                    match std::env::var("KERNSHIP_SHOT_UPDATE").as_deref() {
                        Ok("ready") => app.update.ready = Some("9.0.0".into()),
                        Ok("downloading") => {
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::mem::forget(tx);
                            app.update.download = Some(("9.0.0".into(), rx));
                        }
                        _ => {}
                    }
                }
                Ok("settings-light") => {
                    app.st.prefs.appearance = store::Appearance::Light;
                    app.dialog = Dialog::Preferences { tab: PrefTab::General };
                }
                Ok("welcome") => app.dialog = Dialog::Welcome,
                Ok("welcome-light") => {
                    app.st.prefs.appearance = store::Appearance::Light;
                    app.dialog = Dialog::Welcome;
                }
                Ok("menu") => {
                    app.selected = vec!["gf:ABeeZee:400".into()];
                }
                Ok("list") => {
                    app.st.view_mode = store::ViewMode::List;
                    app.st.list_size = 18.0;
                }
                Ok("search") => {
                    app.search_pop = true;
                    app.focus_search = true;
                    app.search = "acme".into();
                    app.quick.starred = false;
                    app.props_filter.weight[2] = true;
                }
                Ok("web") => {
                    app.selected = vec!["gf:Abhaya Libre:600".into()];
                    app.st.info_visible = true;
                    app.st.info_mode = store::InfoMode::Web;
                    app.center_selected = 200;
                }
                Ok("info") => {
                    app.selected = vec!["gf:Abhaya Libre:600".into()];
                    app.st.info_visible = true;
                    app.center_selected = 200;
                }
                Ok("light") => {
                    app.st.prefs.appearance = store::Appearance::Light;
                    app.nav = Nav::Library(app.st.libraries[0].id.clone());
                    app.expanded.insert(app.st.libraries[0].id.clone());
                    app.expanded.insert("system".into());
                }
                _ => {}
            }
        }
        app.rebuild_index();
        app
    }

    pub fn apply_watcher_settings(&self) {
        let apps = self
            .st
            .prefs
            .auto_activation_apps
            .iter()
            .filter(|(_, on)| **on)
            .map(|(k, _)| k.clone())
            .collect();
        self.watcher.configure(self.st.prefs.enable_auto_activation, apps);
    }

    pub fn start_loading(&mut self, force_catalog: bool) {
        let l = self.loaded.clone();
        self.loading_system = true;
        std::thread::spawn(move || {
            let faces = scan::scan_dirs(&scan::system_font_dirs(), Source::System);
            l.lock().system = Some(faces);
        });
        let l = self.loaded.clone();
        self.loading_catalog = true;
        let days = self.st.prefs.catalog_refresh_days;
        std::thread::spawn(move || {
            let r = google::load_catalog(&store::data_dir(), force_catalog, days);
            l.lock().catalog = Some(r);
        });
    }

    /// Picks up results from background work.
    pub fn poll(&mut self, now: f64) {
        let mut l = self.loaded.lock();
        if let Some(sys) = l.system.take() {
            self.loading_system = false;
            self.all.retain(|f| f.source != Source::System);
            self.all.extend(sys);
            self.dirty = true;
        }
        if let Some(cat) = l.catalog.take() {
            self.loading_catalog = false;
            match cat {
                Ok(c) => {
                    self.catalog_by_family = c.iter().enumerate().map(|(i, f)| (f.family.clone(), i)).collect();
                    self.all.retain(|f| f.source != Source::Google);
                    self.all.extend(google::faces(&c));
                    self.catalog = c;
                    self.catalog_error = None;
                }
                Err(e) => self.catalog_error = Some(e),
            }
            self.dirty = true;
        }
        if let Some(sets) = l.icon_sets.take() {
            self.icon_sets = sets;
            self.loading_icons = false;
        }
        let icon_downloaded = std::mem::take(&mut l.icon_downloaded);
        for (lib, name, ids) in std::mem::take(&mut l.new_sets) {
            if let Some(library) = self.st.libraries.iter_mut().find(|x| x.id == lib) {
                library.collections.push(Collection { id: new_id(), name, parent: None, font_ids: ids });
                self.expanded.insert(lib);
            }
        }
        let imported = std::mem::take(&mut l.imported);
        let downloaded = std::mem::take(&mut l.downloaded);
        let messages = std::mem::take(&mut l.messages);
        drop(l);
        for (lib, faces, quiet) in imported {
            let mut added = Vec::new();
            for f in faces {
                let dup = self.st.prefs.skip_duplicates
                    && self.st.library_fonts.iter().any(|x| x.ps_name == f.ps_name && x.source == f.source);
                if !dup && !self.st.library_fonts.iter().any(|x| x.id == f.id) {
                    added.push(f.id.clone());
                    self.st.library_fonts.push(f);
                }
            }
            if !quiet {
                let name = self.library_name(&lib);
                self.toast(format!("Imported {} fonts into {name}", added.len()), now);
            }
            self.rebuild_index();
            if self.st.prefs.activate_after_import && !quiet {
                self.set_active_by(&added, true, ActivatedBy::Manual, now);
            }
            self.save_soon(now);
        }
        for (id, by, res) in downloaded {
            self.downloading.remove(&id);
            match res {
                Ok(path) => {
                    if let Some(i) = self.by_id.get(&id).copied() {
                        if let Ok(data) = std::fs::read(&path) {
                            if let Ok(face) = ttf_parser::Face::parse(&data, 0) {
                                self.all[i].glyph_count = face.number_of_glyphs();
                            }
                        }
                        self.all[i].path = Some(path.clone());
                    }
                    if self.activate_path(&id, &path, by) {
                        self.st.used.insert(id.clone(), now_secs());
                    }
                    self.save_soon(now);
                    self.dirty = true;
                }
                Err(e) => self.toast(e, now),
            }
        }
        for m in messages {
            self.toast(m, now);
        }
        for (id, res) in icon_downloaded {
            self.icon_downloads.remove(&id);
            match res {
                Ok(_) => {
                    self.toast("Icon set downloaded", now);
                    self.load_icon_sets();
                }
                Err(e) => self.toast(e, now),
            }
        }
        for req in self.watcher.poll_requests() {
            self.handle_design_request(req, now);
        }
        if self.dirty {
            self.rebuild_index();
        }
        if let Some(t) = self.save_due {
            if now >= t {
                if self.screenshot.is_none() {
                    self.st.save();
                }
                self.save_due = None;
            }
        }
        self.toasts.retain(|(_, until)| *until > now);
    }

    pub fn save_soon(&mut self, now: f64) {
        self.save_due = Some(now + 0.8);
    }

    /// A newer version than this one, as of the last check.
    pub fn update_available(&self) -> Option<String> {
        let p = &self.st.prefs;
        p.update_latest.clone().filter(|v| crate::update::is_newer(v, &crate::update::current()))
    }

    /// Download and prepare `version` in the background, when this copy can replace itself.
    fn start_download(&mut self, version: String) {
        if self.update.download.is_some() || self.update.ready.as_deref() == Some(version.as_str()) || crate::update::can_update_itself().is_none() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let v = version.clone();
        std::thread::spawn(move || {
            let _ = tx.send(crate::update::download(&v));
        });
        self.update.download = Some((version, rx));
        self.update.install_error = None;
    }

    /// The prepared version waits for Kernship to end: a script swaps it in then.
    fn arm_install(&mut self, version: &str, prepared: &std::path::Path) {
        match crate::update::can_update_itself() {
            Some(app) => match crate::update::install_when_closed(&app, prepared) {
                Ok(()) => self.update.ready = Some(version.to_string()),
                Err(e) => self.update.install_error = Some(e),
            },
            None => self.update.install_error = Some("Kernship cannot replace itself in its folder.".into()),
        }
    }

    /// The version found: install it if it is prepared already, otherwise download it.
    fn get_update(&mut self, version: String) {
        match (crate::update::staged(&version), crate::update::can_update_itself()) {
            (Some(prepared), Some(_)) => self.arm_install(&version, &prepared),
            _ => self.start_download(version),
        }
    }

    /// Install the prepared version now: Kernship ends and the script opens the new one.
    pub fn restart_to_update(&mut self) {
        if self.update.ready.is_none() {
            return;
        }
        if let Err(e) = crate::update::request_restart() {
            self.update.install_error = Some(e);
            return;
        }
        // what on_exit does, since exit() skips it
        self.deactivate_on_quit();
        self.st.save();
        std::process::exit(0);
    }

    /// At start (never for screenshots): the check once a day, or at every start with a
    /// pretended version; then the download of what it found, and the script that installs it.
    fn poll_update(&mut self, ctx: &egui::Context) {
        if self.frame == 3 && self.screenshot.is_none() {
            let now = crate::model::now_secs() as i64;
            let due = now - self.st.prefs.update_checked >= crate::update::CHECK_EVERY_SECS || crate::update::pretending();
            if self.st.prefs.update_check && due {
                // the answer decides what to download: the version remembered may be older
                self.update.start(false);
            } else if let Some(v) = self.update_available() {
                self.get_update(v);
            }
        }
        if let Some((v, rx)) = &self.update.download {
            match rx.try_recv() {
                Ok(res) => {
                    let v = v.clone();
                    self.update.download = None;
                    match res {
                        Ok(prepared) => self.arm_install(&v, &prepared),
                        Err(e) => self.update.install_error = Some(e),
                    }
                    // a newer one came out meanwhile: that is the one to install
                    if let Some(newer) = self.update_available().filter(|n| *n != v) {
                        self.start_download(newer);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(500)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.update.download = None,
            }
        }
        let Some(rx) = &self.update.rx else { return };
        match rx.try_recv() {
            Ok(res) => {
                self.update.rx = None;
                match res {
                    Ok(v) => {
                        self.st.prefs.update_latest = Some(v);
                        self.st.prefs.update_checked = crate::model::now_secs() as i64;
                        self.st.save();
                        if let Some(v) = self.update_available() {
                            self.get_update(v);
                        }
                    }
                    Err(e) => self.update.error = Some(e),
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(250)),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.update.rx = None,
        }
    }

    pub fn toast(&mut self, msg: impl Into<String>, now: f64) {
        self.toasts.push((msg.into(), now + 3.5));
    }

    pub fn rebuild_index(&mut self) {
        self.all.retain(|f| !matches!(f.source, Source::Library(_)));
        self.all.extend(self.st.library_fonts.iter().cloned());
        let gdir = store::google_library_dir();
        for f in self.all.iter_mut() {
            if f.source == Source::Google && f.path.is_none() {
                let p = gdir.join(google::file_name(&f.family, f.gf_variant.as_deref().unwrap_or("400")));
                if p.exists() {
                    f.path = Some(p);
                }
            }
            if let Some(c) = self.st.categories.get(&f.id) {
                f.category = *c;
            }
        }
        self.by_id = self.all.iter().enumerate().map(|(i, f)| (f.id.clone(), i)).collect();
        self.counts.borrow_mut().clear();
        self.regroup();
        self.dirty = false;
    }

    pub fn is_active(&self, f: &FontFace) -> bool {
        f.source == Source::System || self.st.active.contains_key(&f.id)
    }

    pub fn activated_by(&self, id: &str) -> Option<ActivatedBy> {
        self.st.active.get(id).map(|e| e.by)
    }

    fn in_library(&self, lib: &str, f: &FontFace) -> bool {
        f.source == Source::Library(lib.to_string())
            || self
                .st
                .libraries
                .iter()
                .find(|l| l.id == lib)
                .map(|l| l.collections.iter().any(|c| c.font_ids.contains(&f.id)))
                .unwrap_or(false)
    }

    fn matches_nav(&self, f: &FontFace) -> bool {
        self.matches(&self.nav, f)
    }

    /// How many fonts (styles) a sidebar item holds, shown when the pointer is over it.
    pub fn nav_count(&self, nav: &Nav) -> usize {
        let key = format!("{nav:?}");
        if let Some(n) = self.counts.borrow().get(&key) {
            return *n;
        }
        let n = match nav {
            Nav::IconFonts => self.icon_sets.iter().map(|s| s.glyphs.len()).sum(),
            Nav::IconFamily(id) => self.icon_set(id).map(|s| s.glyphs.len()).unwrap_or(0),
            _ => self.all.iter().filter(|f| self.matches(nav, f)).count(),
        };
        self.counts.borrow_mut().insert(key, n);
        n
    }

    fn matches(&self, nav: &Nav, f: &FontFace) -> bool {
        // Icon fonts are icon sets (see icons.rs), never listed, activated or categorised as fonts.
        if f.icon_font {
            return false;
        }
        match nav {
            Nav::AllFonts => f.source != Source::Google || f.path.is_some(),
            Nav::SystemFonts => f.source == Source::System,
            Nav::SystemFolder(k) => {
                let p = f.path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                f.source == Source::System
                    && match k {
                        0 => p.starts_with("/System/"),
                        1 => p.starts_with("/Library/"),
                        _ => !p.starts_with("/System/") && !p.starts_with("/Library/"),
                    }
            }
            Nav::Starred => self.st.starred.contains(&f.id),
            Nav::Recents => f.added > 0 && now_secs().saturating_sub(f.added) < 31 * 86400,
            Nav::RecentsPeriod(k) => {
                let age = now_secs().saturating_sub(f.added);
                f.added > 0
                    && match k {
                        0 => age < 86400,
                        1 => (86400..2 * 86400).contains(&age),
                        2 => age < 7 * 86400,
                        _ => age < 31 * 86400,
                    }
            }
            Nav::GoogleFonts => f.source == Source::Google,
            Nav::IconFonts | Nav::IconFamily(_) => false,
            Nav::BuiltinSmart(k) => {
                // Activated / Auto Activated / Deactivated / Temporarily Activated (Google excluded
                // from the bulk lists only when not downloaded).
                if f.source == Source::Google && f.path.is_none() {
                    return false;
                }
                match k {
                    0 => self.is_active(f),
                    1 => self.activated_by(&f.id) == Some(ActivatedBy::Auto),
                    2 => !self.is_active(f),
                    _ => self.activated_by(&f.id) == Some(ActivatedBy::Temporary),
                }
            }
            Nav::SmartFilter(id) => self
                .st
                .smart_filters
                .iter()
                .find(|s| &s.id == id)
                .map(|s| (f.source != Source::Google || f.path.is_some()) && self.filter_matches(s, f))
                .unwrap_or(false),
            Nav::Library(id) => self.in_library(id, f),
            Nav::Collection(lib, col) => self
                .st
                .libraries
                .iter()
                .find(|l| &l.id == lib)
                .and_then(|l| l.collections.iter().find(|c| &c.id == col))
                .map(|c| c.font_ids.contains(&f.id))
                .unwrap_or(false),
            Nav::Tag(id) => self
                .st
                .tags
                .iter()
                .find(|t| &t.id == id)
                .map(|t| t.font_ids.contains(&f.id))
                .unwrap_or(false),
        }
    }

    pub fn filter_matches(&self, s: &SmartFilter, f: &FontFace) -> bool {
        if s.rules.is_empty() {
            return true;
        }
        let test = |r: &Rule| -> bool {
            let v = r.value.to_lowercase();
            let text = |t: &str| {
                let t = t.to_lowercase();
                match r.op {
                    RuleOp::Contains => t.contains(&v),
                    RuleOp::BeginsWith => t.starts_with(&v),
                    RuleOp::EndsWith => t.ends_with(&v),
                    RuleOp::Is => t == v,
                    RuleOp::IsNot => t != v,
                    RuleOp::Greater => t > v,
                    RuleOp::Less => t < v,
                }
            };
            let any = |items: Vec<String>| {
                let hit = items.iter().any(|x| {
                    let x = x.to_lowercase();
                    match r.op {
                        RuleOp::Contains => x.contains(&v),
                        RuleOp::BeginsWith => x.starts_with(&v),
                        RuleOp::EndsWith => x.ends_with(&v),
                        _ => x == v,
                    }
                });
                if r.op == RuleOp::IsNot {
                    !hit
                } else {
                    hit
                }
            };
            let yes = |b: bool| text(if b { "yes" } else { "no" });
            match r.field {
                RuleField::Name => text(&f.display_name()),
                RuleField::Family => text(&f.family),
                RuleField::Style => text(&f.style),
                RuleField::Manufacturer => text(&f.manufacturer),
                RuleField::Designer => text(&f.designer),
                RuleField::Copyright => text(&f.copyright),
                RuleField::License => text(&f.license),
                RuleField::Kind => text(f.format_name()),
                RuleField::Category => text(f.category.label()),
                RuleField::Activation => {
                    let mut states = vec![if self.is_active(f) { "Activated" } else { "Deactivated" }.to_string()];
                    match self.activated_by(&f.id) {
                        Some(ActivatedBy::Temporary) => states.push("Temporarily Activated".into()),
                        Some(ActivatedBy::Auto) => states.push("Auto Activated".into()),
                        _ => {}
                    }
                    any(states)
                }
                RuleField::Starred => yes(self.st.starred.contains(&f.id)),
                RuleField::Variable => yes(f.variable),
                RuleField::Libraries => any(
                    self.st.libraries.iter().filter(|l| self.in_library(&l.id, f)).map(|l| l.name.clone()).collect(),
                ),
                RuleField::Tags => any(
                    self.st.tags.iter().filter(|t| t.font_ids.contains(&f.id)).map(|t| t.name.clone()).collect(),
                ),
            }
        };
        if s.match_all {
            s.rules.iter().all(test)
        } else {
            s.rules.iter().any(test)
        }
    }

    fn matches_search(&self, f: &FontFace, q: &[String]) -> bool {
        if q.is_empty() {
            return true;
        }
        let hay = match self.search_by {
            SearchBy::All => format!("{} {} {} {}", f.full_name, f.family, f.style, f.ps_name),
            SearchBy::Name => f.full_name.clone(),
            SearchBy::Family => f.family.clone(),
            SearchBy::Style => f.style.clone(),
            SearchBy::Manufacturer => f.manufacturer.clone(),
            SearchBy::Designer => f.designer.clone(),
            SearchBy::Copyright => f.copyright.clone(),
            SearchBy::License => f.license.clone(),
        }
        .to_lowercase();
        q.iter().all(|w| hay.contains(w))
    }

    fn matches_filters(&self, f: &FontFace) -> bool {
        if !self.category_filter.is_empty() && !self.category_filter.contains(&f.category) {
            return false;
        }
        let q = self.quick;
        (!q.starred || self.st.starred.contains(&f.id))
            && (!q.variable || f.variable)
            && (!q.activated || self.is_active(f))
            && (!q.deactivated || !self.is_active(f))
    }

    /// Colour, Duplicate and the Properties row; `props` is the measured table (locked once).
    fn matches_props(&self, f: &FontFace, props: &HashMap<String, crate::props::FontProps>, dups: &HashSet<String>) -> bool {
        let q = self.quick;
        let pf = self.props_filter;
        if q.duplicate && !dups.contains(&f.full_name.to_lowercase()) {
            return false;
        }
        let none = PropFilter::default();
        if !q.color && pf == none {
            return true;
        }
        let p = match props.get(&f.id) {
            Some(p) => *p,
            None if f.is_google() => {
                let subsets = self.catalog_by_family.get(&f.family).map(|&i| self.catalog[i].subsets.clone()).unwrap_or_default();
                crate::props::from_catalog(&f.style, &subsets)
            }
            None => return false,
        };
        let pick = |sel: &[bool], i: usize| !sel.iter().any(|b| *b) || sel[i];
        // Weight buckets: Light ≤ 350 < Medium < 550 ≤ Bold.
        let w = if f.weight <= 350 { 0 } else if f.weight < 550 { 1 } else { 2 };
        let wd = if p.width_class < 4 { 0 } else if p.width_class > 5 { 2 } else { 1 };
        if (q.color && !p.color) || !pick(&pf.weight, w) || !pick(&pf.width, wd) {
            return false;
        }
        if pf.needs_measuring() {
            if !p.measured {
                return false;
            }
            let xh = if p.xheight <= 0.4 { 0 } else if p.xheight < 0.6 { 1 } else { 2 };
            let fig = if p.oldstyle <= 0.9 { 0 } else { 1 };
            let ct = if p.contrast > 0.55 { 0 } else if p.contrast > 0.25 { 1 } else { 2 };
            if !pick(&pf.xheight, xh) || !pick(&pf.figures, fig) || !pick(&pf.contrast, ct) {
                return false;
            }
        }
        match pf.language {
            Some(l) => p.langs & (1 << l) != 0,
            None => true,
        }
    }

    pub fn filters_on(&self) -> bool {
        !self.category_filter.is_empty() || self.quick != QuickFilter::default() || self.props_filter != PropFilter::default()
    }

    pub fn clear_filters(&mut self) {
        self.category_filter.clear();
        self.quick = QuickFilter::default();
        self.props_filter = PropFilter::default();
        self.dirty = true;
    }

    /// Measures fonts that have a file but no properties yet, in the background, and saves them
    /// to props.json. New results regroup the list when a property filter is on.
    pub fn ensure_props(&mut self, ctx: &egui::Context) {
        use std::sync::atomic::Ordering;
        let seen = self.props_seen.load(Ordering::Relaxed);
        if seen != self.props_applied {
            self.props_applied = seen;
            if self.quick.color || self.props_filter != PropFilter::default() {
                self.dirty = true;
            }
        }
        if self.all.len() == self.props_queued || self.props_running.load(Ordering::Relaxed) {
            return;
        }
        self.props_queued = self.all.len();
        let have = self.props.lock();
        let cache = store::google_cache_dir();
        let todo: Vec<(String, PathBuf, u32)> = self
            .all
            .iter()
            .filter(|f| !have.contains_key(&f.id))
            .filter_map(|f| {
                let p = f.path.clone().or_else(|| {
                    let c = cache.join(google::file_name(&f.family, f.gf_variant.as_deref().unwrap_or("400")));
                    c.exists().then_some(c)
                })?;
                Some((f.id.clone(), p, f.index))
            })
            .collect();
        drop(have);
        if todo.is_empty() {
            return;
        }
        self.props_running.store(true, Ordering::Relaxed);
        let (map, running, seen, ctx) = (self.props.clone(), self.props_running.clone(), self.props_seen.clone(), ctx.clone());
        std::thread::spawn(move || {
            for (n, (id, path, index)) in todo.into_iter().enumerate() {
                if let Some(p) = crate::props::measure(&path, index) {
                    map.lock().insert(id, p);
                }
                if n % 200 == 199 {
                    seen.fetch_add(1, Ordering::Relaxed);
                    ctx.request_repaint();
                }
            }
            crate::props::save_cache(&map.lock());
            seen.fetch_add(1, Ordering::Relaxed);
            running.store(false, Ordering::Relaxed);
            ctx.request_repaint();
        });
    }

    pub fn regroup(&mut self) {
        let q: Vec<String> = self.search.to_lowercase().split_whitespace().map(String::from).collect();
        let mut fam: HashMap<String, Vec<usize>> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        // Duplicates: another font with the same name.
        let dups: HashSet<String> = if self.quick.duplicate {
            let mut seen: HashMap<String, usize> = HashMap::new();
            for f in self.all.iter().filter(|f| f.path.is_some()) {
                *seen.entry(f.full_name.to_lowercase()).or_default() += 1;
            }
            seen.into_iter().filter(|(_, n)| *n > 1).map(|(k, _)| k).collect()
        } else {
            HashSet::new()
        };
        let props_arc = self.props.clone();
        let props = props_arc.lock();
        for (i, f) in self.all.iter().enumerate() {
            if self.matches_nav(f) && self.matches_search(f, &q) && self.matches_filters(f) && self.matches_props(f, &props, &dups) {
                let key = f.family.clone();
                if !fam.contains_key(&key) {
                    order.push(key.clone());
                }
                fam.entry(key).or_default().push(i);
            }
        }
        drop(props);
        let all = &self.all;
        let first = |a: &String| &all[fam[a][0]];
        match self.st.sort_by {
            // Byte order, as SQLite's BINARY collation: capitals before lower case.
            SortBy::Name => order.sort(),
            SortBy::Popular => order.sort_by_key(|a| {
                let p = first(a).popularity;
                (if p == 0 { u32::MAX } else { p }, sort_key(a))
            }),
            SortBy::DateAdded => order.sort_by(|a, b| {
                let (fa, fb) = (first(a), first(b));
                (fb.added, &fb.date_added).cmp(&(fa.added, &fa.date_added))
            }),
            SortBy::Enabled => order.sort_by_key(|a| (!fam[a].iter().any(|&i| self.is_active(&all[i])), sort_key(a))),
            SortBy::Styles => order.sort_by_key(|a| (std::cmp::Reverse(fam[a].len()), sort_key(a))),
            SortBy::Starred => order.sort_by_key(|a| (!fam[a].iter().any(|&i| self.st.starred.contains(&all[i].id)), sort_key(a))),
            SortBy::Format => order.sort_by_key(|a| (first(a).format_name(), sort_key(a))),
            SortBy::Category => order.sort_by_key(|a| (first(a).category.code(), sort_key(a))),
            SortBy::Designer => order.sort_by_key(|a| (first(a).designer.to_lowercase(), sort_key(a))),
            SortBy::Random => {
                let seed = self.random_seed;
                order.sort_by_key(|a| {
                    let mut h: u64 = seed ^ 0xcbf29ce484222325;
                    for b in a.bytes() {
                        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
                    }
                    h
                })
            }
        }
        if !self.st.sort_ascending && self.st.sort_by != SortBy::Random {
            order.reverse();
        }
        self.groups = order
            .into_iter()
            .map(|family| {
                let mut faces = fam.remove(&family).unwrap();
                faces.sort_by_key(|&i| (all[i].italic, all[i].weight, all[i].style.clone()));
                Group { family, faces }
            })
            .collect();
    }

    pub fn set_nav(&mut self, nav: Nav) {
        if self.nav != nav {
            self.nav = nav;
            self.selected.clear();
            self.dirty = true;
        }
    }

    pub fn library_name(&self, id: &str) -> String {
        self.st.libraries.iter().find(|l| l.id == id).map(|l| l.name.clone()).unwrap_or_default()
    }

    pub fn font(&self, id: &str) -> Option<&FontFace> {
        self.by_id.get(id).map(|&i| &self.all[i])
    }

    /// Where to read the font's file for previews (Google styles are fetched on demand).
    pub fn locator(&self, f: &FontFace) -> impl FnOnce() -> Option<(PathBuf, u32)> + Send + 'static {
        let path = f.path.clone();
        let idx = f.index;
        let g = if f.source == Source::Google {
            let variants = self.variants_of(&f.family);
            Some((f.family.clone(), f.gf_variant.clone().unwrap_or_else(|| "400".into()), variants))
        } else {
            None
        };
        move || {
            if let Some(p) = path {
                return Some((p, idx));
            }
            let (family, variant, variants) = g?;
            google::ensure_file(&store::google_cache_dir(), &family, &variant, &variants).map(|p| (p, 0))
        }
    }

    pub fn variants_of(&self, family: &str) -> Vec<String> {
        self.catalog_by_family.get(family).map(|&i| self.catalog[i].variants.clone()).unwrap_or_default()
    }

    // ----- activation -----

    fn activate_path(&mut self, id: &str, path: &std::path::Path, by: ActivatedBy) -> bool {
        match activate::activate(path, Scope::Persistent) {
            Ok(()) => {
                self.st.active.insert(id.to_string(), ActiveEntry { path: path.to_path_buf(), by });
                true
            }
            Err(e) => {
                self.loaded.lock().messages.push(e);
                false
            }
        }
    }

    /// Registers again what was on when the app last quit.
    pub fn reactivate_all(&mut self) {
        let mut gone = Vec::new();
        for (id, e) in &self.st.active {
            if !e.path.exists() || activate::activate(&e.path, Scope::Persistent).is_err() {
                gone.push(id.clone());
            }
        }
        for id in gone {
            self.st.active.remove(&id);
        }
    }

    pub fn set_active(&mut self, ids: &[String], on: bool, now: f64) {
        self.set_active_by(ids, on, ActivatedBy::Manual, now);
    }

    pub fn set_active_by(&mut self, ids: &[String], on: bool, by: ActivatedBy, now: f64) {
        let mut n = 0;
        for id in ids {
            let Some(f) = self.font(id).cloned() else { continue };
            if f.source == Source::System {
                continue;
            }
            if on {
                if let Some(e) = self.st.active.get_mut(id) {
                    if e.by != by && by != ActivatedBy::Auto {
                        e.by = by;
                    }
                    continue;
                }
                if f.source == Source::Google && f.path.is_none() {
                    self.download(&f, by);
                    n += 1;
                    continue;
                }
                if let Some(p) = &f.path {
                    if self.activate_path(id, p, by) {
                        self.st.used.insert(id.clone(), now_secs());
                        n += 1;
                    }
                }
            } else if let Some(e) = self.st.active.remove(id) {
                // Several styles may share one file (collections); keep it while another is on.
                if !self.st.active.values().any(|x| x.path == e.path) {
                    let _ = activate::deactivate(&e.path, Scope::Persistent);
                }
                n += 1;
            }
        }
        if n > 0 {
            let verb = match (on, by) {
                (false, _) => "Deactivated",
                (true, ActivatedBy::Temporary) => "Temporarily activated",
                (true, ActivatedBy::Auto) => "Auto-activated",
                (true, ActivatedBy::Manual) => "Activated",
            };
            self.toast(format!("{verb} {n} font{}", if n == 1 { "" } else { "s" }), now);
            self.save_soon(now);
            self.dirty = true;
        }
    }

    pub fn toggle_active(&mut self, id: &str, now: f64) {
        let on = self.font(id).map(|f| !self.is_active(f)).unwrap_or(false);
        self.set_active(&[id.to_string()], on, now);
    }

    /// Downloads a Google style into the Google Fonts library and activates it.
    pub fn download(&mut self, f: &FontFace, by: ActivatedBy) {
        if self.downloading.contains(&f.id) || f.source != Source::Google {
            return;
        }
        self.downloading.insert(f.id.clone());
        let id = f.id.clone();
        let family = f.family.clone();
        let variant = f.gf_variant.clone().unwrap_or_else(|| "400".into());
        let variants = self.variants_of(&family);
        let l = self.loaded.clone();
        std::thread::spawn(move || {
            let dest = store::google_library_dir();
            let cached = store::google_cache_dir().join(google::file_name(&family, &variant));
            let target = dest.join(google::file_name(&family, &variant));
            let res = if cached.exists() {
                std::fs::copy(&cached, &target).map(|_| target).map_err(|e| e.to_string())
            } else {
                google::ensure_file(&dest, &family, &variant, &variants)
                    .ok_or_else(|| format!("Failed: download font ({family} {})", google::style_name(&variant)))
            };
            l.lock().downloaded.push((id, by, res));
        });
    }

    /// Opens the right-hand pane on "Convert Font for Web" (or on Info), or closes it when it
    /// already shows that.
    pub fn toggle_pane(&mut self, mode: store::InfoMode, now: f64) {
        if self.st.info_visible && self.st.info_mode == mode {
            self.st.info_visible = false;
        } else {
            self.st.info_visible = true;
            self.st.info_mode = mode;
        }
        self.save_soon(now);
    }

    /// The selected faces as conversion inputs; Google styles not downloaded yet carry what is
    /// needed to fetch them first.
    pub fn web_inputs(&self) -> Vec<(crate::webfont::Input, Option<(String, String, Vec<String>)>)> {
        self.selected
            .iter()
            .filter_map(|id| self.font(id))
            .filter(|f| f.path.is_some() || f.is_google())
            .map(|f| {
                let input = crate::webfont::Input {
                    path: f.path.clone().unwrap_or_default(),
                    index: f.index,
                    family: f.family.clone(),
                    style: f.style.clone(),
                    full_name: f.full_name.clone(),
                    ps_name: f.ps_name.clone(),
                    weight: f.weight,
                    italic: f.italic,
                };
                let fetch = if f.path.is_none() {
                    Some((f.family.clone(), f.gf_variant.clone().unwrap_or_else(|| "400".into()), self.variants_of(&f.family)))
                } else {
                    None
                };
                (input, fetch)
            })
            .collect()
    }

    /// Converts the selected fonts in the background (Google styles are downloaded first).
    pub fn start_web_convert(&mut self, ctx: &egui::Context) {
        if self.web_job.is_some() {
            return;
        }
        let items = self.web_inputs();
        let opts = self.st.webfont.clone();
        let job = Arc::new(Mutex::new(crate::webfont::Job { stage: "Preparing…".into(), ..Default::default() }));
        self.web_job = Some(job.clone());
        self.web_result = None;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mut inputs = Vec::new();
            for (mut input, fetch) in items {
                if let Some((family, variant, variants)) = fetch {
                    job.lock().stage = format!("Downloading {} {}…", input.family, input.style);
                    ctx.request_repaint();
                    match google::ensure_file(&store::google_cache_dir(), &family, &variant, &variants) {
                        Some(p) => input.path = p,
                        None => continue,
                    }
                }
                inputs.push(input);
            }
            job.lock().stage = "Converting…".into();
            let progress = |t: f32| {
                job.lock().progress = t;
                ctx.request_repaint();
            };
            let res = crate::webfont::convert(&inputs, &opts, &opts.save_dir(), &progress);
            job.lock().done = Some(res);
            ctx.request_repaint();
        });
    }

    /// Removes a downloaded Google style from disk (it stays in the catalog).
    pub fn remove_download(&mut self, id: &str, now: f64) {
        self.set_active(&[id.to_string()], false, now);
        if let Some(i) = self.by_id.get(id).copied() {
            if let Some(p) = self.all[i].path.take() {
                let _ = std::fs::remove_file(p);
            }
        }
        self.renderer.forget_font(id);
        self.dirty = true;
    }

    pub fn family_ids(&self, family: &str) -> Vec<String> {
        self.all.iter().filter(|f| f.family == family && self.matches_nav(f)).map(|f| f.id.clone()).collect()
    }

    /// Deactivates on quit what the Auto Activation settings ask for.
    pub fn deactivate_on_quit(&mut self) {
        let p = self.st.prefs.clone();
        let ids: Vec<String> = self
            .st
            .active
            .iter()
            .filter(|(_, e)| match e.by {
                ActivatedBy::Temporary => true,
                ActivatedBy::Auto => p.deactivate_auto_on_quit,
                ActivatedBy::Manual => p.deactivate_manual_on_quit,
            })
            .map(|(k, _)| k.clone())
            .collect();
        for id in ids {
            if let Some(e) = self.st.active.remove(&id) {
                let _ = activate::deactivate(&e.path, Scope::Persistent);
            }
        }
    }

    // ----- libraries, tags, imports -----

    pub fn create_library(&mut self, name: &str, folder: Option<PathBuf>, now: f64) -> String {
        let id = new_id();
        let mut unique = name.to_string();
        let mut k = 2;
        while store::libraries_root().join(format!("{unique}.{}", store::LIBRARY_EXT)).exists() && folder.is_none() {
            unique = format!("{name} {k}");
            k += 1;
        }
        let location = store::libraries_root().join(format!("{unique}.{}", store::LIBRARY_EXT));
        write_manifest(&location, &id, &unique);
        self.st.libraries.push(Library { id: id.clone(), name: unique, location, collections: vec![], expanded: false });
        if let Some(f) = folder {
            self.import_paths(id.clone(), vec![f]);
        }
        self.save_soon(now);
        id
    }

    /// Adds an existing library package (Open Library…).
    pub fn open_library(&mut self, path: PathBuf, now: f64) {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Library".into());
        let manifest: Option<serde_json::Value> = std::fs::read_to_string(path.join(store::MANIFEST))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());
        let name = manifest
            .as_ref()
            .and_then(|m| m["name"].as_str().map(String::from))
            .unwrap_or(name);
        let id = new_id();
        let fonts = path.join("fonts");
        self.st.libraries.push(Library { id: id.clone(), name: name.clone(), location: path.clone(), collections: vec![], expanded: false });
        let l = self.loaded.clone();
        let lib = id.clone();
        std::thread::spawn(move || {
            let mut faces = scan::scan_dirs(&[if fonts.is_dir() { fonts } else { path }], Source::Library(lib.clone()));
            let t = now_secs();
            for f in faces.iter_mut() {
                f.added = t;
            }
            let mut g = l.lock();
            g.imported.push((lib, faces, false));
        });
        self.toast(format!("Opening {name}…"), now);
        self.save_soon(now);
    }

    pub fn import_paths(&mut self, lib: String, paths: Vec<PathBuf>) {
        let Some(location) = self.st.libraries.iter().find(|l| l.id == lib).map(|l| l.location.join("fonts")) else {
            return;
        };
        let copy = self.st.prefs.copy_into_library;
        let make_sets = self.st.prefs.sets_from_folders;
        let l = self.loaded.clone();
        std::thread::spawn(move || {
            let _ = std::fs::create_dir_all(&location);
            // (file, folder it came from when "Create sets from folder structure" is on)
            let mut files: Vec<(PathBuf, Option<String>)> = Vec::new();
            for p in paths {
                if p.is_dir() {
                    let set = make_sets.then(|| p.file_name().map(|n| n.to_string_lossy().to_string())).flatten();
                    for e in walkdir::WalkDir::new(&p).into_iter().flatten() {
                        if e.file_type().is_file() && scan::is_font_file(e.path()) {
                            files.push((e.path().to_path_buf(), set.clone()));
                        }
                    }
                } else if scan::is_font_file(&p) {
                    files.push((p, None));
                }
            }
            let mut faces = Vec::new();
            let mut sets: HashMap<String, Vec<String>> = HashMap::new();
            let t = now_secs();
            for (f, set) in files {
                let target = if copy && !f.starts_with(&location) {
                    let t = location.join(f.file_name().unwrap());
                    if !t.exists() && std::fs::copy(&f, &t).is_err() {
                        continue;
                    }
                    t
                } else {
                    f
                };
                for mut face in scan::read_file(&target, Source::Library(lib.clone())) {
                    face.added = t;
                    if let Some(s) = &set {
                        sets.entry(s.clone()).or_default().push(face.id.clone());
                    }
                    faces.push(face);
                }
            }
            let mut g = l.lock();
            g.imported.push((lib.clone(), faces, false));
            for (name, ids) in sets {
                g.new_sets.push((lib.clone(), name, ids));
            }
        });
    }

    /// Add To Set: puts fonts into a library's set (collection). Google styles are downloaded.
    pub fn add_to_collection(&mut self, ids: &[String], lib: &str, col: &str, now: f64) {
        for id in ids {
            if let Some(f) = self.font(id).cloned() {
                if f.is_google() && f.path.is_none() {
                    self.download(&f, ActivatedBy::Manual);
                }
            }
        }
        if let Some(c) = self
            .st
            .libraries
            .iter_mut()
            .find(|l| l.id == lib)
            .and_then(|l| l.collections.iter_mut().find(|c| c.id == col))
        {
            for id in ids {
                if !c.font_ids.contains(id) {
                    c.font_ids.push(id.clone());
                }
            }
        }
        self.toast(format!("Added {} font(s) to the set", ids.len()), now);
        self.save_soon(now);
        self.dirty = true;
    }

    pub fn copy_into_library(&mut self, ids: &[String], lib: &str, now: f64) {
        let paths: Vec<PathBuf> = ids.iter().filter_map(|i| self.font(i).and_then(|f| f.path.clone())).collect();
        if paths.is_empty() {
            self.toast("Download the fonts first", now);
            return;
        }
        self.import_paths(lib.to_string(), paths);
    }

    pub fn toggle_tag(&mut self, ids: &[String], tag: &str, now: f64) {
        if let Some(t) = self.st.tags.iter_mut().find(|t| t.id == tag) {
            let all_in = ids.iter().all(|i| t.font_ids.contains(i));
            if all_in {
                t.font_ids.retain(|i| !ids.contains(i));
            } else {
                for id in ids {
                    if !t.font_ids.contains(id) {
                        t.font_ids.push(id.clone());
                    }
                }
            }
        }
        self.save_soon(now);
        self.dirty = true;
    }

    pub fn toggle_star(&mut self, ids: &[String], now: f64) {
        let all_on = ids.iter().all(|i| self.st.starred.contains(i));
        for id in ids {
            if all_on {
                self.st.starred.remove(id);
            } else {
                self.st.starred.insert(id.clone());
            }
        }
        self.save_soon(now);
        self.dirty = true;
    }

    pub fn set_category(&mut self, ids: &[String], c: Category, now: f64) {
        for id in ids {
            self.st.categories.insert(id.clone(), c);
        }
        self.save_soon(now);
        self.dirty = true;
    }

    /// Delete from Library: deactivates, forgets the fonts and removes copied files.
    pub fn delete_from_library(&mut self, ids: &[String], now: f64) {
        self.set_active(ids, false, now);
        for f in self.st.library_fonts.iter().filter(|f| ids.contains(&f.id)) {
            if let (Some(p), Source::Library(lib)) = (&f.path, &f.source) {
                let inside = self.st.libraries.iter().any(|l| &l.id == lib && p.starts_with(&l.location));
                let shared = self.st.library_fonts.iter().any(|o| !ids.contains(&o.id) && o.path.as_ref() == Some(p));
                if inside && !shared {
                    let _ = std::fs::remove_file(p);
                }
            }
        }
        self.st.library_fonts.retain(|f| !ids.contains(&f.id));
        for id in ids {
            if self.font(id).map(|f| f.is_google()).unwrap_or(false) {
                self.remove_download(id, now);
            }
        }
        self.remove_from_sets(ids, now);
        self.selected.clear();
        self.dirty = true;
    }

    pub fn remove_from_sets(&mut self, ids: &[String], now: f64) {
        for l in self.st.libraries.iter_mut() {
            for c in l.collections.iter_mut() {
                c.font_ids.retain(|i| !ids.contains(i));
            }
        }
        self.save_soon(now);
        self.dirty = true;
    }

    /// Export…: copies the font files into a folder the person picks.
    pub fn export(&mut self, ids: &[String], dest: PathBuf, now: f64) {
        let mut n = 0;
        for id in ids {
            if let Some(p) = self.font(id).and_then(|f| f.path.clone()) {
                if std::fs::copy(&p, dest.join(p.file_name().unwrap())).is_ok() {
                    n += 1;
                }
            }
        }
        self.toast(format!("Exported {n} font file(s)"), now);
    }

    // ----- design programs -----

    fn find_by_name(&self, name: &str) -> Option<usize> {
        let n = name.to_lowercase().replace([' ', '-', '_'], "");
        let norm = |s: &str| s.to_lowercase().replace([' ', '-', '_'], "");
        self.all
            .iter()
            .position(|f| f.source != Source::Google && (norm(&f.ps_name) == n || norm(&f.full_name) == n))
            .or_else(|| self.all.iter().position(|f| f.source == Source::Google && (norm(&f.ps_name) == n || norm(&f.full_name) == n)))
    }

    fn handle_design_request(&mut self, req: design_apps::Request, now: f64) {
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for name in &req.fonts {
            match self.find_by_name(name) {
                Some(i) => {
                    if !self.is_active(&self.all[i]) {
                        if self.st.prefs.activate_entire_family {
                            let fam = self.all[i].family.clone();
                            let src = self.all[i].source.clone();
                            found.extend(self.all.iter().filter(|f| f.family == fam && f.source == src).map(|f| f.id.clone()));
                        } else {
                            found.push(self.all[i].id.clone());
                        }
                    }
                }
                None => {
                    if !installed_name(name) {
                        missing.push(name.clone());
                    }
                }
            }
        }
        found.dedup();
        if !found.is_empty() {
            self.set_active_by(&found, true, ActivatedBy::Auto, now);
            self.toast(format!("Fonts Activated – {} missing font(s) activated for {}", found.len(), req.app), now);
        }
        if !missing.is_empty() && !self.st.prefs.silent_mode && self.dialog == Dialog::None {
            self.dialog = Dialog::MissingFonts { app: req.app, names: missing };
        }
    }

    pub fn apply_to_selection(&mut self, id: &str, now: f64) {
        let Some(f) = self.font(id).cloned() else { return };
        let Some(app) = self.watcher.last_design_app() else {
            self.toast("Bring a design program to the front first", now);
            return;
        };
        if !self.is_active(&f) {
            self.set_active(&[id.to_string()], true, now);
        }
        match design_apps::apply_font(app, &f.ps_name, &f.family, &f.style) {
            Ok(()) => self.toast(format!("Applied {} in {}", f.display_name(), app.name), now),
            Err(e) => self.toast(e, now),
        }
    }

    pub fn reveal(&self, id: &str) {
        if let Some(p) = self.font(id).and_then(|f| f.path.clone()) {
            let _ = std::process::Command::new("open").arg("-R").arg(p).spawn();
        }
    }

    pub fn nav_title(&self) -> String {
        match &self.nav {
            Nav::AllFonts => "All Fonts".into(),
            Nav::SystemFonts => "System Fonts".into(),
            Nav::SystemFolder(k) => ["System", "All Users", "Current User"][(*k as usize).min(2)].into(),
            Nav::Starred => "Starred".into(),
            Nav::Recents => "Recents".into(),
            Nav::RecentsPeriod(k) => ["Today", "Yesterday", "This Week", "This Month"][(*k as usize).min(3)].into(),
            Nav::GoogleFonts => "Google Fonts".into(),
            Nav::IconFonts => "Icon Fonts".into(),
            Nav::IconFamily(f) => f.clone(),
            Nav::BuiltinSmart(k) => BUILTIN_SMART[(*k as usize).min(3)].into(),
            Nav::SmartFilter(id) => self.st.smart_filters.iter().find(|s| &s.id == id).map(|s| s.name.clone()).unwrap_or_default(),
            Nav::Library(id) => self.library_name(id),
            Nav::Collection(l, c) => self
                .st
                .libraries
                .iter()
                .find(|x| &x.id == l)
                .and_then(|x| x.collections.iter().find(|y| &y.id == c))
                .map(|c| c.name.clone())
                .unwrap_or_default(),
            Nav::Tag(id) => self.st.tags.iter().find(|t| &t.id == id).map(|t| t.name.clone()).unwrap_or_default(),
        }
    }

    pub fn preview_text_for(&self, f: &FontFace) -> String {
        let t = if !self.st.preview_text.trim().is_empty() {
            self.st.preview_text.clone()
        } else {
            f.display_name()
        };
        match self.st.case {
            store::TextCase::None => t,
            store::TextCase::Upper => t.to_uppercase(),
            store::TextCase::Lower => t.to_lowercase(),
        }
    }

    /// Example sets for the first library (Clients, Foundries › Adobe, Apple, Google, Styles ›
    /// Display, Handwriting, Sans Serif, Serif, Traditional, Trial).
    pub fn add_demo_sets(&mut self) {
        self.st.demo_sets_added = true;
        let Some(lib) = self.st.libraries.first_mut() else { return };
        if !lib.collections.is_empty() {
            return;
        }
        let mut add = |name: &str, parent: Option<String>| -> String {
            let id = new_id();
            lib.collections.push(Collection { id: id.clone(), name: name.into(), parent, font_ids: vec![] });
            id
        };
        add("Clients", None);
        let foundries = add("Foundries", None);
        for n in ["Adobe", "Apple", "Google"] {
            add(n, Some(foundries.clone()));
        }
        let styles = add("Styles", None);
        for n in ["Display", "Handwriting", "Sans Serif", "Serif", "Traditional"] {
            add(n, Some(styles.clone()));
        }
        add("Trial", None);
    }

    pub fn load_icon_sets(&mut self) {
        self.loading_icons = true;
        let l = self.loaded.clone();
        std::thread::spawn(move || {
            let sets = crate::icons::discover();
            l.lock().icon_sets = Some(sets);
        });
    }

    pub fn download_icon_set(&mut self, set_id: &str) {
        let Some(remote) = self.icon_sets.iter().find(|s| s.id == set_id).and_then(|s| s.remote.clone()) else { return };
        if !self.icon_downloads.insert(set_id.to_string()) {
            return;
        }
        let l = self.loaded.clone();
        let id = set_id.to_string();
        std::thread::spawn(move || {
            let r = crate::icons::download(&remote);
            l.lock().icon_downloaded.push((id, r));
        });
    }

    /// Icon font files dropped on the icon view become icon sets (with their .css names when present).
    pub fn import_icon_fonts(&mut self, paths: Vec<PathBuf>, now: f64) {
        let dir = crate::icons::own_dir();
        let mut n = 0;
        for p in paths {
            if scan::is_font_file(&p) {
                if let Some(name) = p.file_name() {
                    if std::fs::copy(&p, dir.join(name)).is_ok() {
                        n += 1;
                    }
                }
                for ext in ["css", "codepoints"] {
                    let side = p.with_extension(ext);
                    if side.exists() {
                        if let Some(name) = side.file_name() {
                            let _ = std::fs::copy(&side, dir.join(name));
                        }
                    }
                }
            }
        }
        self.toast(format!("Added {n} icon font(s)"), now);
        self.load_icon_sets();
    }

    pub fn icon_set(&self, id: &str) -> Option<&crate::icons::IconSet> {
        self.icon_sets.iter().find(|s| s.id == id)
    }

    pub fn is_icon_nav(&self) -> bool {
        matches!(self.nav, Nav::IconFonts | Nav::IconFamily(_))
    }

    /// The selected icons as SVG text (one per icon, at the icon slider size).
    pub fn selected_svgs(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (set_id, ch) in &self.icon_selected {
            let Some(set) = self.icon_set(set_id) else { continue };
            let Some(path) = &set.path else { continue };
            let Ok(data) = std::fs::read(path) else { continue };
            let name = set.glyphs.iter().find(|g| g.ch == *ch).map(|g| g.name.clone()).unwrap_or_else(|| format!("u{:04X}", *ch as u32));
            if let Some(svg) = crate::icons::svg(&data, set.index, *ch, self.st.icon_size, "#000000") {
                out.push((name, svg));
            }
        }
        out
    }

    pub fn copy_selected_svg(&mut self, now: f64) {
        let svgs = self.selected_svgs();
        if let Some((_, first)) = svgs.first() {
            crate::macos::copy_svg(first);
            let n = svgs.len();
            self.toast(if n == 1 { "SVG copied".to_string() } else { format!("SVG of the first of {n} icons copied") }, now);
        }
    }

    /// Writes the selected icons as SVG files into `dir`.
    pub fn export_selected_svgs(&mut self, dir: &std::path::Path, now: f64) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for (name, svg) in self.selected_svgs() {
            let p = dir.join(crate::icons::file_name(&name));
            if std::fs::write(&p, svg).is_ok() {
                files.push(p);
            }
        }
        if now > 0.0 {
            self.toast(format!("Saved {} SVG file(s)", files.len()), now);
        }
        files
    }

    pub fn set_launch_on_login(&mut self, on: bool) {
        self.st.prefs.launch_on_login = on;
        let Some(home) = dirs::home_dir() else { return };
        let plist = home.join("Library/LaunchAgents/com.brainupstudio.kernship.plist");
        if on {
            if let Ok(exe) = std::env::current_exe() {
                let body = format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Label</key><string>com.brainupstudio.kernship</string><key>ProgramArguments</key><array><string>{}</string></array><key>RunAtLoad</key><true/></dict></plist>\n",
                    exe.display()
                );
                let _ = std::fs::create_dir_all(plist.parent().unwrap());
                let _ = std::fs::write(&plist, body);
            }
        } else {
            let _ = std::fs::remove_file(plist);
        }
    }
}

fn write_manifest(location: &std::path::Path, id: &str, name: &str) {
    let _ = std::fs::create_dir_all(location.join("fonts"));
    let _ = std::fs::create_dir_all(location.join("metadata/fonts"));
    let manifest = serde_json::json!({
        "uuid": id, "name": name, "kind": "KernshipLibrary", "type": 1, "version": 1,
        "created": now_secs(), "modified": now_secs(),
    });
    let p = location.join(store::MANIFEST);
    if !p.exists() {
        let _ = std::fs::write(p, serde_json::to_string_pretty(&manifest).unwrap());
    }
}

/// True when the system already has a font with this PostScript name (not missing).
fn installed_name(name: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        use core_foundation::base::TCFType;
        use core_foundation::string::CFString;
        #[link(name = "CoreText", kind = "framework")]
        extern "C" {
            fn CTFontCreateWithName(name: core_foundation_sys::string::CFStringRef, size: f64, m: *const std::ffi::c_void) -> *const std::ffi::c_void;
            fn CTFontCopyPostScriptName(font: *const std::ffi::c_void) -> core_foundation_sys::string::CFStringRef;
        }
        unsafe {
            let n = CFString::new(name);
            let f = CTFontCreateWithName(n.as_concrete_TypeRef(), 12.0, std::ptr::null());
            if f.is_null() {
                return false;
            }
            let ps = CFString::wrap_under_create_rule(CTFontCopyPostScriptName(f));
            core_foundation_sys::base::CFRelease(f);
            ps == name
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = name;
        false
    }
}

impl eframe::App for KernshipApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        self.frame += 1;
        if self.ns_view == 0 {
            use raw_window_handle::HasWindowHandle;
            if let Ok(h) = frame.window_handle() {
                #[cfg(target_os = "macos")]
                if let raw_window_handle::RawWindowHandle::AppKit(a) = h.as_raw() {
                    self.ns_view = a.ns_view.as_ptr() as usize;
                    crate::macos::set_window_movable(self.ns_view as *mut std::ffi::c_void, false);
                    crate::macos::set_main_view(self.ns_view as *mut std::ffi::c_void);
                }
                let _ = h;
            }
        }
        if self.screenshot.is_none() && !self.loading_system {
            self.ensure_props(ctx);
        }
        if self.screenshot.is_none() {
            // Run in background: the menu bar icon follows the setting, and closing the window
            // hides it (auto activation keeps working) instead of quitting.
            let bg = self.st.prefs.run_in_background;
            if self.status_item_on != Some(bg) {
                self.status_item_on = Some(bg);
                if let Some(m) = &self.menubar {
                    m.set_status_item(bg);
                }
            }
            if bg && ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                crate::macos::hide_window();
                self.st.save();
            }
            // AppKit adds "Enter Full Screen" to the View menu later and renames it: refresh icons.
            if self.frame % 90 == 5 {
                crate::macos::decorate_menus(crate::menubar::ICONS);
            }
            // Window buttons on the sidebar's top row (centre 26 pt down, first button 14 pt in).
            crate::macos::position_traffic_lights(self.ns_view as *mut std::ffi::c_void, 14.0, 26.0);
            let checks = (self.st.view_mode, self.st.expand_families, self.st.float_on_top, self.st.info_visible);
            if self.menu_checks != Some(checks) {
                self.menu_checks = Some(checks);
                crate::macos::set_menu_checked(&[
                    ("as Grid", checks.0 == store::ViewMode::Grid),
                    ("as List", checks.0 == store::ViewMode::List),
                    ("Expand Font Families", checks.1),
                    ("Float on Top", checks.2),
                ]);
            }
        }
        // Appearance: System follows macOS; the switch fades over 200 ms.
        let want_light = match self.st.prefs.appearance {
            store::Appearance::Light => true,
            store::Appearance::Dark => false,
            store::Appearance::System => ctx.input(|i| i.raw.system_theme) == Some(egui::Theme::Light),
        };
        let t = ctx.animate_bool_with_time(egui::Id::new("appearance"), want_light, crate::theme::ANIM);
        if (t - crate::theme::light_t()).abs() > 0.001 || self.frame == 1 {
            crate::theme::set_light_t(t);
            crate::theme::apply_style(ctx);
        }
        if std::env::var_os("KERNSHIP_DEBUG_REPAINT").is_some() {
            let causes: Vec<String> = ctx.repaint_causes().iter().map(|c| format!("{}:{} {}", c.file, c.line, c.reason)).collect();
            eprintln!("frame {} {:.3}s {:?}", self.frame, now, causes);
        }
        let actions = self.menubar.as_ref().map(|m| m.poll()).unwrap_or_default();
        for a in actions {
            crate::ui_main::run_action(self, ctx, a, now);
        }
        self.poll(now);
        self.poll_update(ctx);
        self.renderer.collect(ctx);
        crate::ui_main::draw(self, ctx, now);
        if self.loading_system || self.loading_catalog || !self.downloading.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(1000));
        }

        if let Some(path) = self.screenshot.clone() {
            ctx.request_repaint();
            // Draw at least 120 frames and wait for the catalog and the first previews.
            let pending = self.renderer.pending_count();
            let ready = !self.loading_catalog && !self.loading_system && pending == 0;
            if ((ready && self.frame > 120) || self.frame > 1500) && !self.shot_requested {
                self.shot_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            let shot = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(img) = shot {
                let [w, h] = img.size;
                let mut buf = image::RgbaImage::new(w as u32, h as u32);
                for (i, p) in img.pixels.iter().enumerate() {
                    buf.put_pixel((i % w) as u32, (i / w) as u32, image::Rgba(p.to_array()));
                }
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = buf.save(&path);
                std::process::exit(0);
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.screenshot.is_none() {
            self.deactivate_on_quit();
            self.st.save();
        }
    }
}
