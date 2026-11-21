//! The left source list: the rail. It is the one yellow surface of the interface (yellow
//! lettering on near-black in the dark appearance) and reads like a sign.
use crate::app::{Dialog, RemoveWhat, RenameWhat, KernshipApp};
use crate::model::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Pos2, Rect, Sense, Stroke, Ui};

#[derive(Clone, Copy)]
pub enum Icon {
    AllFonts,
    System,
    Google,
    IconFonts,
    SmartFolder,
    Library,
    Collection,
    Folder,
    Tag,
    Star,
    Clock,
}

/// A pictogram about 13 pt across, drawn in `col`; `bg` is what it sits on (for the shapes
/// that overlap).
pub fn draw_icon(p: &egui::Painter, c: Pos2, icon: Icon, col: Color32, bg: Color32) {
    let s = Stroke::new(1.4_f32, col);
    let mid = egui::StrokeKind::Middle;
    match icon {
        Icon::AllFonts | Icon::Collection => {
            let back = Rect::from_center_size(c + vec2(-1.6, -1.6), vec2(8.5, 8.5));
            let front = Rect::from_center_size(c + vec2(1.6, 1.6), vec2(8.5, 8.5));
            p.rect_stroke(back, 2.0, s, mid);
            p.rect_filled(front, 2.0, bg);
            p.rect_stroke(front, 2.0, s, mid);
        }
        Icon::System => {
            p.rect_stroke(Rect::from_center_size(c, vec2(11.5, 11.5)), 2.5, s, mid);
            p.line_segment([c + vec2(-2.4, 2.8), c + vec2(0.0, -3.0)], s);
            p.line_segment([c + vec2(0.0, -3.0), c + vec2(2.4, 2.8)], s);
            p.line_segment([c + vec2(-1.4, 0.9), c + vec2(1.4, 0.9)], s);
        }
        Icon::Google => {
            // The web: a globe.
            p.circle_stroke(c, 5.6, s);
            p.add(egui::Shape::ellipse_stroke(c, vec2(2.4, 5.6), s));
            p.line_segment([c + vec2(-5.6, 0.0), c + vec2(5.6, 0.0)], s);
        }
        Icon::IconFonts => {
            p.circle_stroke(c + vec2(-3.1, -3.1), 2.2, s);
            p.rect_stroke(Rect::from_center_size(c + vec2(3.1, -3.1), vec2(4.4, 4.4)), 1.0, s, mid);
            p.rect_stroke(Rect::from_center_size(c + vec2(-3.1, 3.1), vec2(4.4, 4.4)), 1.0, s, mid);
            p.circle_stroke(c + vec2(3.1, 3.1), 2.2, s);
        }
        Icon::SmartFolder => {
            // A funnel: three shortening lines.
            p.line_segment([c + vec2(-5.5, -3.8), c + vec2(5.5, -3.8)], s);
            p.line_segment([c + vec2(-3.4, 0.0), c + vec2(3.4, 0.0)], s);
            p.line_segment([c + vec2(-1.3, 3.8), c + vec2(1.3, 3.8)], s);
        }
        Icon::Folder => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.0), vec2(11.5, 8.0));
            p.rect_stroke(r, 1.5, s, mid);
            p.line_segment([r.left_top(), r.left_top() + vec2(1.5, -2.2)], s);
            p.line_segment([r.left_top() + vec2(1.5, -2.2), r.left_top() + vec2(5.0, -2.2)], s);
            p.line_segment([r.left_top() + vec2(5.0, -2.2), r.left_top() + vec2(6.5, 0.0)], s);
        }
        Icon::Library => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.8), vec2(11.5, 7.0));
            p.rect_stroke(r, 1.5, s, mid);
            p.line_segment([r.left_top() + vec2(1.5, -2.6), r.right_top() + vec2(-1.5, -2.6)], s);
            p.line_segment([r.left_top() + vec2(3.2, -5.0), r.right_top() + vec2(-3.2, -5.0)], s);
        }
        Icon::Tag => {
            p.line_segment([c + vec2(-1.2, -5.5), c + vec2(-2.8, 5.5)], s);
            p.line_segment([c + vec2(2.8, -5.5), c + vec2(1.2, 5.5)], s);
            p.line_segment([c + vec2(-5.0, -1.9), c + vec2(5.4, -1.9)], s);
            p.line_segment([c + vec2(-5.4, 1.9), c + vec2(5.0, 1.9)], s);
        }
        Icon::Star => paint_star(p, c + vec2(0.0, 0.4), 6.0, false, col),
        Icon::Clock => {
            p.circle_stroke(c, 5.6, s);
            p.line_segment([c, c + vec2(0.0, -3.4)], s);
            p.line_segment([c, c + vec2(2.6, 1.4)], s);
        }
    }
}

