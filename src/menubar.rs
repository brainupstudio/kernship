//! The macOS menu bar: Kernship's menus and shortcuts.
use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{ContextMenu, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use parking_lot::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Settings,
    Import,
    AddFolder,
    NewLibrary,
    OpenLibrary,
    NewTag,
    NewSmartFilter,
    Find,
    Activate,
    ActivateTemporarily,
    Deactivate,
    Export,
    ShowInFinder,
    RemoveFromSet,
    DeleteFromLibrary,
    Grid,
    List,
    ExpandFamilies,
    EditPreviewText,
    IncreaseSize,
    DecreaseSize,
    ToggleInfo,
    ToggleSidebar,
    FloatOnTop,
    CheckMissingFonts,
    SyncGoogleFonts,
    Welcome,
    ToggleDarkMode,
    /// From the menu bar icon: bring the window back.
    ShowWindow,
}

/// SF Symbols for the menu bar items (by title).
pub const ICONS: &[(&str, &str)] = &[
    ("Settings…", "gearshape"),
    ("Import Fonts…", "square.and.arrow.down"),
    ("Add Folder…", "folder.badge.plus"),
    ("New Library…", "books.vertical"),
    ("Open Library…", "folder"),
    ("New Tag", "number"),
    ("New Smart Filter…", "folder.badge.gearshape"),
    ("Sync Google Fonts", "arrow.triangle.2.circlepath"),
    ("Find", "magnifyingglass"),
    ("Activate", "NSStatusAvailable"),
    ("Activate Temporarily", "NSStatusPartiallyAvailable"),
    ("Deactivate", "NSStatusNone"),
    ("Export…", "square.and.arrow.up"),
    ("Show in Finder", "arrow.right"),
    ("Remove from Fontlist", "minus.circle"),
    ("Delete from Library", "trash"),
    ("Check Missing Fonts", "text.magnifyingglass"),
    ("as Grid", "square.grid.2x2"),
    ("as List", "list.bullet"),
    ("Expand Font Families", "rectangle.expand.vertical"),
    ("Enter Full Screen", "arrow.up.left.and.arrow.down.right"),
    ("Exit Full Screen", "arrow.down.right.and.arrow.up.left"),
    ("Close Window", "xmark.square"),
    ("Minimize", "minus.square"),
    ("Zoom", "plus.square"),
    ("Bring All to Front", "macwindow.on.rectangle"),
    ("About Kernship", "info.circle"),
    ("Hide Kernship", "eye.slash"),
    ("Hide Others", "eye.slash"),
    ("Show All", "eye"),
    ("Quit Kernship", "power"),
    ("Open Kernship", "macwindow"),
    ("Edit Preview Text", "character.cursor.ibeam"),
    ("Increase Size", "plus.magnifyingglass"),
    ("Decrease Size", "minus.magnifyingglass"),
    ("Toggle Sidebar", "sidebar.left"),
    ("Toggle Info Pane", "info.circle"),
    ("Toggle Dark Mode", "moon"),
    ("Float on Top", "pin"),
    ("Welcome to Kernship", "sparkles"),
];

pub struct MenuBar {
    _menu: Menu,
    /// The menu of the menu bar icon (Settings › General › Run Kernship in background).
    pub tray: Submenu,
    queue: Arc<Mutex<Vec<Action>>>,
    ctx: egui::Context,
}

fn acc(mods: Modifiers, code: Code) -> Option<Accelerator> {
    Some(Accelerator::new(Some(mods), code))
}

