//! Main window: toolbar, font grid/list, floating preview-text bar, shortcuts, drag and drop.
use crate::app::{Dialog, PrefTab, KernshipApp, SearchBy};
use crate::model::*;
use crate::render::RenderKey;
use crate::store::{Align, InfoMode, SortBy, TextCase, ViewMode, SIZE_MIN};
use crate::theme::*;
use crate::ui_sidebar::{self};
use egui::{pos2, vec2, Align2, Color32, Rect, Sense, Stroke, StrokeKind, Ui};
use std::path::PathBuf;

pub const ICON_MARK: &str = "\u{1}icons";

/// Grid card size:
/// the preview text (or "Hamburgefons" when it is empty)
/// is measured with the system font at the preview size inside a 10000×10000 box;
/// height = round(h) + 48, at least 80; width = round(w) + 20, at least 80 / 0.75;
/// and never narrower than height / 0.75 (ratio 0.75, minimum 80).
pub fn card_size(ui: &Ui, size: f32, preview_text: &str) -> (f32, f32) {
    let sample = if preview_text.trim().is_empty() { "Hamburgefons" } else { preview_text };
    let g = ui.fonts(|f| f.layout(sample.to_string(), ui_font(size), pal().ink, BOUNDING_BOX));
    let mut h = (g.size().y * FACE_H).round() + CARD_EXTRA_H;
    if h < CARD_MIN_H {
        h = CARD_MIN_H;
    }
    let mut w = (g.size().x * FACE_W).round() + CARD_EXTRA_W;
    if w < CARD_MIN_H / CARD_RATIO {
        w = CARD_MIN_H / CARD_RATIO;
    }
    if h / w > CARD_RATIO {
        w = (h / CARD_RATIO).round();
    }
    (w, h)
}
pub fn _list_row_height(size: f32) -> f32 {
    (size * 1.45 + 18.0).max(40.0)
}

const BOUNDING_BOX: f32 = 10000.0;
/// The rules above were measured with the system font; the interface font is narrower and
/// shorter in its line, so its measure is scaled back to the system font's.
const FACE_W: f32 = 1.14;
const FACE_H: f32 = 0.95;
const CARD_EXTRA_H: f32 = 48.0;
const CARD_EXTRA_W: f32 = 20.0;
const CARD_MIN_H: f32 = 80.0;
const CARD_RATIO: f32 = 0.75;
/// Grid layout: header height 26, header padding bottom 8,
/// container padding {12, 12, 90, 12}.
const HEADER_H: f32 = 26.0;
const HEADER_PAD: f32 = 8.0;
const PAD_SIDE: f32 = 12.0;
const PAD_BOTTOM: f32 = 90.0;

/// Opens a click menu under a custom-drawn button.
pub fn click_menu(ui: &Ui, resp: &egui::Response, id: &str, add: impl FnOnce(&mut Ui)) {
    let pid = ui.make_persistent_id(id);
    if resp.clicked() {
        ui.memory_mut(|m| m.toggle_popup(pid));
    }
    egui::popup::popup_below_widget(ui, pid, resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
        crate::theme::menu_style(ui);
        ui.set_min_width(crate::theme::MENU_W);
        add(ui)
    });
}

/// What the menu bar's items do.
pub fn run_action(app: &mut KernshipApp, ctx: &egui::Context, a: crate::menubar::Action, now: f64) {
    use crate::menubar::Action as A;
    let sel = app.selected.clone();
    match a {
        A::Settings => app.dialog = Dialog::Preferences { tab: PrefTab::General },
        A::Import => import_dialog(app, now),
        A::AddFolder => {
            if let Some(f) = pick_folder() {
                let lib = target_library(app, now);
                app.import_paths(lib, vec![f]);
            }
        }
        A::NewLibrary => app.dialog = Dialog::NewLibrary { name: "Untitled Library".into(), folder: None },
        A::OpenLibrary => {
            if let Some(f) = pick_library() {
                app.open_library(f, now);
            }
        }
        A::NewTag => app.dialog = Dialog::NewTag { name: String::new() },
        A::NewSmartFilter => {
            app.dialog = Dialog::SmartFilter {
                filter: SmartFilter { id: new_id(), name: "Untitled Smart Filter".into(), match_all: true, rules: vec![Rule { field: RuleField::Name, op: RuleOp::Contains, value: String::new() }] },
                is_new: true,
            }
        }
        A::Find => app.focus_search = true,
        A::Activate if !sel.is_empty() => app.set_active(&sel, true, now),
        A::ActivateTemporarily if !sel.is_empty() => app.set_active_by(&sel, true, ActivatedBy::Temporary, now),
        A::Deactivate if !sel.is_empty() => app.set_active(&sel, false, now),
        A::Export if !sel.is_empty() => {
            if let Some(dest) = pick_folder() {
                app.export(&sel, dest, now);
            }
        }
        A::ShowInFinder => {
            if let Some(id) = sel.first() {
                app.reveal(id);
            }
        }
        A::RemoveFromSet if !sel.is_empty() => app.remove_from_sets(&sel, now),
        A::DeleteFromLibrary if !sel.is_empty() => app.dialog = Dialog::ConfirmDelete { ids: sel },
        A::Grid => app.st.view_mode = ViewMode::Grid,
        A::List => app.st.view_mode = ViewMode::List,
        A::ExpandFamilies => {
            app.st.expand_families = !app.st.expand_families;
            app.dirty = true;
        }
        A::EditPreviewText => app.focus_preview = true,
        A::IncreaseSize | A::DecreaseSize => {
            let d = if a == A::IncreaseSize { 4.0 } else { -4.0 };
            if app.is_icon_nav() {
                app.st.icon_size = (app.st.icon_size + d).clamp(crate::store::ICON_MIN, crate::store::ICON_MAX);
            } else {
                let v = app.st.preview_size() + d;
                app.st.set_preview_size(v);
            }
        }
        A::ToggleInfo => app.toggle_pane(InfoMode::Info, now),
        A::ToggleSidebar => app.st.sidebar_visible = !app.st.sidebar_visible,
        A::FloatOnTop => {
            app.st.float_on_top = !app.st.float_on_top;
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if app.st.float_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            }));
        }
        A::CheckMissingFonts => {
            app.watcher.check_now();
            app.toast("Checking open documents for missing fonts…", now);
        }
        A::SyncGoogleFonts => app.start_loading(true),
        A::Welcome => app.dialog = Dialog::Welcome,
        A::ToggleDarkMode => {
            let light_now = crate::theme::light_t() > 0.5;
            app.st.prefs.appearance = if light_now { crate::store::Appearance::Dark } else { crate::store::Appearance::Light };
        }
        _ => {}
    }
    app.save_soon(now);
}