/// Disclosure arrow; `t` 0 = closed (▸), 1 = open (▾), animated in between.
fn chevron(p: &egui::Painter, c: Pos2, t: f32, col: Color32) {
    let s = Stroke::new(1.5_f32, col);
    let a = t * std::f32::consts::FRAC_PI_2;
    let rot = |v: egui::Vec2| vec2(v.x * a.cos() - v.y * a.sin(), v.x * a.sin() + v.y * a.cos());
    let pts = [vec2(-1.8, -3.5), vec2(1.8, 0.0), vec2(-1.8, 3.5)];
    p.line_segment([c + rot(pts[0]), c + rot(pts[1])], s);
    p.line_segment([c + rot(pts[1]), c + rot(pts[2])], s);
}

pub struct RowOut {
    pub clicked: bool,
    pub toggled: bool,
    pub response: egui::Response,
}

/// Counts on the right of the rail sit a step back from the names.
const COUNT_ALPHA: f32 = 0.7;

/// Badge of a set that is not downloaded yet.
const GET: &str = "\u{1}get";

/// A name too long for its row loses its middle, never its end: "Material Icons Outlined"
/// and "Material Icons Round" stay different ("Material…Outlined", "Material…Round").
fn middle_truncate(ui: &Ui, label: &str, width: f32) -> String {
    let measure = |s: &str| ui.fonts(|f| f.layout_no_wrap(s.to_string(), ui_medium(13.5), Color32::WHITE).size().x);
    if measure(label) <= width {
        return label.to_string();
    }
    let chars: Vec<char> = label.chars().collect();
    // First choice: keep whole last words and as much of the start as fits.
    let words: Vec<&str> = label.split(' ').collect();
    for take in (1..words.len().min(3)).rev() {
        let tail = words[words.len() - take..].join(" ");
        let room = chars.len().saturating_sub(tail.chars().count() + 1);
        for head in (3..=room).rev() {
            let s = chars[..head].iter().collect::<String>().trim_end().to_string() + "…" + &tail;
            if measure(&s) <= width {
                return s;
            }
        }
    }
    let mut keep = chars.len().saturating_sub(1);
    while keep > 2 {
        let head = keep.div_ceil(2);
        let tail = keep - head;
        let s: String = chars[..head].iter().collect::<String>().trim_end().to_string() + "…" + chars[chars.len() - tail..].iter().collect::<String>().trim_start();
        if measure(&s) <= width {
            return s;
        }
        keep -= 1;
    }
    label.chars().take(2).collect::<String>() + "…"
}

/// Each nesting level moves the arrow, pictogram and name 14 pt right.
const INDENT: f32 = 14.0;

#[allow(clippy::too_many_arguments)]
fn row(
    ui: &mut Ui,
    panel: Rect,
    y: &mut f32,
    depth: usize,
    icon: Icon,
    label: &str,
    badge: Option<String>,
    selected: bool,
    expandable: Option<bool>,
    id: &str,
) -> RowOut {
    row_count(ui, panel, y, depth, icon, label, badge, None::<&dyn Fn() -> usize>, selected, expandable, id)
}