impl MenuBar {
    pub fn install(ctx: egui::Context) -> Option<Self> {
        let ctx2 = ctx.clone();
        let menu = Menu::new();
        let mut ids = HashMap::new();
        let cmd = Modifiers::SUPER;
        let mut item = |sub: &Submenu, text: &str, a: Option<Accelerator>, action: Action| {
            let it = MenuItem::new(text, true, a);
            ids.insert(it.id().clone(), action);
            let _ = sub.append(&it);
        };

        let app = Submenu::new("Kernship", true);
        let _ = app.append(&PredefinedMenuItem::about(Some("About Kernship"), None));
        let _ = app.append(&PredefinedMenuItem::separator());
        item(&app, "Settings…", acc(cmd, Code::Comma), Action::Settings);
        let _ = app.append(&PredefinedMenuItem::separator());
        let _ = app.append(&PredefinedMenuItem::hide(None));
        let _ = app.append(&PredefinedMenuItem::hide_others(None));
        let _ = app.append(&PredefinedMenuItem::show_all(None));
        let _ = app.append(&PredefinedMenuItem::separator());
        let _ = app.append(&PredefinedMenuItem::quit(Some("Quit Kernship")));

        let file = Submenu::new("File", true);
        item(&file, "Import Fonts…", acc(cmd, Code::KeyO), Action::Import);
        item(&file, "Add Folder…", acc(cmd | Modifiers::ALT, Code::KeyO), Action::AddFolder);
        let _ = file.append(&PredefinedMenuItem::separator());
        item(&file, "New Library…", acc(cmd, Code::KeyN), Action::NewLibrary);
        item(&file, "Open Library…", acc(cmd | Modifiers::CONTROL, Code::KeyO), Action::OpenLibrary);
        item(&file, "New Tag", acc(cmd | Modifiers::SHIFT, Code::KeyN), Action::NewTag);
        item(&file, "New Smart Filter…", acc(cmd | Modifiers::ALT, Code::KeyN), Action::NewSmartFilter);
        let _ = file.append(&PredefinedMenuItem::separator());
        item(&file, "Sync Google Fonts", None, Action::SyncGoogleFonts);
        // ⌘W: same path as the red button (performClose:), so with Run in Background the
        // window goes to the menu bar instead of quitting.
        let _ = file.append(&PredefinedMenuItem::separator());
        let _ = file.append(&PredefinedMenuItem::close_window(Some("Close Window")));

        let edit = Submenu::new("Edit", true);
        item(&edit, "Find", acc(cmd, Code::KeyF), Action::Find);

        let font = Submenu::new("Font", true);
        item(&font, "Activate", acc(cmd, Code::KeyR), Action::Activate);
        item(&font, "Activate Temporarily", acc(cmd | Modifiers::ALT, Code::KeyR), Action::ActivateTemporarily);
        item(&font, "Deactivate", acc(cmd | Modifiers::SHIFT, Code::KeyR), Action::Deactivate);
        let _ = font.append(&PredefinedMenuItem::separator());
        item(&font, "Export…", None, Action::Export);
        item(&font, "Show in Finder", None, Action::ShowInFinder);
        let _ = font.append(&PredefinedMenuItem::separator());
        item(&font, "Remove from Fontlist", None, Action::RemoveFromSet);
        item(&font, "Delete from Library", None, Action::DeleteFromLibrary);
        let _ = font.append(&PredefinedMenuItem::separator());
        item(&font, "Check Missing Fonts", None, Action::CheckMissingFonts);

        let view = Submenu::new("View", true);
        item(&view, "as Grid", acc(cmd, Code::Digit1), Action::Grid);
        item(&view, "as List", acc(cmd, Code::Digit2), Action::List);
        item(&view, "Expand Font Families", acc(cmd, Code::KeyG), Action::ExpandFamilies);
        let _ = view.append(&PredefinedMenuItem::separator());
        item(&view, "Edit Preview Text", acc(cmd, Code::KeyE), Action::EditPreviewText);
        item(&view, "Increase Size", acc(cmd, Code::Equal), Action::IncreaseSize);
        item(&view, "Decrease Size", acc(cmd, Code::Minus), Action::DecreaseSize);
        let _ = view.append(&PredefinedMenuItem::separator());
        item(&view, "Toggle Sidebar", acc(cmd | Modifiers::CONTROL, Code::KeyS), Action::ToggleSidebar);
        item(&view, "Toggle Info Pane", acc(cmd, Code::KeyI), Action::ToggleInfo);
        item(&view, "Toggle Dark Mode", acc(cmd, Code::KeyD), Action::ToggleDarkMode);

        let window = Submenu::new("Window", true);
        let _ = window.append(&PredefinedMenuItem::minimize(None));
        let _ = window.append(&PredefinedMenuItem::maximize(Some("Zoom")));
        item(&window, "Float on Top", acc(cmd | Modifiers::CONTROL, Code::KeyT), Action::FloatOnTop);
        let _ = window.append(&PredefinedMenuItem::separator());
        item(&window, "Welcome to Kernship", None, Action::Welcome);
        let _ = window.append(&PredefinedMenuItem::bring_all_to_front(None));

        // The menu bar icon's menu (works while the window is closed).
        let tray = Submenu::new("Kernship", true);
        item(&tray, "Open Kernship", None, Action::ShowWindow);
        item(&tray, "Check Missing Fonts", None, Action::CheckMissingFonts);
        let _ = tray.append(&PredefinedMenuItem::separator());
        item(&tray, "Settings…", None, Action::Settings);
        let _ = tray.append(&PredefinedMenuItem::separator());
        let _ = tray.append(&PredefinedMenuItem::quit(Some("Quit Kernship")));

        for s in [&app, &file, &edit, &font, &view, &window] {
            let _ = menu.append(s);
        }
        #[cfg(target_os = "macos")]
        menu.init_for_nsapp();
        // Menu clicks arrive here even while the window is closed (no frames are drawn then).
        let ids = Arc::new(ids);
        let queue: Arc<Mutex<Vec<Action>>> = Arc::new(Mutex::new(Vec::new()));
        let q = queue.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if let Some(a) = ids.get(&e.id) {
                if matches!(a, Action::ShowWindow | Action::Settings) {
                    crate::macos::show_window();
                }
                q.lock().push(*a);
                ctx.request_repaint();
            }
        }));
        Some(Self { _menu: menu, tray, queue, ctx: ctx2 })
    }

    pub fn poll(&self) -> Vec<Action> {
        std::mem::take(&mut *self.queue.lock())
    }

    /// Shows or removes the menu bar icon.
    pub fn set_status_item(&self, on: bool) {
        #[cfg(target_os = "macos")]
        {
            let ctx = self.ctx.clone();
            crate::macos::set_status_item(on, self.tray.ns_menu(), Box::new(move || ctx.request_repaint()));
        }
        #[cfg(not(target_os = "macos"))]
        let _ = on;
    }
}