pub fn pick_paths() -> Option<Vec<PathBuf>> {
    let script = r#"set f to choose file with prompt "Choose fonts or font folders to import" with multiple selections allowed
set out to ""
repeat with x in f
set out to out & POSIX path of x & linefeed
end repeat
out"#;
    let o = std::process::Command::new("osascript").arg("-e").arg(script).output().ok()?;
    let paths: Vec<PathBuf> = String::from_utf8_lossy(&o.stdout).lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect();
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

/// Chooses a Kernship library to open, which macOS shows as a single item (a package) that
/// the AppleScript choosers cannot pick.
pub fn pick_library() -> Option<PathBuf> {
    crate::macos::choose_folder("Choose a Kernship library", true)
}

pub fn pick_folder() -> Option<PathBuf> {
    let o = std::process::Command::new("osascript")
        .arg("-e")
        .arg("POSIX path of (choose folder with prompt \"Choose a folder of fonts\")")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

pub fn target_library(app: &mut KernshipApp, now: f64) -> String {
    if let Nav::Library(id) | Nav::Collection(id, _) = &app.nav {
        return id.clone();
    }
    if let Some(l) = app.st.libraries.first() {
        return l.id.clone();
    }
    app.create_library("Font Library", None, now)
}

pub fn import_dialog(app: &mut KernshipApp, now: f64) {
    if let Some(paths) = pick_paths() {
        let lib = target_library(app, now);
        app.import_paths(lib, paths);
    }
}

pub fn draw(app: &mut KernshipApp, ctx: &egui::Context, now: f64) {
    track_input(ctx);
    shortcuts(app, ctx, now);

    // Files dropped on the window go into the current (or first) library.
    let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
    if !dropped.is_empty() {
        if app.is_icon_nav() {
            app.import_icon_fonts(dropped, now);
        } else {
            let lib = target_library(app, now);
            app.import_paths(lib, dropped);
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(pal().ground))
        .show(ctx, |ui| {
            let full = ui.max_rect();
            // Empty space along the top moves the window (double-click zooms); controls drawn
            // after this take precedence, so dragging the size slider never moves the window.
            let strip = Rect::from_min_max(full.min, pos2(full.right(), full.top() + TOOLBAR_H));
            let drag = ui.interact(strip, egui::Id::new("window-drag"), Sense::click_and_drag());
            if drag.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
            }
            // Sidebar and info pane slide in and out over 200 ms.
            let sb_t = ui.ctx().animate_bool_with_time(egui::Id::new("sidebar-slide"), app.st.sidebar_visible, ANIM);
            let sb_w = app.st.sidebar_width.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
            // Any change of layout (sidebar sliding, Grid ⇄ List) keeps the selected font centred.
            let mode_now = app.st.view_mode == ViewMode::Grid;
            let mode_before = ui.ctx().memory_mut(|m| {
                let before = m.data.get_temp::<bool>(egui::Id::new("last-mode"));
                m.data.insert_temp(egui::Id::new("last-mode"), mode_now);
                before
            });
            if (sb_t > 0.0 && sb_t < 1.0) || mode_before != Some(mode_now) {
                app.center_selected = app.center_selected.max(3);
            }
            let hidden_left = full.left() + 96.0 + 18.0 + 10.0;
            let shown_left = full.left() + SIDEBAR_INSET + sb_w + 9.0;
            let main_left = hidden_left + (shown_left - hidden_left) * sb_t;
            if sb_t > 0.0 {
                let dx = -(sb_w + SIDEBAR_INSET) * (1.0 - sb_t);
                let panel = Rect::from_min_max(
                    pos2(full.left() + SIDEBAR_INSET + dx, full.top() + SIDEBAR_INSET),
                    pos2(full.left() + SIDEBAR_INSET + sb_w + dx, full.bottom() - SIDEBAR_INSET),
                );
                ui_sidebar::draw(app, ui, panel, now);
                if sb_t >= 1.0 {
                    // The divider between sidebar and fonts: drag it to resize the sidebar.
                    let handle = Rect::from_min_max(pos2(panel.right() - 2.0, panel.top() + 40.0), pos2(panel.right() + 7.0, panel.bottom()));
                    if let Some(w) = divider(ui, handle, "sidebar-divider", sb_w, 1.0) {
                        app.st.sidebar_width = w.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
                        app.center_selected = app.center_selected.max(2);
                        app.save_soon(now);
                    }
                }
            }
            if sb_t < 1.0 {
                // Leave room for the window buttons, then the "show sidebar" button.
                let b = Rect::from_center_size(pos2(full.left() + 96.0, full.top() + 26.0), vec2(32.0, 32.0));
                let r = circle_button(ui, b, "show-sidebar", false, "Show Sidebar");
                let c = b.center();
                let col = pal().ink.gamma_multiply(1.0 - sb_t);
                ui.painter().rect_stroke(Rect::from_center_size(c, vec2(18.0, 14.0)), 3.0, Stroke::new(1.2_f32, col), StrokeKind::Middle);
                ui.painter().line_segment([c + vec2(-3.5, -7.0), c + vec2(-3.5, 7.0)], Stroke::new(1.2_f32, col));
                if r.clicked() {
                    app.st.sidebar_visible = true;
                }
            }
            let info_t = ui.ctx().animate_bool_with_time(egui::Id::new("info-slide"), app.st.info_visible, ANIM);
            if info_t > 0.0 && info_t < 1.0 {
                // While the info pane slides, keep the selected font in the middle of the view.
                app.center_selected = app.center_selected.max(2);
            }
            let iw = app.st.info_width.clamp(INFO_MIN, INFO_MAX);
            let info_w = iw + 8.0;
            let main_right = full.right() - 8.0 - info_w * info_t;
            if info_t > 0.0 {
                let dx = (info_w + SIDEBAR_INSET) * (1.0 - info_t);
                let panel = Rect::from_min_max(
                    pos2(full.right() - SIDEBAR_INSET - iw + dx, full.top() + TOOLBAR_H),
                    pos2(full.right() - SIDEBAR_INSET + dx, full.bottom() - SIDEBAR_INSET),
                );
                crate::ui_panels::info_panel(app, ui, panel, now);
                if info_t >= 1.0 {
                    let handle = Rect::from_min_max(pos2(panel.left() - 7.0, panel.top()), pos2(panel.left() + 2.0, panel.bottom()));
                    if let Some(w) = divider(ui, handle, "info-divider", iw, -1.0) {
                        app.st.info_width = w.clamp(INFO_MIN, INFO_MAX);
                        app.center_selected = app.center_selected.max(2);
                        app.save_soon(now);
                    }
                }
            }
            let toolbar = Rect::from_min_max(pos2(main_left, full.top()), pos2(full.right() - 8.0, full.top() + TOOLBAR_H));
            toolbar_ui(app, ui, toolbar, now);
            let mut content_top = full.top() + TOOLBAR_H - 2.0;
            let style_t = ui.ctx().animate_bool_with_time(egui::Id::new("style-bar"), app.st.show_style_bar, ANIM);
            if style_t > 0.0 {
                let r = Rect::from_min_max(pos2(main_left + 2.0, content_top + 2.0), pos2(main_right, content_top + 32.0));
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
                child.set_opacity(style_t);
                style_bar(app, &mut child, r, now);
                content_top += 36.0 * style_t;
            }
            let content = Rect::from_min_max(pos2(main_left, content_top), pos2(main_right, full.bottom()));
            if app.is_icon_nav() {
                crate::ui_icons::draw(app, ui, content, now);
            } else {
                content_ui(app, ui, content, now);
                preview_bar(app, ui, content, now);
            }
            toasts(app, ui, content, now);
        });

    // A drag of fonts onto a library, set or tag in the sidebar.
    let dragging = ctx.memory(|m| m.data.get_temp::<bool>(egui::Id::new("dragging-fonts")).unwrap_or(false));
    if dragging {
        if let Some(pos) = ctx.pointer_latest_pos() {
            let n = app.selected.len();
            let label = if n == 1 { app.font(&app.selected[0]).map(|f| f.display_name()).unwrap_or_default() } else { format!("{n} fonts") };
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag-ghost")));
            let g = painter.layout_no_wrap(label, ui_medium(12.0), pal().ground);
            let r = Rect::from_min_size(pos + vec2(12.0, 8.0), g.size() + vec2(16.0, 8.0));
            painter.rect_filled(r, CTRL_RADIUS, pal().ink);
            painter.galley(r.min + vec2(8.0, 4.0), g, pal().ground);
        }
        if ctx.input(|i| i.pointer.any_released()) {
            ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("dragging-fonts"), false));
        }
    }

    // Test picture of the font menu (screenshot runs with KERNSHIP_SHOT_VIEW=menu only).
    if app.screenshot.is_some() && std::env::var("KERNSHIP_SHOT_VIEW").as_deref() == Ok("menu") && app.font("gf:ABeeZee:400").is_some() {
        egui::Area::new(egui::Id::new("shot-menu")).fixed_pos(pos2(520.0, 160.0)).order(egui::Order::Foreground).show(ctx, |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                ui.set_max_width(crate::theme::MENU_W);
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| font_menu(app, ui, now));
            });
        });
    }

    crate::ui_panels::dialogs(app, ctx, now);
}