/// One line of the rail: arrow, pictogram on its square, name, and the number of fonts,
/// always shown. The current source is the inverted band (painted by `draw`, which slides it).
#[allow(clippy::too_many_arguments)]
fn row_count(
    ui: &mut Ui,
    panel: Rect,
    y: &mut f32,
    depth: usize,
    icon: Icon,
    label: &str,
    badge: Option<String>,
    count: Option<&dyn Fn() -> usize>,
    selected: bool,
    expandable: Option<bool>,
    id: &str,
) -> RowOut {
    let c = pal();
    let rect = Rect::from_min_size(pos2(panel.left(), *y), vec2(panel.width(), SIDEBAR_ROW_H));
    *y += SIDEBAR_ROW_H;
    let resp = ui.interact(rect, ui.id().with(("sb", id)), Sense::click());
    let p = ui.painter_at(panel);
    let hover_t = fade(ui, ("sb-h", id), resp.hovered() && !selected);
    let sel_t = fade(ui, ("sb-s", id), selected);
    if hover_t > 0.0 {
        p.rect_filled(rect, 0.0, c.rail_hover.gamma_multiply(hover_t * (1.0 - sel_t)));
    }
    if selected {
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("sb-band-at"), rect.top()));
    }
    // On the band the lettering takes the band's ink.
    let ink = mix(c.rail_ink, c.band_ink, sel_t);
    let ground = mix(c.rail, c.band, sel_t);
    let indent = depth as f32 * INDENT;
    let cy = rect.center().y;
    let mut toggled = false;
    if let Some(open) = expandable {
        let cc = pos2(panel.left() + 11.0 + indent, cy);
        let t = fade(ui, ("sb-open", id), open);
        chevron(&p, cc, t, ink);
        let hit = Rect::from_center_size(cc, vec2(20.0, SIDEBAR_ROW_H));
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if hit.contains(pos) {
                    toggled = true;
                }
            }
        }
    }
    // The pictogram: a square in the lettering's colour, the glyph in the ground's.
    let sq = Rect::from_center_size(pos2(panel.left() + 31.0 + indent, cy), vec2(20.0, 20.0));
    let icon_col = mix(c.rail_icon, c.band_ink, sel_t);
    p.rect_filled(sq, 4.0, icon_col);
    draw_icon(&p, sq.center(), icon, ground, icon_col);
    let text_x = sq.right() + 9.0;
    let mut right = rect.right() - 12.0;
    let number = badge.or_else(|| count.map(|f| crate::ui_main::thousands(f())));
    if let Some(b) = number {
        if b == GET {
            // Not downloaded yet: an action, so it gets its own mark, not a number.
            paint_download(&p, pos2(rect.right() - 18.0, cy), ink);
            right = rect.right() - 32.0;
        } else {
            let r = p.text(pos2(rect.right() - 12.0, cy), Align2::RIGHT_CENTER, b, ui_bold(13.0), ink.gamma_multiply(COUNT_ALPHA));
            right = r.left() - 8.0;
        }
    }
    let shown = middle_truncate(ui, label, (right - text_x).max(20.0));
    let galley = ui.fonts(|f| f.layout_job(egui::text::LayoutJob::simple_singleline(shown, ui_medium(13.5), ink)));
    p.galley(pos2(text_x, cy - galley.size().y / 2.0), galley, ink);
    if resp.has_focus() && ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::new("kbd-nav")).unwrap_or(false)) {
        p.rect_stroke(rect.shrink(2.0), 3.0, Stroke::new(2.0_f32, ink), egui::StrokeKind::Inside);
    }
    RowOut { clicked: resp.clicked() && !toggled, toggled, response: resp }
}

fn header(ui: &mut Ui, panel: Rect, y: &mut f32, text: &str) {
    *y += 12.0;
    ui.painter_at(panel).text(pos2(panel.left() + 14.0, *y + 10.0), Align2::LEFT_CENTER, text, ui_bold(12.0), pal().rail_dim);
    *y += 24.0;
}

fn new_filter() -> Dialog {
    Dialog::SmartFilter {
        filter: SmartFilter {
            id: new_id(),
            name: "Untitled Smart Filter".into(),
            match_all: true,
            rules: vec![Rule { field: RuleField::Name, op: RuleOp::Contains, value: String::new() }],
        },
        is_new: true,
    }
}

/// The "+" menu of the sidebar (and File menu equivalents).
pub fn plus_menu(app: &mut KernshipApp, ui: &mut Ui, now: f64) {
    if crate::theme::mi(ui, "Import Fonts…") {
        crate::ui_main::import_dialog(app, now);
        ui.close_menu();
    }
    if crate::theme::mi(ui, "Add Folder…") {
        if let Some(f) = crate::ui_main::pick_folder() {
            let lib = crate::ui_main::target_library(app, now);
            app.import_paths(lib, vec![f]);
        }
        ui.close_menu();
    }
    ui.separator();
    if crate::theme::mi(ui, "New Library…") {
        app.dialog = Dialog::NewLibrary { name: "Untitled Library".into(), folder: None };
        ui.close_menu();
    }
    if crate::theme::mi(ui, "New Library from Folder…") {
        if let Some(f) = crate::ui_main::pick_folder() {
            let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled Library".into());
            app.dialog = Dialog::NewLibrary { name, folder: Some(f) };
        }
        ui.close_menu();
    }
    if crate::theme::mi(ui, "Open Library…") {
        if let Some(f) = crate::ui_main::pick_library() {
            app.open_library(f, now);
        }
        ui.close_menu();
    }
    ui.separator();
    if crate::theme::mi(ui, "New Tag") {
        app.dialog = Dialog::NewTag { name: String::new() };
        ui.close_menu();
    }
    if crate::theme::mi(ui, "New Smart Filter...") {
        app.dialog = new_filter();
        ui.close_menu();
    }
}

pub fn draw(app: &mut KernshipApp, ui: &mut Ui, panel: Rect, now: f64) {
    let c = pal();
    let painter = ui.painter_at(panel);
    painter.rect_filled(panel, 0.0, c.rail);
    // The dark rail has no colour of its own to part it from the fonts: a hairline does.
    painter.line_segment([panel.right_top(), panel.right_bottom()], Stroke::new(1.0_f32, mix(c.rule, c.rail, light_t())));

    // Title bar strip: sidebar toggle and "+".
    let ty = panel.top() + 26.0;
    let toggle_rect = Rect::from_center_size(pos2(panel.left() + 96.0, ty), vec2(28.0, 26.0));
    let tr = ui.interact(toggle_rect, ui.id().with("sb-toggle"), Sense::click());
    let tp = toggle_rect.center();
    if tr.hovered() {
        painter.rect_filled(toggle_rect, 4.0, c.rail_hover);
    }
    painter.rect_stroke(Rect::from_center_size(tp, vec2(18.0, 14.0)), 3.0, Stroke::new(1.4_f32, c.rail_ink), egui::StrokeKind::Middle);
    painter.line_segment([tp + vec2(-3.5, -7.0), tp + vec2(-3.5, 7.0)], Stroke::new(1.4_f32, c.rail_ink));
    focus_ring(ui, &tr, toggle_rect, 4.0);
    if tr.on_hover_text("Hide Sidebar").clicked() {
        app.st.sidebar_visible = false;
    }
    let plus = Rect::from_center_size(pos2(panel.right() - 22.0, ty), vec2(28.0, 26.0));
    let pr = ui.interact(plus, ui.id().with("sb-plus"), Sense::click()).on_hover_text("Add");
    let pc = plus.center();
    if pr.hovered() {
        painter.rect_filled(plus, 4.0, c.rail_hover);
    }
    painter.line_segment([pc + vec2(-6.5, 0.0), pc + vec2(6.5, 0.0)], Stroke::new(1.5_f32, c.rail_ink));
    painter.line_segment([pc + vec2(0.0, -6.5), pc + vec2(0.0, 6.5)], Stroke::new(1.5_f32, c.rail_ink));
    focus_ring(ui, &pr, plus, 4.0);
    crate::ui_main::click_menu(ui, &pr, "sb-plus-menu", |ui| plus_menu(app, ui, now));

    // Foot of the rail: how many fonts are switched on, by how they got there.
    let top = panel.top() + TOOLBAR_H;
    let foot_h = 104.0;
    let foot = Rect::from_min_max(pos2(panel.left(), panel.bottom() - foot_h), panel.max);
    if panel.height() > 420.0 {
        tally(app, ui, foot);
    }
    let mut list_bottom = if panel.height() > 420.0 { foot.top() - 8.0 } else { panel.bottom() };
    // A newer Kernship is out: one quiet row above the foot, always in view.
    if let Some(v) = app.update_available() {
        let row = Rect::from_min_max(pos2(panel.left() + 8.0, list_bottom - 34.0), pos2(panel.right() - 8.0, list_bottom - 4.0));
        let ready = app.update.ready.as_deref() == Some(v.as_str());
        let downloading = app.update.downloading() == Some(v.as_str());
        if update_row(ui, row, &v, ready, downloading) {
            app.restart_to_update();
        }
        list_bottom = row.top() - 4.0;
    }
    let clip = Rect::from_min_max(pos2(panel.left(), top), pos2(panel.right(), list_bottom));
    let panel = Rect::from_min_max(panel.min, pos2(panel.right(), list_bottom));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(clip));
    child.set_clip_rect(clip);
    let ui = &mut child;
    let sid = egui::Id::new("sb-scroll");
    let content_h = ui.ctx().memory(|m| m.data.get_temp::<f32>(sid.with("h")).unwrap_or(0.0));
    let max_off = (content_h - clip.height() + 12.0).max(0.0);
    let mut off = ui.ctx().memory(|m| m.data.get_temp::<f32>(sid).unwrap_or(0.0)).clamp(0.0, max_off);
    if ui.rect_contains_pointer(clip) {
        let d = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        if d != 0.0 {
            off = (off - d).clamp(0.0, max_off);
        }
    }
    ui.ctx().memory_mut(|m| m.data.insert_temp(sid, off));
    let mut y = top - off;

    // The band of the current source slides from row to row (its place comes from the row
    // drawn last frame, kept in list coordinates so scrolling does not animate it).
    let band_id = egui::Id::new("sb-band-at");
    if let Some(at) = ui.ctx().data_mut(|d| d.remove_temp::<f32>(band_id)) {
        let target = at - top + off;
        let first = ui.ctx().data(|d| d.get_temp::<bool>(band_id.with("seen")).is_none());
        ui.ctx().data_mut(|d| d.insert_temp(band_id.with("seen"), true));
        let by = ease_to(ui.ctx(), band_id.with("y"), target, if first { 0.0 } else { GLIDE_TIME }, GLIDE);
        let band = Rect::from_min_size(pos2(panel.left(), by + top - off), vec2(panel.width(), SIDEBAR_ROW_H));
        ui.painter_at(clip).rect_filled(band, 0.0, c.band);
    }

    let sel = app.nav.clone();
    let mut go: Option<Nav> = None;
    let p = app.st.prefs.clone();

    if p.show_all_fonts && row_count(ui, panel, &mut y, 0, Icon::AllFonts, "All Fonts", None, Some(&|| app.nav_count(&Nav::AllFonts)), sel == Nav::AllFonts, None, "all").clicked {
        go = Some(Nav::AllFonts);
    }

    if p.show_system {
        let open = app.expanded.contains("system");
        let r = row_count(ui, panel, &mut y, 0, Icon::System, "System Fonts", None, Some(&|| app.nav_count(&Nav::SystemFonts)), sel == Nav::SystemFonts, Some(open), "system");
        if r.toggled {
            toggle(app, "system");
        }
        if r.clicked {
            go = Some(Nav::SystemFonts);
        }
        if open {
            for (k, name) in ["System", "All Users", "Current User"].iter().enumerate() {
                let nav = Nav::SystemFolder(k as u8);
                if row(ui, panel, &mut y, 1, Icon::Folder, name, None, sel == nav, None, &format!("sysf{k}")).clicked {
                    go = Some(nav);
                }
            }
        }
    }

    if p.show_starred && row_count(ui, panel, &mut y, 0, Icon::Star, "Starred", None, Some(&|| app.nav_count(&Nav::Starred)), sel == Nav::Starred, None, "starred").clicked {
        go = Some(Nav::Starred);
    }
    if p.show_recents {
        let open = app.expanded.contains("recents");
        let r = row_count(ui, panel, &mut y, 0, Icon::Clock, "Recents", None, Some(&|| app.nav_count(&Nav::Recents)), sel == Nav::Recents, Some(open), "recents");
        if r.toggled {
            toggle(app, "recents");
        }
        if r.clicked {
            go = Some(Nav::Recents);
        }
        if open {
            for (k, name) in ["Today", "Yesterday", "This Week", "This Month"].iter().enumerate() {
                let nav = Nav::RecentsPeriod(k as u8);
                if row(ui, panel, &mut y, 1, Icon::Clock, name, None, sel == nav, None, &format!("rec{k}")).clicked {
                    go = Some(nav);
                }
            }
        }
    }

    if p.show_google {
        let gcount = app.all.iter().filter(|f| f.source == Source::Google).count();
        let badge = if app.loading_catalog && gcount == 0 { Some("…".into()) } else { Some(crate::ui_main::thousands(gcount)) };
        let r = row(ui, panel, &mut y, 0, Icon::Google, "Google Fonts", badge, sel == Nav::GoogleFonts, None, "google");
        if r.clicked {
            go = Some(Nav::GoogleFonts);
        }
        r.response.context_menu(|ui| {
            crate::theme::menu_style(ui);
            if crate::theme::mi(ui, "Sync") {
                app.start_loading(true);
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Show in Finder") {
                let _ = std::process::Command::new("open").arg(crate::store::google_library_dir()).spawn();
                ui.close_menu();
            }
        });
    }

    if p.show_icon_fonts {
        let open = app.expanded.contains("icons");
        let r = row_count(ui, panel, &mut y, 0, Icon::IconFonts, "Icon Fonts", None, Some(&|| app.nav_count(&Nav::IconFonts)), sel == Nav::IconFonts, Some(open), "icons");
        if r.toggled {
            toggle(app, "icons");
        }
        if r.clicked {
            go = Some(Nav::IconFonts);
        }
        if open {
            let sets: Vec<(String, String, bool, usize)> =
                app.icon_sets.iter().map(|s| (s.id.clone(), s.name.clone(), s.remote.is_some(), s.glyphs.len())).collect();
            for (id, name, remote, n) in sets {
                let nav = Nav::IconFamily(id.clone());
                let badge = if remote { Some(GET.to_string()) } else { Some(crate::ui_main::thousands(n)) };
                if row(ui, panel, &mut y, 1, Icon::IconFonts, &name, badge, sel == nav, None, &format!("ic-{id}")).clicked {
                    go = Some(nav);
                }
            }
        }
    }

    if p.show_smart_filters {
        let open = app.expanded.contains("smart");
        let r = row(ui, panel, &mut y, 0, Icon::SmartFolder, "Smart Filters", None, false, Some(open), "smart");
        if r.toggled || r.clicked {
            toggle(app, "smart");
        }
        r.response.context_menu(|ui| {
            crate::theme::menu_style(ui);
            if crate::theme::mi(ui, "New Smart Filter...") {
                app.dialog = new_filter();
                ui.close_menu();
            }
        });
        if open {
            for (k, name) in BUILTIN_SMART.iter().enumerate() {
                let nav = Nav::BuiltinSmart(k as u8);
                if row(ui, panel, &mut y, 1, Icon::SmartFolder, name, None, sel == nav, None, &format!("bs{k}")).clicked {
                    go = Some(nav);
                }
            }
            let filters = app.st.smart_filters.clone();
            for f in filters {
                let r = row(ui, panel, &mut y, 1, Icon::SmartFolder, &f.name, None, sel == Nav::SmartFilter(f.id.clone()), None, &f.id);
                if r.clicked {
                    go = Some(Nav::SmartFilter(f.id.clone()));
                }
                r.response.context_menu(|ui| {
            crate::theme::menu_style(ui);
                    if crate::theme::mi(ui, "Edit Smart Filter") {
                        app.dialog = Dialog::SmartFilter { filter: f.clone(), is_new: false };
                        ui.close_menu();
                    }
                    if crate::theme::mi(ui, "Delete") {
                        app.dialog = Dialog::ConfirmRemove { what: RemoveWhat::SmartFilter(f.id.clone()), name: f.name.clone() };
                        ui.close_menu();
                    }
                });
            }
        }
    }

    header(ui, panel, &mut y, "Libraries");
    let libs = app.st.libraries.clone();
    for lib in &libs {
        let has_sets = !lib.collections.is_empty();
        let expandable = if has_sets { Some(app.expanded.contains(&lib.id)) } else { None };
        let r = row_count(ui, panel, &mut y, 0, Icon::Library, &lib.name, None, Some(&|| app.nav_count(&Nav::Library(lib.id.clone()))), sel == Nav::Library(lib.id.clone()), expandable, &lib.id);
        if r.toggled {
            toggle(app, &lib.id);
        }
        if r.clicked {
            go = Some(Nav::Library(lib.id.clone()));
        }
        drop_target(app, &r.response, &lib.id, None, now);
        let lib_ids = || -> Vec<String> { Vec::new() };
        let _ = lib_ids;
        r.response.context_menu(|ui| {
            crate::theme::menu_style(ui);
            let ids: Vec<String> = app.all.iter().filter(|f| f.source == Source::Library(lib.id.clone())).map(|f| f.id.clone()).collect();
            if crate::theme::mi(ui, "Activate") {
                app.set_active(&ids, true, now);
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Deactivate") {
                app.set_active(&ids, false, now);
                ui.close_menu();
            }
            ui.separator();
            if crate::theme::mi(ui, "Import Fonts…") {
                if let Some(paths) = crate::ui_main::pick_paths() {
                    app.import_paths(lib.id.clone(), paths);
                }
                ui.close_menu();
            }
            if crate::theme::mi(ui, "New Set…") {
                app.dialog = Dialog::NewCollection { library: lib.id.clone(), parent: None, name: "Untitled Set".into() };
                ui.close_menu();
            }
            ui.separator();
            if crate::theme::mi(ui, "Rename") {
                app.dialog = Dialog::Rename { what: RenameWhat::Library(lib.id.clone()), name: lib.name.clone() };
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Show in Finder") {
                let _ = std::process::Command::new("open").arg(&lib.location).spawn();
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Delete") {
                app.dialog = Dialog::ConfirmRemove { what: RemoveWhat::Library(lib.id.clone()), name: lib.name.clone() };
                ui.close_menu();
            }
        });
        if app.expanded.contains(&lib.id) {
            draw_collections(app, ui, panel, &mut y, lib, None, 1, &sel, &mut go, now);
        }
    }

    header(ui, panel, &mut y, "Tags");
    let tags = app.st.tags.clone();
    for t in &tags {
        let r = row_count(ui, panel, &mut y, 0, Icon::Tag, &t.name, None, Some(&|| app.nav_count(&Nav::Tag(t.id.clone()))), sel == Nav::Tag(t.id.clone()), None, &t.id);
        if r.clicked {
            go = Some(Nav::Tag(t.id.clone()));
        }
        if r.response.hovered() && !app.selected.is_empty() && dropped_here(&r.response) {
            let ids = app.selected.clone();
            app.toggle_tag(&ids, &t.id, now);
        }
        r.response.context_menu(|ui| {
            crate::theme::menu_style(ui);
            if crate::theme::mi(ui, "Rename") {
                app.dialog = Dialog::Rename { what: RenameWhat::Tag(t.id.clone()), name: t.name.clone() };
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Delete") {
                app.dialog = Dialog::ConfirmRemove { what: RemoveWhat::Tag(t.id.clone()), name: t.name.clone() };
                ui.close_menu();
            }
        });
    }

    let h = y + off - top;
    ui.ctx().memory_mut(|m| m.data.insert_temp(sid.with("h"), h));
    if let Some(n) = go {
        app.set_nav(n);
    }
}