fn shortcuts(app: &mut KernshipApp, ctx: &egui::Context, now: f64) {
    let typing = ctx.wants_keyboard_input();
    // "Shortcut to check missing fonts" (works while Kernship is in front).
    if app.st.prefs.check_shortcut_enabled && !app.recording_shortcut {
        if let (Some(rec), Some(pressed)) = (app.st.prefs.check_shortcut.clone(), crate::ui_settings::pressed_shortcut(ctx)) {
            if rec == pressed {
                app.watcher.check_now();
                app.toast("Checking open documents for missing fonts…", now);
            }
        }
    }
    if app.is_icon_nav() && !typing && !app.icon_selected.is_empty() {
        let copy = ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Copy)));
        if copy {
            app.copy_selected_svg(now);
        }
    }
    let keys = ctx.input(|i| i.events.clone());
    for e in keys {
        let egui::Event::Key { key, pressed: true, modifiers: m, .. } = e else { continue };
        use egui::Key;
        let sel = app.selected.clone();
        if m.command {
            match key {
                // File
                Key::O if m.alt => {
                    if let Some(f) = pick_folder() {
                        let lib = target_library(app, now);
                        app.import_paths(lib, vec![f]);
                    }
                }
                Key::O if m.ctrl => {
                    if let Some(f) = pick_library() {
                        app.open_library(f, now);
                    }
                }
                Key::O => import_dialog(app, now),
                Key::N if m.shift => app.dialog = Dialog::NewTag { name: String::new() },
                Key::N if m.alt => {
                    app.dialog = Dialog::SmartFilter {
                        filter: SmartFilter { id: new_id(), name: "Untitled Smart Filter".into(), match_all: true, rules: vec![Rule { field: RuleField::Name, op: RuleOp::Contains, value: String::new() }] },
                        is_new: true,
                    }
                }
                Key::N => app.dialog = Dialog::NewLibrary { name: "Untitled Library".into(), folder: None },
                Key::F => app.focus_search = true,
                // Font
                Key::R if m.alt && !sel.is_empty() => app.set_active_by(&sel, true, ActivatedBy::Temporary, now),
                Key::R if m.shift && !sel.is_empty() => app.set_active(&sel, false, now),
                Key::R if !sel.is_empty() => app.set_active(&sel, true, now),
                Key::Backspace if !typing && !sel.is_empty() => app.dialog = Dialog::ConfirmDelete { ids: sel },
                // View
                Key::Num1 if m.alt => {
                    app.st.sort_by = SortBy::Random;
                    app.random_seed = app.random_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    app.dirty = true;
                }
                Key::Num1 => app.st.view_mode = ViewMode::Grid,
                Key::Num2 => app.st.view_mode = ViewMode::List,
                Key::G => {
                    app.st.expand_families = !app.st.expand_families;
                    app.dirty = true;
                }
                Key::E => app.focus_preview = true,
                Key::I => app.toggle_pane(InfoMode::Info, now),
                Key::Plus | Key::Equals => {
                    let v = app.st.preview_size() + 4.0;
                    app.st.set_preview_size(v);
                }
                Key::Minus => {
                    let v = app.st.preview_size() - 4.0;
                    app.st.set_preview_size(v);
                }
                Key::T if m.ctrl => {
                    app.st.float_on_top = !app.st.float_on_top;
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if app.st.float_on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    }));
                }
                Key::Comma => app.dialog = Dialog::Preferences { tab: PrefTab::General },
                Key::A if !typing => {
                    app.selected = app.groups.iter().flat_map(|g| g.faces.iter().map(|&i| app.all[i].id.clone())).collect();
                }
                Key::S if m.ctrl => app.st.sidebar_visible = !app.st.sidebar_visible,
                _ => {}
            }
            app.save_soon(now);
        } else if !typing {
            match key {
                Key::Space => {
                    if let Some(id) = sel.first().cloned() {
                        app.dialog = if app.dialog == (Dialog::Details { font_id: id.clone() }) { Dialog::None } else { Dialog::Details { font_id: id } };
                    }
                }
                Key::Escape => {
                    if app.dialog != Dialog::None {
                        app.dialog = Dialog::None;
                    } else {
                        app.selected.clear();
                    }
                }
                Key::Enter if !sel.is_empty() => {
                    let on = !sel.iter().all(|id| app.font(id).map(|f| app.is_active(f)).unwrap_or(false));
                    app.set_active(&sel, on, now);
                }
                Key::ArrowRight | Key::ArrowLeft | Key::ArrowDown | Key::ArrowUp => {
                    let flat: Vec<String> = app.groups.iter().flat_map(|g| g.faces.iter().map(|&i| app.all[i].id.clone())).collect();
                    if flat.is_empty() {
                        continue;
                    }
                    let cur = sel.first().and_then(|s| flat.iter().position(|x| x == s));
                    let next = match (cur, key) {
                        (None, _) => 0,
                        (Some(i), Key::ArrowRight | Key::ArrowDown) => (i + 1).min(flat.len() - 1),
                        (Some(i), _) => i.saturating_sub(1),
                    };
                    app.selected = vec![flat[next].clone()];
                }
                _ => {}
            }
        }
    }
}

const SIDEBAR_MIN: f32 = 170.0;
const SIDEBAR_MAX: f32 = 380.0;
const INFO_MIN: f32 = 260.0;
const INFO_MAX: f32 = 520.0;

/// A draggable divider; returns the new width while it is dragged (`dir` -1 when the panel is
/// on the right, so dragging left makes it wider).
fn divider(ui: &mut Ui, handle: Rect, id: &str, width: f32, dir: f32) -> Option<f32> {
    let r = ui.interact(handle, egui::Id::new(id), Sense::drag());
    let t = fade(ui, id, r.hovered() || r.dragged());
    if t > 0.0 {
        let x = handle.center().x;
        ui.painter().line_segment(
            [pos2(x, handle.top() + 4.0), pos2(x, handle.bottom() - 4.0)],
            Stroke::new(2.0_f32, pal().ink.gamma_multiply(t)),
        );
    }
    if r.hovered() || r.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    if r.dragged() {
        let start = ui.ctx().memory_mut(|m| *m.data.get_temp_mut_or_insert_with(egui::Id::new((id, "start")), || width));
        let origin = ui.input(|i| i.pointer.press_origin()).map(|p| p.x).unwrap_or(0.0);
        let delta = r.interact_pointer_pos().map(|p| p.x - origin).unwrap_or(0.0);
        return Some(start + delta * dir);
    }
    if r.drag_stopped() || !r.dragged() {
        ui.ctx().memory_mut(|m| m.data.remove::<f32>(egui::Id::new((id, "start"))));
    }
    None
}

/// A square toolbar button. What is on is an ink fill; the second value is the colour its
/// icon takes on that fill.
fn tool_button(ui: &mut Ui, rect: Rect, id: &str, selected: bool, tip: &str) -> (egui::Response, Color32) {
    let c = pal();
    let r = ui.interact(rect, ui.id().with(id), Sense::click()).on_hover_text(tip);
    let ht = fade(ui, ("cb-h", id), r.hovered());
    let st = fade(ui, ("cb-s", id), selected);
    ui.painter().rect_filled(rect, CTRL_RADIUS, mix(mix(c.panel, c.panel_hi, ht), c.ink, st));
    focus_ring(ui, &r, rect, CTRL_RADIUS);
    (r, mix(c.ink, c.ground, st))
}

fn circle_button(ui: &mut Ui, rect: Rect, id: &str, selected: bool, tip: &str) -> egui::Response {
    tool_button(ui, rect, id, selected, tip).0
}

/// 7986 → "7,986".
pub fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// The sign band: where you are, what is selected and how many fonts are listed, read like a
/// direction sign ("Google Fonts → Abhaya Libre → SemiBold   7,986 styles").
fn sign_band(app: &KernshipApp, ui: &Ui, rect: Rect) {
    let c = pal();
    let p = ui.painter().with_clip_rect(rect);
    p.rect_filled(rect, CTRL_RADIUS, c.ink);
    let cy = rect.center().y;
    let mut right = rect.right() - 12.0;
    let icons = app.is_icon_nav();
    let label = if icons {
        let n = app.nav_count(&app.nav);
        format!("{} {}", thousands(n), if n == 1 { "icon" } else { "icons" })
    } else {
        let n: usize = app.groups.iter().map(|g| g.faces.len()).sum();
        let on = app.groups.iter().flat_map(|g| g.faces.iter()).filter(|&&i| app.all[i].source != Source::System && app.is_active(&app.all[i])).count();
        let styles = format!("{} {}", thousands(n), if n == 1 { "style" } else { "styles" });
        if on > 0 {
            format!("{styles}, {} active", thousands(on))
        } else {
            styles
        }
    };
    let r = p.text(pos2(right, cy), Align2::RIGHT_CENTER, label, ui_bold(13.0), c.ground);
    right = r.left() - 12.0;
    p.line_segment([pos2(right, rect.top() + 7.0), pos2(right, rect.bottom() - 7.0)], Stroke::new(1.0_f32, c.ground.gamma_multiply(0.4)));
    right -= 10.0;
    let mut parts: Vec<(String, bool)> = Vec::new();
    if icons {
        parts.push(("Icon Fonts".to_string(), false));
        if let Nav::IconFamily(id) = &app.nav {
            if let Some(set) = app.icon_set(id) {
                parts.push((set.name.clone(), true));
            }
        }
    } else {
        parts.push((app.nav_title(), false));
        match app.selected.len() {
            0 => {}
            1 => {
                if let Some(f) = app.font(&app.selected[0]) {
                    parts.push((f.family.clone(), false));
                    parts.push((f.style.clone(), true));
                }
            }
            n => parts.push((format!("{n} fonts selected"), true)),
        }
    }
    // Lay the parts out left to right; a part that does not fit is left out with its arrow
    // (the name of the place always stays).
    let p = ui.painter().with_clip_rect(Rect::from_min_max(rect.min, pos2(right, rect.max.y)));
    let mut x = rect.left() + 12.0;
    for (i, (text, strong)) in parts.iter().enumerate() {
        let font = if *strong { ui_bold(13.0) } else { ui_medium(13.0) };
        let w = ui.fonts(|f| f.layout_no_wrap(text.clone(), font.clone(), c.ground).size().x);
        let lead = if i > 0 { 22.0 } else { 0.0 };
        if i > 0 && x + lead + w > right {
            break;
        }
        if i > 0 {
            let s = Stroke::new(1.4_f32, c.ground);
            let ac = pos2(x + 7.0, cy);
            p.line_segment([ac + vec2(-5.0, 0.0), ac + vec2(5.0, 0.0)], s);
            p.line_segment([ac + vec2(1.5, -3.5), ac + vec2(5.0, 0.0)], s);
            p.line_segment([ac + vec2(1.5, 3.5), ac + vec2(5.0, 0.0)], s);
            x += lead;
        }
        p.text(pos2(x, cy), Align2::LEFT_CENTER, text, font, c.ground);
        x += w + 8.0;
    }
}