fn dropped_here(resp: &egui::Response) -> bool {
    resp.ctx.input(|i| i.pointer.any_released()) && resp.ctx.memory(|m| m.data.get_temp::<bool>(egui::Id::new("dragging-fonts")).unwrap_or(false))
}

#[allow(clippy::too_many_arguments)]
fn draw_collections(
    app: &mut KernshipApp,
    ui: &mut Ui,
    panel: Rect,
    y: &mut f32,
    lib: &Library,
    parent: Option<&str>,
    depth: usize,
    sel: &Nav,
    go: &mut Option<Nav>,
    now: f64,
) {
    let cols: Vec<Collection> = lib.collections.iter().filter(|c| c.parent.as_deref() == parent).cloned().collect();
    for c in cols {
        let has_children = lib.collections.iter().any(|x| x.parent.as_deref() == Some(c.id.as_str()));
        let key = format!("{}/{}", lib.id, c.id);
        let expandable = if has_children { Some(app.expanded.contains(&key)) } else { None };
        let nav = Nav::Collection(lib.id.clone(), c.id.clone());
        let r = row(ui, panel, y, depth, Icon::Collection, &c.name, None, *sel == nav, expandable, &key);
        if r.toggled {
            toggle(app, &key);
        }
        if r.clicked {
            *go = Some(nav);
        }
        drop_target(app, &r.response, &lib.id, Some(&c.id), now);
        r.response.context_menu(|ui| {
            crate::theme::menu_style(ui);
            if crate::theme::mi(ui, "Activate") {
                app.set_active(&c.font_ids, true, now);
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Deactivate") {
                app.set_active(&c.font_ids, false, now);
                ui.close_menu();
            }
            ui.separator();
            if crate::theme::mi(ui, "New Set…") {
                app.dialog = Dialog::NewCollection { library: lib.id.clone(), parent: Some(c.id.clone()), name: "Untitled Set".into() };
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Export…") {
                if let Some(dest) = crate::ui_main::pick_folder() {
                    app.export(&c.font_ids, dest, now);
                }
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Rename") {
                app.dialog = Dialog::Rename { what: RenameWhat::Collection(lib.id.clone(), c.id.clone()), name: c.name.clone() };
                ui.close_menu();
            }
            if crate::theme::mi(ui, "Delete") {
                app.dialog = Dialog::ConfirmRemove { what: RemoveWhat::Collection(lib.id.clone(), c.id.clone()), name: c.name.clone() };
                ui.close_menu();
            }
        });
        if has_children && app.expanded.contains(&key) {
            draw_collections(app, ui, panel, y, lib, Some(&c.id), depth + 1, sel, go, now);
        }
    }
}