fn toolbar_ui(app: &mut KernshipApp, ui: &mut Ui, bar: Rect, now: f64) {
    let p = ui.painter().clone();
    let c = pal();
    let cy = bar.top() + 26.0;
    let h = 32.0;

    // Preview size slider "A ──●── A" (8–256 pt in grid, 8–128 pt in list).
    let pill = Rect::from_min_size(pos2(bar.left() + 1.0, cy - h / 2.0), vec2(116.0, h));
    p.rect_filled(pill, CTRL_RADIUS, c.panel);
    p.text(pos2(pill.left() + 12.0, cy), Align2::CENTER_CENTER, "A", ui_medium(10.0), c.ink);
    p.text(pos2(pill.right() - 13.0, cy), Align2::CENTER_CENTER, "A", ui_medium(15.0), c.ink);
    let track = Rect::from_min_max(pos2(pill.left() + 24.0, cy - 1.0), pos2(pill.right() - 27.0, cy + 1.0));
    let icons = app.is_icon_nav();
    let (min, max, cur) = if icons {
        (crate::store::ICON_MIN, crate::store::ICON_MAX, app.st.icon_size)
    } else {
        (SIZE_MIN, app.st.size_max(), app.st.preview_size())
    };
    let t = ((cur - min) / (max - min)).clamp(0.0, 1.0).sqrt();
    let knob_x = track.left() + t * track.width();
    p.rect_filled(track, 1.0, c.rule_strong);
    p.rect_filled(Rect::from_min_max(track.min, pos2(knob_x, track.max.y)), 1.0, c.ink);
    let sr = ui.interact(pill, ui.id().with("size-slider"), Sense::click_and_drag());
    if sr.dragged() || sr.clicked() {
        if let Some(pos) = sr.interact_pointer_pos() {
            let nt = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
            // Square curve so small sizes get more of the track.
            let v = (min + nt * nt * (max - min)).round();
            if icons {
                app.st.icon_size = v;
            } else {
                app.st.set_preview_size(v);
            }
            app.save_soon(now);
        }
    }
    // The slider answers to the arrow keys when it has keyboard focus.
    if sr.has_focus() {
        let step = ui.input(|i| i.key_pressed(egui::Key::ArrowRight) as i32 - i.key_pressed(egui::Key::ArrowLeft) as i32) as f32;
        if step != 0.0 {
            let v = (cur + step * 2.0).clamp(min, max);
            if icons {
                app.st.icon_size = v;
            } else {
                app.st.set_preview_size(v);
            }
            app.save_soon(now);
        }
    }
    p.circle_filled(pos2(knob_x, cy), 7.0, c.ink);
    focus_ring(ui, &sr, pill, CTRL_RADIUS);
    sr.on_hover_text(format!("{} pt", cur as i32));

    // Right side: style bar toggle, grid/list, convert, info, search.
    let search_w = 260.0f32.min((bar.width() * 0.3).max(170.0));
    let search = Rect::from_min_size(pos2(bar.right() - search_w, cy - h / 2.0), vec2(search_w, h));
    let info = Rect::from_min_size(pos2(search.left() - 12.0 - h, cy - h / 2.0), vec2(h, h));
    let web = Rect::from_min_size(pos2(info.left() - 6.0 - h, cy - h / 2.0), vec2(h, h));
    let seg = Rect::from_min_size(pos2(web.left() - 12.0 - 68.0, cy - h / 2.0), vec2(68.0, h));
    let ibeam = Rect::from_min_size(pos2(seg.left() - 6.0 - h, cy - h / 2.0), vec2(h, h));

    let band = Rect::from_min_max(pos2(pill.right() + 12.0, cy - h / 2.0), pos2(ibeam.left() - 12.0, cy + h / 2.0));
    if band.width() >= 150.0 {
        sign_band(app, ui, band);
    }

    let (r, ic) = tool_button(ui, ibeam, "tb-style", app.st.show_style_bar, "Style Bar");
    let cc = ibeam.center();
    let s = Stroke::new(1.3_f32, ic);
    p.line_segment([cc + vec2(-8.5, -4.5), cc + vec2(-3.0, -4.5)], s);
    p.line_segment([cc + vec2(-8.5, 4.5), cc + vec2(-3.0, 4.5)], s);
    p.line_segment([cc + vec2(-8.5, -4.5), cc + vec2(-8.5, 4.5)], s);
    p.line_segment([cc + vec2(3.0, -4.5), cc + vec2(8.5, -4.5)], s);
    p.line_segment([cc + vec2(3.0, 4.5), cc + vec2(8.5, 4.5)], s);
    p.line_segment([cc + vec2(8.5, -4.5), cc + vec2(8.5, 4.5)], s);
    p.line_segment([cc + vec2(0.0, -6.5), cc + vec2(0.0, 6.5)], s);
    p.line_segment([cc + vec2(-2.5, -6.5), cc + vec2(2.5, -6.5)], s);
    p.line_segment([cc + vec2(-2.5, 6.5), cc + vec2(2.5, 6.5)], s);
    if r.clicked() {
        app.st.show_style_bar = !app.st.show_style_bar;
        app.save_soon(now);
    }

    p.rect_filled(seg, CTRL_RADIUS, c.panel);
    let g = Rect::from_min_size(seg.min, vec2(34.0, h));
    let l = Rect::from_min_size(pos2(seg.center().x, seg.top()), vec2(34.0, h));
    let gr = ui.interact(g, ui.id().with("tb-grid"), Sense::click()).on_hover_text("Grid View");
    let lr = ui.interact(l, ui.id().with("tb-list"), Sense::click()).on_hover_text("List View");
    // The selection slides between Grid and List.
    let k = ui.ctx().animate_value_with_time(ui.id().with("view-seg"), if app.st.view_mode == ViewMode::Grid { 0.0 } else { 1.0 }, ANIM);
    p.rect_filled(g.translate(vec2((l.left() - g.left()) * k, 0.0)), CTRL_RADIUS, c.ink);
    let (g_ic, l_ic) = (mix(c.ground, c.ink, k), mix(c.ink, c.ground, k));
    let gc = g.center();
    for (dx, dy) in [(-3.6, -3.6), (3.6, -3.6), (-3.6, 3.6), (3.6, 3.6)] {
        p.rect_stroke(Rect::from_center_size(gc + vec2(dx, dy), vec2(5.5, 5.5)), 1.2, Stroke::new(1.3_f32, g_ic), StrokeKind::Middle);
    }
    let lc = l.center();
    for dy in [-5.0, 0.0, 5.0] {
        p.circle_filled(lc + vec2(-6.0, dy), 1.1, l_ic);
        p.line_segment([lc + vec2(-3.0, dy), lc + vec2(7.0, dy)], Stroke::new(1.3_f32, l_ic));
    }
    focus_ring(ui, &gr, g, CTRL_RADIUS);
    focus_ring(ui, &lr, l, CTRL_RADIUS);
    if gr.clicked() {
        app.st.view_mode = ViewMode::Grid;
        app.save_soon(now);
    }
    if lr.clicked() {
        app.st.view_mode = ViewMode::List;
        app.save_soon(now);
    }

    let web_on = app.st.info_visible && app.st.info_mode == InfoMode::Web;
    let (r, ic) = tool_button(ui, web, "tb-web", web_on, "Convert Font for Web");
    crate::theme::paint_convert_icon(&p, web.center(), 8.0, ic);
    if r.clicked() {
        app.toggle_pane(InfoMode::Web, now);
    }

    let (r, ic) = tool_button(ui, info, "tb-info", app.st.info_visible && !web_on, "Info");
    p.circle_stroke(info.center(), 7.5, Stroke::new(1.3_f32, ic));
    p.line_segment([info.center() + vec2(0.0, -0.5), info.center() + vec2(0.0, 3.8)], Stroke::new(1.5_f32, ic));
    p.circle_filled(info.center() + vec2(0.0, -3.4), 0.95, ic);
    if r.clicked() {
        app.toggle_pane(InfoMode::Info, now);
    }

    p.rect_filled(search, CTRL_RADIUS, c.field);
    let mc = pos2(search.left() + 17.0, cy);
    let mag = ui.interact(Rect::from_center_size(mc, vec2(26.0, 26.0)), ui.id().with("search-by"), Sense::click()).on_hover_text("Filters");
    let filtered = app.filters_on() || app.search_by != SearchBy::All;
    if filtered {
        // Filters are on: the magnifier sits on an ink square.
        p.rect_filled(Rect::from_center_size(mc, vec2(22.0, 22.0)), 4.0, c.ink);
    }
    let mcol = if filtered { c.ground } else { c.dim };
    p.circle_stroke(mc + vec2(-1.0, -1.0), 4.6, Stroke::new(1.4_f32, mcol));
    p.line_segment([mc + vec2(2.4, 2.4), mc + vec2(5.6, 5.6)], Stroke::new(1.6_f32, mcol));
    if mag.clicked() {
        // The magnifier opens the filter panel, like focusing the field does.
        app.search_pop = true;
        app.focus_search = true;
    }
    let field = Rect::from_min_max(pos2(search.left() + 34.0, cy - 10.0), pos2(search.right() - 26.0, cy + 10.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(field));
    let hint = if app.search_by == SearchBy::All { "Search".to_string() } else { format!("Search {}", SearchBy::ALL.iter().find(|x| x.0 == app.search_by).unwrap().1) };
    let te = egui::TextEdit::singleline(&mut app.search)
        .hint_text(egui::RichText::new(hint).color(c.dim))
        .frame(false)
        .font(ui_font(14.0))
        .text_color(c.ink)
        .desired_width(field.width());
    let resp = child.add(te);
    if app.focus_search {
        resp.request_focus();
        app.focus_search = false;
    }
    if resp.changed() {
        app.dirty = true;
    }
    // Focusing the search field opens the filter panel below it.
    if resp.gained_focus() {
        app.search_pop = true;
    }
    let ring = ui.ctx().animate_bool_with_time(ui.id().with("search-ring"), resp.has_focus(), ANIM);
    p.rect_stroke(search, CTRL_RADIUS, Stroke::new(1.0 + ring, mix(c.rule_strong, c.ink, ring)), StrokeKind::Inside);
    if !app.search.is_empty() {
        let xr = Rect::from_center_size(pos2(search.right() - 15.0, cy), vec2(18.0, 18.0));
        let xresp = ui.interact(xr, ui.id().with("search-clear"), Sense::click()).on_hover_text("Clear");
        p.circle_filled(xr.center(), 7.5, if xresp.hovered() { c.ink } else { c.faint });
        paint_cross(&p, xr.center(), 2.8, c.ground);
        if xresp.clicked() {
            app.search.clear();
            app.dirty = true;
        }
    }
    crate::ui_search::search_popover(app, ui, search, now);
}

/// The filter panel's gear menu: Search by, Sort By, Expand Font Families, Reset Filters.
pub fn filter_menu(app: &mut KernshipApp, ui: &mut Ui) {
    ui.set_min_width(230.0);
    ui.label(egui::RichText::new("Search by").small().color(pal().dim));
    for (k, n) in SearchBy::ALL {
        if ui.selectable_label(app.search_by == k, n).clicked() {
            app.search_by = k;
            app.dirty = true;
        }
    }
    ui.separator();
    crate::theme::submenu(ui, "Sort By", |ui| {
        for (k, n) in SortBy::ALL {
            if ui.selectable_label(app.st.sort_by == k, n).clicked() {
                if k == SortBy::Random {
                    app.random_seed = app.random_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                }
                app.st.sort_by = k;
                app.dirty = true;
            }
        }
        ui.separator();
        if ui.selectable_label(app.st.sort_ascending, "Ascending").clicked() {
            app.st.sort_ascending = true;
            app.dirty = true;
        }
        if ui.selectable_label(!app.st.sort_ascending, "Descending").clicked() {
            app.st.sort_ascending = false;
            app.dirty = true;
        }
    });
    if ui.checkbox(&mut app.st.expand_families, "Expand Font Families").changed() {
        app.dirty = true;
    }
    if app.filters_on() || app.search_by != SearchBy::All {
        ui.separator();
        if crate::theme::mi(ui, "Reset Filters") {
            app.clear_filters();
            app.search_by = SearchBy::All;
            app.dirty = true;
        }
    }
}

fn style_bar(app: &mut KernshipApp, ui: &mut Ui, rect: Rect, now: f64) {
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
    let ui = &mut child;
    ui.spacing_mut().item_spacing.x = 6.0;
    let mut changed = false;
    for (a, label) in [(Align::Left, "Left"), (Align::Center, "Center"), (Align::Right, "Right")] {
        if toggle(ui, label, app.st.align == a).on_hover_text("Align").clicked() {
            app.st.align = a;
            changed = true;
        }
    }
    ui.separator();
    for (c, label) in [(TextCase::None, "Aa"), (TextCase::Upper, "AA"), (TextCase::Lower, "aa")] {
        if toggle(ui, label, app.st.case == c).on_hover_text("Text Case").clicked() {
            app.st.case = c;
            changed = true;
        }
    }
    ui.separator();
    ui.label(egui::RichText::new("Letter space").color(pal().dim));
    if ui.add(egui::Slider::new(&mut app.st.letter_spacing, -10.0..=40.0).show_value(true).suffix("%")).changed() {
        changed = true;
    }
    if changed {
        app.save_soon(now);
    }
}

enum Row {
    Header(usize),
    Cards(usize, usize, usize),
    /// Collapsed families (Expand Font Families off): one card per family, groups a..b.
    Families(usize, usize),
    Tail,
}

/// The style a collapsed family shows: Regular when there is one.
pub fn representative(app: &KernshipApp, gi: usize) -> usize {
    let g = &app.groups[gi];
    g.faces
        .iter()
        .copied()
        .min_by_key(|&i| (app.all[i].italic, (app.all[i].weight as i32 - 400).abs()))
        .unwrap_or(g.faces[0])
}

fn content_ui(app: &mut KernshipApp, ui: &mut Ui, area: Rect, now: f64) {
    let size = app.st.preview_size();
    let mode = app.st.view_mode;
    // The content starts right after the sidebar; the layout's own side padding is 12.
    let pad_left = PAD_SIDE - 8.0;
    let inner_w = area.width() - pad_left - 4.0;
    let (min_w, card_h) = card_size(ui, size, &app.st.preview_text);
    let (cols, card_w, row_h) = match mode {
        ViewMode::Grid => {
            // columns = max(1, floor((width + spacing) / (minWidth + spacing))),
            // item width = (width - spacing × (columns - 1)) / columns.
            let cols = (((inner_w + CARD_GAP) / (min_w + CARD_GAP)).floor() as usize).max(1);
            let w = (inner_w - CARD_GAP * (cols as f32 - 1.0)) / cols as f32;
            (cols, w, card_h + CARD_GAP)
        }
        ViewMode::List => (1, inner_w, 0.0),
    };

    // Lay out rows (headers, card rows) once per frame; only visible ones are drawn.
    let mut rows: Vec<(f32, f32, Row)> = Vec::new();
    let mut y = 0.0;
    if !app.st.expand_families {
        y = 8.0;
        let mut gi = 0;
        while gi < app.groups.len() {
            let end = (gi + cols).min(app.groups.len());
            rows.push((y, row_h, Row::Families(gi, end)));
            y += row_h;
            gi = end;
        }
    }
    for (gi, g) in app.groups.iter().enumerate().filter(|_| app.st.expand_families) {
        rows.push((y, HEADER_H + HEADER_PAD, Row::Header(gi)));
        y += HEADER_H + HEADER_PAD;
        let mut i = 0;
        while i < g.faces.len() {
            let end = (i + cols).min(g.faces.len());
            rows.push((y, row_h, Row::Cards(gi, i, end)));
            y += row_h;
            i = end;
        }
        if mode == ViewMode::List {
            rows.push((y, CARD_GAP, Row::Tail));
            y += CARD_GAP;
        }
    }
    let total_h = y + PAD_BOTTOM;

    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area));
    child.set_clip_rect(area);
    if app.groups.is_empty() {
        let msg = if app.loading_catalog && matches!(app.nav, Nav::GoogleFonts) {
            "Loading Google Fonts…".to_string()
        } else if app.loading_system && matches!(app.nav, Nav::AllFonts | Nav::SystemFonts) {
            "Loading fonts…".to_string()
        } else if let (Nav::GoogleFonts, Some(e)) = (&app.nav, &app.catalog_error) {
            format!("{e}\nClick below to download and sync Google Fonts.")
        } else if !app.search.is_empty() || app.filters_on() {
            "No Results\nNo fonts match your search.".to_string()
        } else {
            match app.nav {
                Nav::Starred => "No Starred Fonts\nStar fonts to access them quickly.".into(),
                Nav::BuiltinSmart(1) => "No Fonts\nFonts auto-activated by apps will appear here.".into(),
                Nav::SmartFilter(_) | Nav::BuiltinSmart(_) => "No Fonts\nFonts matching this smart filter will appear here.".into(),
                _ => "No Fonts\nDrag fonts here or click the button below.".into(),
            }
        };
        let mut lines = msg.lines();
        let title = lines.next().unwrap_or_default().to_string();
        let sub = lines.next().unwrap_or_default().to_string();
        let c = area.center() - vec2(0.0, 30.0);
        child.painter().text(c, Align2::CENTER_CENTER, title, ui_bold(18.0), pal().ink);
        child.painter().text(c + vec2(0.0, 25.0), Align2::CENTER_CENTER, sub, ui_font(13.5), pal().dim);
        let wants_button = matches!(app.nav, Nav::Library(_) | Nav::Collection(..) | Nav::AllFonts)
            || (app.nav == Nav::GoogleFonts && app.catalog_error.is_some());
        if wants_button && app.search.is_empty() {
            let label = if app.nav == Nav::GoogleFonts { "Sync Google Fonts" } else { "Import Fonts…" };
            let b = Rect::from_center_size(c + vec2(0.0, 64.0), vec2(170.0, 32.0));
            let mut bu = child.new_child(egui::UiBuilder::new().max_rect(b));
            if button_w(&mut bu, label, Btn::Primary, true, b.width(), b.height()) {
                if app.nav == Nav::GoogleFonts {
                    app.start_loading(true);
                } else {
                    import_dialog(app, now);
                }
            }
        }
        return;
    }

    if mode == ViewMode::List {
        return crate::ui_list::draw(app, &mut child, area, now);
    }

    // When the layout changes (info pane, divider, sidebar), keep the selected font where the
    // eye is: its row goes to the middle of the window.
    let mut target: Option<f32> = None;
    if app.center_selected > 0 {
        app.center_selected -= 1;
        if let Some(sel) = app.selected.first().and_then(|id| app.by_id.get(id).copied()) {
            for (ry, rh, row) in rows.iter() {
                let hit = match *row {
                    Row::Cards(gi, a, b) => app.groups[gi].faces[a..b].contains(&sel),
                    Row::Families(a, b) => (a..b).any(|gi| representative(app, gi) == sel),
                    _ => false,
                };
                if hit {
                    target = Some((ry + rh / 2.0 - area.height() / 2.0).clamp(0.0, (total_h - area.height()).max(0.0)));
                    break;
                }
            }
        }
        ui.ctx().request_repaint();
    }
    let scroll_id = format!("content-{:?}", app.nav);
    let mut sa = egui::ScrollArea::vertical()
        .id_salt(scroll_id)
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);
    if let Some(t) = target {
        sa = sa.vertical_scroll_offset(t);
    }
    sa.show_viewport(&mut child, |ui, vp| {
            ui.set_height(total_h);
            let origin = ui.min_rect().min;
            let ppp = ui.ctx().pixels_per_point();
            for (ry, rh, row) in rows.iter() {
                if ry + rh < vp.min.y - 4.0 || *ry > vp.max.y + 4.0 {
                    continue;
                }
                let top = origin.y + ry;
                match *row {
                    Row::Header(gi) => {
                        let g = &app.groups[gi];
                        let cyh = top + HEADER_H / 2.0;
                        ui.painter().text(pos2(origin.x + pad_left + 2.0, cyh), Align2::LEFT_CENTER, &g.family, ui_bold(14.0), pal().ink);
                        let n = g.faces.len();
                        let label = format!("{n} style{}", if n == 1 { "" } else { "s" });
                        ui.painter().text(pos2(origin.x + pad_left + inner_w - 2.0, cyh), Align2::RIGHT_CENTER, label, ui_font(12.0), pal().dim);
                        let hr = Rect::from_min_size(pos2(origin.x, top), vec2(inner_w, 26.0));
                        let resp = ui.interact(hr, ui.id().with(("fam", gi)), Sense::click());
                        let family = g.family.clone();
                        resp.context_menu(|ui| {
            crate::theme::menu_style(ui);
                            let ids = app.family_ids(&family);
                            if crate::theme::mi(ui, "Activate Family") {
                                app.set_active(&ids, true, now);
                                ui.close_menu();
                            }
                            if crate::theme::mi(ui, "Deactivate Family") {
                                app.set_active(&ids, false, now);
                                ui.close_menu();
                            }
                            if crate::theme::mi(ui, "Select Family") {
                                app.selected = ids;
                                ui.close_menu();
                            }
                            if crate::theme::mi(ui, "Copy Family Name") {
                                ui.ctx().copy_text(family.clone());
                                ui.close_menu();
                            }
                        });
                    }
                    Row::Cards(gi, a, b) => {
                        for (k, fi) in (a..b).enumerate() {
                            let idx = app.groups[gi].faces[fi];
                            let x = origin.x + pad_left + k as f32 * (card_w + if mode == ViewMode::Grid { CARD_GAP } else { 0.0 });
                            let h = if mode == ViewMode::Grid { card_h } else { *rh };
                            let rect = Rect::from_min_size(pos2(x, top), vec2(card_w, h));
                            if mode == ViewMode::Grid {
                                card(app, ui, rect, idx, None, ppp, now);
                            } else {
                                list_row(app, ui, rect, idx, ppp, now);
                            }
                        }
                    }
                    Row::Families(a, b) => {
                        for (k, gi) in (a..b).enumerate() {
                            let idx = representative(app, gi);
                            let n = app.groups[gi].faces.len();
                            let x = origin.x + pad_left + k as f32 * (card_w + if mode == ViewMode::Grid { CARD_GAP } else { 0.0 });
                            let h = if mode == ViewMode::Grid { card_h } else { *rh };
                            let rect = Rect::from_min_size(pos2(x, top), vec2(card_w, h));
                            if mode == ViewMode::Grid {
                                let label = format!("{n} style{}", if n == 1 { "" } else { "s" });
                                card(app, ui, rect, idx, Some(label), ppp, now);
                            } else {
                                list_row(app, ui, rect, idx, ppp, now);
                            }
                        }
                    }
                    Row::Tail => {}
                }
            }
        });
}

pub fn preview_image(app: &mut KernshipApp, ui: &Ui, idx: usize, text_rect: Rect, ppp: f32, color: Color32) {
    preview_image_with(app, ui, idx, None, text_rect, ppp, color)
}

/// Like `preview_image`, with other text than the font's name when no preview text is set
/// (the list view's family rows show the family name).
pub fn preview_image_with(app: &mut KernshipApp, ui: &Ui, idx: usize, name: Option<&str>, text_rect: Rect, ppp: f32, color: Color32) {
    let f = &app.all[idx];
    let text = if f.icon_font && app.st.preview_text.trim().is_empty() {
        ICON_MARK.to_string()
    } else if let (Some(n), true) = (name, app.st.preview_text.trim().is_empty()) {
        n.to_string()
    } else {
        app.preview_text_for(f)
    };
    let px = (app.st.preview_size() * ppp).round() as u32;
    let key = RenderKey {
        font_id: f.id.clone(),
        text,
        px,
        max_w: ((text_rect.width()) * ppp).max(8.0) as u32,
        spacing: app.st.letter_spacing.round() as i32,
    };
    let loc = app.locator(f);
    match app.renderer.get(&key, loc) {
        Some(Some((tex, size))) => {
            let sz = size / ppp;
            let x = match app.st.align {
                Align::Left => text_rect.left(),
                Align::Center => (text_rect.center().x - sz.x / 2.0).max(text_rect.left()),
                Align::Right => (text_rect.right() - sz.x).max(text_rect.left()),
            };
            let r = Rect::from_min_size(pos2(x, text_rect.top()), sz);
            let a = app.renderer.fade_in(&key, ui.ctx());
            ui.painter().with_clip_rect(text_rect).image(
                tex.id(),
                r,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                color.gamma_multiply(a),
            );
        }
        Some(None) => {
            ui.painter().text(
                text_rect.left_center(),
                Align2::LEFT_CENTER,
                if f.is_google() { "Preview unavailable offline" } else { "Preview unavailable" },
                ui_font(12.0),
                pal().faint,
            );
        }
        None => {
            // Placeholder bar while the style is fetched and drawn.
            let h = (app.st.preview_size() * 0.55).min(text_rect.height());
            let r = Rect::from_min_size(text_rect.left_top() + vec2(0.0, app.st.preview_size() * 0.3), vec2(text_rect.width() * 0.6, h));
            ui.painter().with_clip_rect(text_rect).rect_filled(r, 6.0, pal().panel);
        }
    }
}