fn drop_target(app: &mut KernshipApp, resp: &egui::Response, lib: &str, set: Option<&str>, now: f64) {
    if resp.hovered() && !app.selected.is_empty() && dropped_here(resp) {
        let ids = app.selected.clone();
        match set {
            Some(c) => app.add_to_collection(&ids, lib, c, now),
            None => app.copy_into_library(&ids, lib, now),
        }
    }
}

fn toggle(app: &mut KernshipApp, key: &str) {
    if !app.expanded.remove(key) {
        app.expanded.insert(key.to_string());
    }
}

/// The count of switched-on fonts at the foot of the rail, each with its status mark.
fn tally(app: &KernshipApp, ui: &Ui, foot: Rect) {
    let c = pal();
    let p = ui.painter_at(foot);
    p.rect_filled(foot, 0.0, c.rail);
    p.line_segment([pos2(foot.left() + 14.0, foot.top()), pos2(foot.right() - 12.0, foot.top())], Stroke::new(1.0_f32, c.rail_ink));
    p.text(pos2(foot.left() + 14.0, foot.top() + 18.0), Align2::LEFT_CENTER, "Activation", ui_bold(12.0), c.rail_dim);
    let mut n = [0usize; 3];
    for e in app.st.active.values() {
        match e.by {
            ActivatedBy::Temporary => n[1] += 1,
            ActivatedBy::Auto => n[2] += 1,
            _ => n[0] += 1,
        }
    }
    let rows = [(Mark::Active, "Active", n[0]), (Mark::Temporary, "Temporary", n[1]), (Mark::Auto, "Auto", n[2])];
    for (i, (mark, word, count)) in rows.iter().enumerate() {
        let cy = foot.top() + 42.0 + i as f32 * 22.0;
        paint_mark(&p, pos2(foot.left() + 20.0, cy), 5.0, *mark, c.rail_icon, c.rail_icon);
        p.text(pos2(foot.left() + 34.0, cy), Align2::LEFT_CENTER, *word, ui_medium(12.5), c.rail_ink);
        p.text(pos2(foot.right() - 12.0, cy), Align2::RIGHT_CENTER, crate::ui_main::thousands(*count), ui_bold(12.5), c.rail_ink.gamma_multiply(COUNT_ALPHA));
    }
}