pub fn status_mark(app: &KernshipApp, idx: usize) -> Mark {
    let f = &app.all[idx];
    if app.downloading.contains(&f.id) {
        Mark::Busy
    } else if !app.is_active(f) {
        Mark::Off
    } else {
        match app.activated_by(&f.id) {
            Some(ActivatedBy::Temporary) => Mark::Temporary,
            Some(ActivatedBy::Auto) => Mark::Auto,
            _ => Mark::Active,
        }
    }
}

pub fn format_icon(p: &egui::Painter, pos: egui::Pos2, f: &FontFace, col: Color32) {
    let otf = f.path.as_ref().and_then(|p| p.extension()).map(|e| e.eq_ignore_ascii_case("otf")).unwrap_or(false);
    if otf {
        p.text(pos, Align2::CENTER_CENTER, "O", ui_font(10.0), col);
    } else {
        p.text(pos + vec2(-2.0, -1.0), Align2::CENTER_CENTER, "T", ui_font(9.5), col);
        p.text(pos + vec2(2.5, 1.0), Align2::CENTER_CENTER, "T", ui_font(7.5), col);
    }
}

pub fn handle_click(app: &mut KernshipApp, ui: &Ui, resp: &egui::Response, idx: usize, dot: Rect, now: f64) {
    let id = app.all[idx].id.clone();
    if resp.clicked() {
        let pos = resp.interact_pointer_pos().unwrap_or_default();
        let m = ui.input(|i| i.modifiers);
        if dot.expand(5.0).contains(pos) {
            app.toggle_active(&id, now);
        } else if m.command {
            if let Some(i) = app.selected.iter().position(|s| s == &id) {
                app.selected.remove(i);
            } else {
                app.selected.push(id.clone());
            }
        } else if m.shift && !app.selected.is_empty() {
            let flat: Vec<String> = app.groups.iter().flat_map(|g| g.faces.iter().map(|&i| app.all[i].id.clone())).collect();
            let a = flat.iter().position(|s| s == &app.selected[0]).unwrap_or(0);
            let b = flat.iter().position(|s| s == &id).unwrap_or(0);
            let (lo, hi) = (a.min(b), a.max(b));
            app.selected = flat[lo..=hi].to_vec();
        } else {
            app.selected = vec![id.clone()];
        }
    }
    if resp.double_clicked() {
        app.toggle_active(&id, now);
    }
    if resp.secondary_clicked() && !app.selected.contains(&id) {
        app.selected = vec![id.clone()];
    }
    resp.context_menu(|ui| font_menu(app, ui, now));
}

/// One font in the grid: a flat panel with its status mark and word, its name, and the
/// specimen. `tail` replaces the status word (a collapsed family shows its number of styles).
fn card(app: &mut KernshipApp, ui: &mut Ui, rect: Rect, idx: usize, tail: Option<String>, ppp: f32, now: f64) {
    let id = app.all[idx].id.clone();
    let resp = ui.interact(rect, ui.id().with(("card", &id)), Sense::click_and_drag());
    let selected = app.selected.contains(&id);
    let c = pal();
    let p = ui.painter().clone();
    let ht = fade(ui, ("card-h", &id), resp.hovered());
    let st = fade(ui, ("card-s", &id), selected);
    p.rect_filled(rect, CARD_RADIUS, mix(mix(c.panel, c.panel_hi, ht), c.ground, st));
    if st > 0.0 {
        // Selection is a heavy ink frame, never a colour.
        p.rect_stroke(rect, CARD_RADIUS, Stroke::new(2.5 * st, c.ink), StrokeKind::Inside);
    }
    focus_ring(ui, &resp, rect, CARD_RADIUS);

    let hy = rect.top() + 16.0;
    let dot_c = pos2(rect.left() + 17.0, hy);
    let mark = status_mark(app, idx);
    let f = &app.all[idx];
    paint_mark(&p, dot_c, 5.0, mark, c.ink, c.dim);
    // Right side of the header: the state in a word (or the family's size), then a star.
    let on = mark != Mark::Off;
    let word = tail.unwrap_or_else(|| mark.word().to_string());
    let wr = p.text(
        pos2(rect.right() - 14.0, hy),
        Align2::RIGHT_CENTER,
        word,
        if on { ui_bold(11.5) } else { ui_font(11.5) },
        if on { c.ink } else { c.dim },
    );
    let mut right = wr.left() - 8.0;
    if app.st.starred.contains(&id) {
        paint_star(&p, pos2(right - 5.0, hy), 5.5, true, c.ink);
        right -= 18.0;
    } else if f.is_google() && f.path.is_none() && resp.hovered() {
        paint_download(&p, pos2(right - 5.0, hy), c.dim);
        right -= 18.0;
    }
    let name_x = rect.left() + 30.0;
    let galley = ui.fonts(|fo| {
        let mut job = egui::text::LayoutJob::simple_singleline(f.display_name(), ui_medium(12.0), c.ink);
        job.wrap = egui::text::TextWrapping::truncate_at_width((right - name_x).max(20.0));
        fo.layout_job(job)
    });
    p.galley(pos2(name_x, hy - galley.size().y / 2.0), galley, c.ink);

    let text_rect = Rect::from_min_max(pos2(rect.left() + 14.0, rect.top() + 30.0), pos2(rect.right() - 14.0, rect.bottom() - 4.0));
    preview_image(app, ui, idx, text_rect, ppp, c.ink);
    if resp.drag_started() {
        if !selected {
            app.selected = vec![id.clone()];
        }
        ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("dragging-fonts"), true));
    }
    // The mark is the switch: its click area is 22 pt across.
    handle_click(app, ui, &resp, idx, Rect::from_center_size(dot_c, vec2(12.0, 12.0)), now);
}

fn list_row(app: &mut KernshipApp, ui: &mut Ui, rect: Rect, idx: usize, ppp: f32, now: f64) {
    let id = app.all[idx].id.clone();
    let resp = ui.interact(rect, ui.id().with(("row", &id)), Sense::click_and_drag());
    let selected = app.selected.contains(&id);
    let p = ui.painter().clone();
    let ht = fade(ui, ("row-h", &id), resp.hovered());
    let st = fade(ui, ("row-s", &id), selected);
    if ht > 0.0 {
        p.rect_filled(rect.shrink2(vec2(0.0, 1.0)), 8.0, pal().panel.gamma_multiply(ht * (1.0 - st)));
    }
    if st > 0.0 {
        p.rect_filled(rect.shrink2(vec2(0.0, 1.0)), 8.0, pal().sel_fill.gamma_multiply(st));
    }
    p.line_segment([pos2(rect.left() + 8.0, rect.bottom()), pos2(rect.right() - 8.0, rect.bottom())], Stroke::new(1.0_f32, pal().rule));
    let cy = rect.center().y;
    let dot_c = pos2(rect.left() + 15.0, cy);
    paint_mark(&p, dot_c, 4.5, status_mark(app, idx), pal().ink, pal().dim);
    let f = &app.all[idx];
    format_icon(&p, pos2(rect.left() + 29.0, cy), f, pal().dim);
    let name_w = 220.0;
    let galley = ui.fonts(|fo| {
        let mut job = egui::text::LayoutJob::simple_singleline(f.style.clone(), ui_font(12.5), pal().ink);
        job.wrap = egui::text::TextWrapping::truncate_at_width(name_w - 50.0);
        fo.layout_job(job)
    });
    p.galley(pos2(rect.left() + 40.0, cy - galley.size().y / 2.0), galley, pal().ink);
    let size = app.st.preview_size();
    let text_rect = Rect::from_min_max(
        pos2(rect.left() + name_w, cy - size * 0.62),
        pos2(rect.right() - 12.0, cy + size * 0.7),
    );
    preview_image(app, ui, idx, text_rect, ppp, pal().ink);
    handle_click(app, ui, &resp, idx, Rect::from_center_size(dot_c, vec2(8.0, 8.0)), now);
}