/// A newer Kernship, in one of three states: downloading (nothing to click), ready (click:
/// restart into it), or just out (click: the download page). True when a restart was asked for.
fn update_row(ui: &mut Ui, row: Rect, version: &str, ready: bool, downloading: bool) -> bool {
    let c = pal();
    let sense = if downloading { Sense::hover() } else { Sense::click() };
    let resp = ui.interact(row, ui.id().with("sb-update"), sense);
    let hot = !downloading && (resp.hovered() || resp.has_focus());
    let p = ui.painter_at(row.expand(3.0));
    p.rect_filled(row, 4.0, c.rail_icon.gamma_multiply(if hot { 0.24 } else { 0.14 }));
    let icon = pos2(row.left() + 16.0, row.center().y);
    if ready {
        paint_restart(&p, icon, 5.5, c.rail_icon);
    } else {
        paint_download(&p, icon + egui::vec2(0.0, 1.5), c.rail_icon);
    }
    let (label, short, tip) = if ready {
        (format!("Restart to update to {version}"), "Restart to update", "Kernship also updates on its own the next time you quit it. Your fonts and libraries stay where they are.")
    } else if downloading {
        (format!("Downloading {version}…"), "Downloading…", "It installs when you quit Kernship, or at once with Restart.")
    } else {
        (format!("Kernship {version} is out"), "Update available", "Opens the download page. Your fonts and libraries stay where they are.")
    };
    // the full words when they fit, the short ones when not, and never past the row
    let room = row.width() - 32.0 - 8.0;
    let fits = |t: &str| ui.fonts(|f| f.layout_no_wrap(t.to_string(), ui_medium(13.0), c.rail_ink).size().x) <= room;
    let mut text = if fits(&label) { label.clone() } else { short.to_string() };
    while !fits(&text) && text.chars().count() > 2 {
        let keep = text.chars().count() - 2;
        text = format!("{}…", text.chars().take(keep).collect::<String>().trim_end_matches('…'));
    }
    let ink = if downloading { c.rail_ink.gamma_multiply(0.75) } else { c.rail_ink };
    let g = ui.fonts(|f| f.layout_no_wrap(text, ui_medium(13.0), ink));
    p.galley(pos2(row.left() + 32.0, row.center().y - g.size().y / 2.0), g, ink);
    focus_ring(ui, &resp, row, 4.0);
    let kind = if ready { egui::WidgetType::Button } else { egui::WidgetType::Link };
    resp.widget_info(|| egui::WidgetInfo::labeled(kind, !downloading, label.clone()));
    if resp.on_hover_text(tip).clicked() {
        if ready {
            return true;
        }
        ui.ctx().open_url(egui::OpenUrl::new_tab(crate::update::DOWNLOAD_PAGE));
    }
    false
}