/// The right-click menu for the selected fonts.
pub fn font_menu(app: &mut KernshipApp, ui: &mut Ui, now: f64) {
    use crate::theme::{menu_item, menu_separator, submenu};
    crate::theme::menu_style(ui);
    ui.set_min_width(crate::theme::MENU_W);
    let ids = app.selected.clone();
    let Some(first) = ids.first().and_then(|i| app.font(i).cloned()) else { return };
    let all_system = ids.iter().all(|i| app.font(i).map(|f| f.source == Source::System).unwrap_or(false));
    let any_active = ids.iter().any(|i| app.font(i).map(|f| app.is_active(f) && f.source != Source::System).unwrap_or(false));
    let all_temp = ids.iter().all(|i| app.activated_by(i) == Some(ActivatedBy::Temporary));
    let all_on = ids.iter().all(|i| matches!(app.activated_by(i), Some(ActivatedBy::Manual) | Some(ActivatedBy::Auto)));

    // Status items with their coloured dots first.
    if menu_item(ui, "Activate", "", !all_system && !all_on, false) {
        app.set_active(&ids, true, now);
        ui.close_menu();
    }
    if menu_item(ui, "Activate Temporarily", "", !all_system && !all_temp, false) {
        app.set_active_by(&ids, true, ActivatedBy::Temporary, now);
        ui.close_menu();
    }
    if menu_item(ui, "Deactivate", "", any_active, false) {
        app.set_active(&ids, false, now);
        ui.close_menu();
    }
    if first.is_google() && first.path.is_none() && menu_item(ui, "Download", "", true, false) {
        for i in &ids {
            if let Some(f) = app.font(i).cloned() {
                app.download(&f, ActivatedBy::Manual);
            }
        }
        ui.close_menu();
    }
    menu_separator(ui);
    let starred = ids.iter().all(|i| app.st.starred.contains(i));
    if menu_item(ui, if starred { "Unstar" } else { "Star" }, "", true, false) {
        app.toggle_star(&ids, now);
        ui.close_menu();
    }
    submenu(ui, "Category", |ui| {
        ui.set_min_width(crate::theme::SUBMENU_W);
        crate::theme::menu_layout(ui, true, true);
        for (i, c) in Category::ALL.iter().enumerate() {
            if menu_item(ui, c.label(), "", true, first.category == *c) {
                app.set_category(&ids, *c, now);
                ui.close_menu();
            }
            if i == 0 {
                menu_separator(ui);
            }
        }
    });
    submenu(ui, "Tags", |ui| {
        ui.set_min_width(crate::theme::SUBMENU_W);
        let tags = app.st.tags.clone();
        let any_on = tags.iter().any(|t| ids.iter().all(|i| t.font_ids.contains(i)));
        crate::theme::menu_layout(ui, any_on, false);
        if tags.is_empty() {
            menu_item(ui, "None", "", false, false);
        }
        for t in &tags {
            let on = ids.iter().all(|i| t.font_ids.contains(i));
            if menu_item(ui, &t.name, "", true, on) {
                app.toggle_tag(&ids, &t.id, now);
            }
        }
    });
    submenu(ui, "Add To Set", |ui| {
        ui.set_min_width(crate::theme::SUBMENU_W + 40.0);
        let libs = app.st.libraries.clone();
        let any_on = libs.iter().any(|l| l.collections.iter().any(|c| c.font_ids.contains(&first.id)));
        crate::theme::menu_layout(ui, any_on, false);
        let mut any = false;
        for l in &libs {
            for c in &l.collections {
                any = true;
                if menu_item(ui, &format!("{} › {}", l.name, c.name), "", true, c.font_ids.contains(&first.id)) {
                    app.add_to_collection(&ids, &l.id, &c.id, now);
                    ui.close_menu();
                }
            }
        }
        if !any {
            menu_item(ui, "None", "", false, false);
        }
        menu_separator(ui);
        for l in &libs {
            if menu_item(ui, &format!("Copy into {}", l.name), "", true, false) {
                app.copy_into_library(&ids, &l.id, now);
                ui.close_menu();
            }
        }
    });
    menu_separator(ui);
    let exportable = ids.iter().any(|i| app.font(i).map(|f| f.path.is_some()).unwrap_or(false));
    if menu_item(ui, "Export…", "", exportable, false) {
        if let Some(dest) = pick_folder() {
            app.export(&ids, dest, now);
        }
        ui.close_menu();
    }
    if menu_item(ui, "Show in Finder", "", first.path.is_some(), false) {
        app.reveal(&first.id);
        ui.close_menu();
    }
    if let Some(dapp) = app.watcher.last_design_app() {
        if menu_item(ui, &format!("Apply in {}", dapp.name), "", true, false) {
            app.apply_to_selection(&first.id, now);
            ui.close_menu();
        }
    }
    let convertible = ids.iter().any(|i| app.font(i).map(|f| f.path.is_some() || f.is_google()).unwrap_or(false));
    if menu_item(ui, "Convert Font for Web", "", convertible, false) {
        app.st.info_visible = true;
        app.st.info_mode = InfoMode::Web;
        app.web_result = None;
        app.save_soon(now);
        ui.close_menu();
    }
    if first.is_google() && menu_item(ui, "Copy Web Embed Code", "", true, false) {
        let variants = app.variants_of(&first.family);
        ui.ctx().copy_text(crate::google::embed_code(&first.family, &variants));
        ui.close_menu();
    }
    menu_separator(ui);
    if let Nav::Collection(l, c) = app.nav.clone() {
        if menu_item(ui, "Remove from Set", "", true, false) {
            if let Some(col) = app.st.libraries.iter_mut().find(|x| x.id == l).and_then(|x| x.collections.iter_mut().find(|y| y.id == c)) {
                col.font_ids.retain(|i| !ids.contains(i));
            }
            app.dirty = true;
            app.save_soon(now);
            ui.close_menu();
        }
    }
    let deletable = !all_system && exportable;
    if menu_item(ui, "Delete from Library", "", deletable, false) {
        app.dialog = Dialog::ConfirmDelete { ids: ids.clone() };
        ui.close_menu();
    }
}

fn preview_bar(app: &mut KernshipApp, ui: &mut Ui, content: Rect, now: f64) {
    let w = 520.0f32.min(content.width() - 24.0);
    let bar = Rect::from_center_size(pos2(content.center().x, content.bottom() - 41.0 - 37.5), vec2(w, 75.0));
    let p = ui.painter().clone();
    p.add(egui::Shape::Rect(egui::epaint::RectShape::filled(bar.translate(vec2(0.0, 8.0)).expand(2.0), 10.0, Color32::from_black_alpha(60)).with_blur_width(18.0)));
    p.rect_filled(bar, 8.0, pal().ground);
    p.rect_stroke(bar, 8.0, Stroke::new(1.0_f32, pal().ink), StrokeKind::Inside);
    // Block clicks from reaching the cards underneath.
    let _ = ui.interact(bar, ui.id().with("pbar-bg"), Sense::click());

    let field = Rect::from_min_size(bar.min + vec2(17.0, 14.0), vec2(w - 34.0, 20.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(field));
    let te = egui::TextEdit::singleline(&mut app.st.preview_text)
        .hint_text(egui::RichText::new("Enter preview text").color(pal().dim))
        .frame(false)
        .font(ui_font(14.0))
        .desired_width(field.width());
    let r = child.add(te);
    if app.focus_preview {
        r.request_focus();
        app.focus_preview = false;
    }
    if r.changed() {
        app.save_soon(now);
    }

    let cy = bar.top() + 52.0;
    let plus = Rect::from_center_size(pos2(bar.left() + 22.0, cy), vec2(20.0, 22.0));
    let pr = ui.interact(plus, ui.id().with("pbar-plus"), Sense::click()).on_hover_text("Save as Sample");
    let pc = plus.center();
    let col = if pr.hovered() { pal().ink } else { pal().dim };
    p.line_segment([pc + vec2(-6.0, 0.0), pc + vec2(6.0, 0.0)], Stroke::new(1.3_f32, col));
    p.line_segment([pc + vec2(0.0, -6.0), pc + vec2(0.0, 6.0)], Stroke::new(1.3_f32, col));
    if pr.clicked() {
        app.dialog = Dialog::NewSample { name: String::new(), text: app.st.preview_text.clone() };
    }

    let chips = Rect::from_min_max(pos2(bar.left() + 38.0, cy - 13.0), pos2(bar.right() - 16.0, cy + 13.0));
    let mut x = chips.left();
    let samples = app.st.samples.clone();
    let clip = p.with_clip_rect(chips);
    for (i, s) in samples.iter().enumerate() {
        let tw = ui.fonts(|f| f.layout_no_wrap(s.name.clone(), ui_font(13.0), pal().ink).size().x);
        let r = Rect::from_min_size(pos2(x, cy - 12.0), vec2(tw + 18.0, 24.0));
        x = r.right() + 6.0;
        if r.left() > chips.right() {
            break;
        }
        let vis = r.intersect(chips);
        let resp = ui.interact(vis, ui.id().with(("chip", i)), Sense::click()).on_hover_text(&s.text);
        let active = app.st.preview_text == s.text;
        let ht = fade(ui, ("chip-h", i), resp.hovered());
        let at = fade(ui, ("chip-a", i), active);
        // The sample in use is an ink chip, like everything that is on.
        let fill = mix(mix(pal().panel, pal().panel_hi, ht), pal().ink, at);
        clip.rect_filled(r, 4.0, fill);
        clip.text(r.center(), Align2::CENTER_CENTER, &s.name, ui_medium(12.5), mix(pal().ink, pal().ground, at));
        focus_ring(ui, &resp, r, 4.0);
        if resp.clicked() {
            if active {
                app.st.preview_text.clear();
            } else {
                app.st.preview_text = s.text.clone();
            }
            app.save_soon(now);
        }
        resp.context_menu(|ui| {
            crate::theme::menu_style(ui);
            if crate::theme::mi(ui, "Delete Sample") {
                app.st.samples.retain(|x| x.name != s.name);
                app.save_soon(now);
                ui.close_menu();
            }
        });
    }
    // Horizontal scrolling of the chip row with the wheel.
    if ui.rect_contains_pointer(chips) {
        let d = ui.input(|i| i.smooth_scroll_delta);
        if d.x.abs() > 0.0 || d.y.abs() > 0.0 {
            let mut list = app.st.samples.clone();
            if d.x + d.y < -8.0 && !list.is_empty() {
                list.rotate_left(1);
                app.st.samples = list;
            } else if d.x + d.y > 8.0 && !list.is_empty() {
                list.rotate_right(1);
                app.st.samples = list;
            }
        }
    }
}

fn toasts(app: &KernshipApp, ui: &Ui, content: Rect, now: f64) {
    let mut y = content.bottom() - 130.0;
    for (msg, until) in app.toasts.iter().rev().take(3) {
        // Fade in and out over 200 ms.
        let left = (*until - now) as f32;
        let a = (left / ANIM).min((3.5 - left) / ANIM).clamp(0.0, 1.0);
        let g = ui.fonts(|f| f.layout_no_wrap(msg.clone(), ui_medium(13.0), pal().ink));
        let r = Rect::from_center_size(pos2(content.center().x, y + (1.0 - a) * 8.0), vec2(g.size().x + 28.0, 30.0));
        ui.painter().rect_filled(r, CTRL_RADIUS, pal().ink.gamma_multiply(a));
        ui.painter().galley_with_override_text_color(r.center() - g.size() / 2.0, g, pal().ground.gamma_multiply(a));
        y -= 38.0;
        if a < 1.0 {
            ui.ctx().request_repaint();
        }
    }
}
