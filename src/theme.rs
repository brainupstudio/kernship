//! Kernship's look: the palette for both appearances, the interface typeface, and the shared
//! controls (menus, checkboxes, buttons, status marks, focus rings).
use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke};
use std::sync::atomic::{AtomicU32, Ordering};

/// Every colour of the interface, by job. Yellow belongs to the rail (the sidebar) and to
/// nothing else; selection, primary actions and status are ink on ground.
#[derive(Clone, Copy)]
pub struct Pal {
    /// The window behind the fonts.
    pub ground: Color32,
    /// Cards, controls and the right pane.
    pub panel: Color32,
    /// A panel under the pointer, quiet chips.
    pub panel_hi: Color32,
    /// Selected rows of a list or table.
    pub sel_fill: Color32,
    /// Text fields and code.
    pub field: Color32,
    /// Menus and popovers.
    pub menu: Color32,
    /// The sidebar: the sign.
    pub rail: Color32,
    pub rail_ink: Color32,
    pub rail_dim: Color32,
    /// Pictograms and status marks in the rail: the sign colour.
    pub rail_icon: Color32,
    pub rail_hover: Color32,
    /// The current source in the rail: the inverted band.
    pub band: Color32,
    pub band_ink: Color32,
    pub ink: Color32,
    pub dim: Color32,
    pub faint: Color32,
    pub rule: Color32,
    pub rule_strong: Color32,
    pub danger: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub const DARK: Pal = Pal {
    ground: rgb(16, 16, 16),
    panel: rgb(28, 28, 27),
    panel_hi: rgb(40, 40, 38),
    sel_fill: rgb(58, 58, 54),
    field: rgb(22, 22, 21),
    menu: rgb(32, 32, 31),
    rail: rgb(25, 25, 24),
    rail_ink: rgb(242, 241, 236),
    rail_dim: rgb(166, 165, 158),
    rail_icon: rgb(255, 210, 0),
    rail_hover: rgb(40, 40, 38),
    band: rgb(255, 210, 0),
    band_ink: rgb(17, 17, 17),
    ink: rgb(242, 241, 236),
    dim: rgb(166, 165, 158),
    faint: rgb(140, 139, 132),
    rule: rgb(52, 52, 47),
    rule_strong: rgb(92, 92, 85),
    danger: rgb(255, 122, 107),
};

pub const LIGHT: Pal = Pal {
    ground: rgb(255, 255, 255),
    panel: rgb(243, 243, 240),
    panel_hi: rgb(232, 232, 228),
    sel_fill: rgb(214, 214, 208),
    field: rgb(255, 255, 255),
    menu: rgb(255, 255, 255),
    rail: rgb(255, 210, 0),
    rail_ink: rgb(17, 17, 17),
    rail_dim: rgb(61, 51, 0),
    rail_icon: rgb(17, 17, 17),
    rail_hover: rgb(240, 194, 0),
    band: rgb(17, 17, 17),
    band_ink: rgb(255, 210, 0),
    ink: rgb(17, 17, 17),
    dim: rgb(85, 85, 79),
    faint: rgb(108, 108, 101),
    rule: rgb(216, 216, 211),
    rule_strong: rgb(150, 150, 143),
    danger: rgb(190, 40, 28),
};

/// The palette of the current appearance (blended while the appearance changes).
pub fn pal() -> Pal {
    let t = light_t();
    if t <= 0.0 {
        return DARK;
    }
    if t >= 1.0 {
        return LIGHT;
    }
    let m = |a: Color32, b: Color32| mix(a, b, t);
    let (d, l) = (DARK, LIGHT);
    Pal {
        ground: m(d.ground, l.ground),
        panel: m(d.panel, l.panel),
        panel_hi: m(d.panel_hi, l.panel_hi),
        sel_fill: m(d.sel_fill, l.sel_fill),
        field: m(d.field, l.field),
        menu: m(d.menu, l.menu),
        rail: m(d.rail, l.rail),
        rail_ink: m(d.rail_ink, l.rail_ink),
        rail_dim: m(d.rail_dim, l.rail_dim),
        rail_icon: m(d.rail_icon, l.rail_icon),
        rail_hover: m(d.rail_hover, l.rail_hover),
        band: m(d.band, l.band),
        band_ink: m(d.band_ink, l.band_ink),
        ink: m(d.ink, l.ink),
        dim: m(d.dim, l.dim),
        faint: m(d.faint, l.faint),
        rule: m(d.rule, l.rule),
        rule_strong: m(d.rule_strong, l.rule_strong),
        danger: m(d.danger, l.danger),
    }
}

/// The rail and the right pane run to the window edges.
pub const SIDEBAR_INSET: f32 = 0.0;
pub const SIDEBAR_ROW_H: f32 = 32.0;
pub const TOOLBAR_H: f32 = 52.0;
/// Horizontal and vertical spacing 12.
pub const CARD_GAP: f32 = 12.0;
pub const CARD_RADIUS: f32 = 6.0;
/// Corner radius of buttons, fields and chips.
pub const CTRL_RADIUS: f32 = 5.0;

pub fn hairline(c: Color32) -> Stroke {
    Stroke::new(1.0_f32, c)
}

pub fn ui_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn ui_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}

pub fn ui_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

/// Familjen Grotesk (SIL Open Font License, see assets/fonts), the interface typeface.
const FACE_REGULAR: &[u8] = include_bytes!("../assets/fonts/FamiljenGrotesk-Regular.ttf");
const FACE_MEDIUM: &[u8] = include_bytes!("../assets/fonts/FamiljenGrotesk-Medium.ttf");
const FACE_BOLD: &[u8] = include_bytes!("../assets/fonts/FamiljenGrotesk-Bold.ttf");

/// The interface is set in one family, in three weights. The system font stands behind it for
/// the letters Familjen Grotesk does not have (font and tag names in other scripts).
pub fn install_fonts(ctx: &egui::Context) {
    let mut defs = FontDefinitions::default();
    let mut behind: Vec<String> = Vec::new();
    for path in ["/System/Library/Fonts/SFNS.ttf", "/System/Library/Fonts/Helvetica.ttc"] {
        if let Ok(bytes) = std::fs::read(path) {
            defs.font_data.insert("sys-ui".into(), FontData::from_owned(bytes).into());
            behind.push("sys-ui".into());
            break;
        }
    }
    behind.extend(defs.families[&FontFamily::Proportional].iter().cloned());
    for (name, bytes) in [("ui-regular", FACE_REGULAR), ("ui-medium", FACE_MEDIUM), ("ui-bold", FACE_BOLD)] {
        let fd = FontData::from_static(bytes);
        defs.font_data.insert(name.into(), fd.into());
    }
    let with = |first: &str| {
        let mut fam = vec![first.to_string()];
        fam.extend(behind.iter().cloned());
        fam
    };
    defs.families.insert(FontFamily::Proportional, with("ui-regular"));
    defs.families.insert(FontFamily::Name("medium".into()), with("ui-medium"));
    defs.families.insert(FontFamily::Name("bold".into()), with("ui-bold"));
    // Fonts for the letters of the Category menu (each kind of type), with the UI font behind them.
    let extra = [
        ("icon-serif", &["/System/Library/Fonts/NewYork.ttf", "/System/Library/Fonts/Supplemental/Times New Roman.ttf"][..]),
        ("icon-slab", &["/System/Library/Fonts/Supplemental/Rockwell.ttc", "/System/Library/Fonts/Supplemental/Courier New Bold.ttf"][..]),
        ("icon-script", &["/System/Library/Fonts/Supplemental/SnellRoundhand.ttc", "/System/Library/Fonts/Supplemental/Apple Chancery.ttf"][..]),
        ("icon-fraktur", &["/System/Library/Fonts/Supplemental/STIXTwoMath.otf", "/System/Library/Fonts/Apple Symbols.ttf"][..]),
        ("icon-mono", &["/System/Library/Fonts/Menlo.ttc", "/System/Library/Fonts/Monaco.ttf"][..]),
        ("icon-dingbat", &["/System/Library/Fonts/ZapfDingbats.ttf", "/System/Library/Fonts/Apple Symbols.ttf"][..]),
        ("proportional-icon", &["/System/Library/Fonts/SFNS.ttf"][..]),
        // Search panel › Properties › Figures: old-style and lining numbers.
        ("icon-oldstyle", &["/System/Library/Fonts/Supplemental/Georgia.ttf"][..]),
        ("icon-lining", &["/System/Library/Fonts/Supplemental/Times New Roman.ttf", "/System/Library/Fonts/NewYork.ttf"][..]),
    ];
    let base = defs.families[&FontFamily::Proportional].clone();
    for (name, paths) in extra {
        let mut fam = Vec::new();
        for path in paths {
            if let Ok(bytes) = std::fs::read(path) {
                defs.font_data.insert(name.to_string(), FontData::from_owned(bytes).into());
                fam.push(name.to_string());
                break;
            }
        }
        fam.extend(base.iter().cloned());
        defs.families.insert(FontFamily::Name(name.into()), fam);
    }
    ctx.set_fonts(defs);
    apply_style(ctx);
}

// ----- appearance -----

static LIGHT_T: AtomicU32 = AtomicU32::new(0);

/// 0 = dark, 1 = light; animated between them over 200 ms when the appearance changes.
pub fn light_t() -> f32 {
    f32::from_bits(LIGHT_T.load(Ordering::Relaxed))
}

pub fn set_light_t(t: f32) {
    LIGHT_T.store(t.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

/// Hover and selection fade (200 ms), as every state change in the app.
pub fn fade(ui: &egui::Ui, id: impl std::hash::Hash, on: bool) -> f32 {
    ui.ctx().animate_bool_with_time(ui.id().with(("fade", id)), on, ANIM)
}

pub const ANIM: f32 = 0.2;

/// egui's own widgets (the few places that still use them) in the same vocabulary: flat
/// panels, ink frames, an ink fill for what is on.
pub fn apply_style(ctx: &egui::Context) {
    let t = light_t();
    let c = pal();
    let mut style = (*ctx.style()).clone();
    style.visuals = if t > 0.5 { egui::Visuals::light() } else { egui::Visuals::dark() };
    style.animation_time = ANIM;
    let v = &mut style.visuals;
    v.panel_fill = c.ground;
    v.window_fill = c.menu;
    v.window_stroke = Stroke::new(1.0_f32, c.rule_strong);
    v.popup_shadow = egui::Shadow { offset: [0, 6], blur: 18, spread: 0, color: Color32::from_black_alpha(if t > 0.5 { 40 } else { 110 }) };
    v.window_shadow = egui::Shadow { offset: [0, 10], blur: 30, spread: 0, color: Color32::from_black_alpha(if t > 0.5 { 45 } else { 120 }) };
    let r = egui::CornerRadius::same(CTRL_RADIUS as u8);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = r;
        w.fg_stroke = Stroke::new(1.0_f32, c.ink);
        w.expansion = 0.0;
    }
    v.widgets.inactive.bg_fill = c.panel_hi;
    v.widgets.inactive.weak_bg_fill = c.panel_hi;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, c.rule_strong);
    v.widgets.hovered.bg_fill = c.sel_fill;
    v.widgets.hovered.weak_bg_fill = c.sel_fill;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, c.ink);
    v.widgets.active.bg_fill = c.ink;
    v.widgets.active.weak_bg_fill = c.ink;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, c.ink);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, c.ground);
    v.widgets.open.bg_fill = c.sel_fill;
    v.widgets.open.weak_bg_fill = c.sel_fill;
    v.widgets.open.bg_stroke = Stroke::new(1.0_f32, c.ink);
    v.widgets.noninteractive.bg_stroke = hairline(c.rule);
    v.extreme_bg_color = c.field;
    v.override_text_color = Some(c.ink);
    // Selected text in a field keeps its ink on a grey fill.
    v.selection.bg_fill = c.sel_fill;
    v.selection.stroke = Stroke::new(1.0_f32, c.ink);
    v.text_cursor.stroke = Stroke::new(1.5_f32, c.ink);
    v.hyperlink_color = c.ink;
    v.window_corner_radius = egui::CornerRadius::same(8);
    v.menu_corner_radius = egui::CornerRadius::same(8);
    style.spacing.menu_margin = egui::Margin::same(5);
    style.spacing.combo_height = 320.0;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.text_styles.insert(egui::TextStyle::Body, ui_font(13.0));
    style.text_styles.insert(egui::TextStyle::Button, ui_medium(13.0));
    style.text_styles.insert(egui::TextStyle::Small, ui_font(11.5));
    style.text_styles.insert(egui::TextStyle::Heading, ui_bold(16.0));
    ctx.set_style(style);
}

/// Menus, submenus and dropdown lists drawn like macOS menus: rows 22 pt tall, no frames,
/// an ink band under the pointer with ground lettering.
pub fn menu_style(ui: &mut egui::Ui) {
    let v = ui.visuals_mut();
    for w in [&mut v.widgets.inactive, &mut v.widgets.noninteractive] {
        w.weak_bg_fill = Color32::TRANSPARENT;
        w.bg_fill = Color32::TRANSPARENT;
        w.bg_stroke = Stroke::NONE;
        w.fg_stroke = Stroke::new(1.0_f32, pal().ink);
        w.corner_radius = egui::CornerRadius::same(5);
    }
    for w in [&mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.weak_bg_fill = menu_highlight();
        w.bg_fill = menu_highlight();
        w.bg_stroke = Stroke::NONE;
        w.fg_stroke = Stroke::new(1.0_f32, pal().ground);
        w.corner_radius = egui::CornerRadius::same(6);
        w.expansion = 0.0;
    }
    v.override_text_color = None;
    v.selection.bg_fill = menu_highlight();
    v.selection.stroke = Stroke::new(1.0_f32, pal().ground);
    let s = ui.spacing_mut();
    // Labels of submenu rows start where menu_item labels start (after the tick column).
    s.button_padding = egui::vec2(12.0, 3.0);
    s.item_spacing = egui::vec2(0.0, 0.0);
    s.interact_size.y = MENU_ROW_H;
    ui.style_mut().override_font_id = Some(ui_font(MENU_FONT));
    // Menus are about 236 pt wide.
    ui.set_max_width(MENU_W);
}

pub const MENU_W: f32 = 236.0;
pub const SUBMENU_W: f32 = 180.0;
/// Measured on the system View menu (picture of the menu bar): tick at 12 pt, icon centred at
/// 30.5 pt, label at 45 pt, rows 24 pt, 13 pt text, shortcuts 16 pt from the right edge.
pub const MENU_TEXT_X: f32 = 45.0;
pub const MENU_ROW_H: f32 = 24.0;
pub const MENU_FONT: f32 = 13.0;
const MENU_RIGHT: f32 = 16.0;

/// The row under the pointer in a menu: an ink band with ground lettering.
pub fn menu_highlight() -> Color32 {
    pal().ink
}

fn menu_text() -> Color32 {
    pal().ink
}

/// macOS pop-up button arrows (⌃⌄) for dropdowns.
pub fn popup_icon(ui: &egui::Ui, rect: egui::Rect, _v: &egui::style::WidgetVisuals, _open: bool, _dir: egui::AboveOrBelow) {
    let c = rect.center();
    let s = Stroke::new(1.4_f32, pal().dim);
    let p = ui.painter();
    p.line_segment([c + egui::vec2(-3.0, -1.5), c + egui::vec2(0.0, -4.5)], s);
    p.line_segment([c + egui::vec2(0.0, -4.5), c + egui::vec2(3.0, -1.5)], s);
    p.line_segment([c + egui::vec2(-3.0, 1.5), c + egui::vec2(0.0, 4.5)], s);
    p.line_segment([c + egui::vec2(0.0, 4.5), c + egui::vec2(3.0, 1.5)], s);
}

/// One menu row: a tick column, the item's icon, its label and an optional shortcut, all on the
/// same lines in every menu. Disabled rows are grey.
/// Which columns a menu uses: the tick column only when some item can be ticked, the icon
/// column only when its items have icons (lists of sets or tags don't), so labels never sit
/// after an empty gap.
pub fn menu_layout(ui: &egui::Ui, ticks: bool, icons: bool) {
    ui.data_mut(|d| d.insert_temp(ui.id().with("menu-layout"), (ticks, icons)));
}

/// (tick x, icon x, label x, draw icons)
fn columns(ui: &egui::Ui) -> (f32, f32, f32, bool) {
    let (ticks, icons) = ui.data(|d| d.get_temp::<(bool, bool)>(ui.id().with("menu-layout"))).unwrap_or((false, true));
    match (ticks, icons) {
        (true, true) => (MENU_CHECK_X, MENU_ICON_X, MENU_TEXT_X, true),
        (false, true) => (0.0, 19.0, 34.0, true),
        (true, false) => (MENU_CHECK_X, 0.0, 26.0, false),
        (false, false) => (0.0, 0.0, 12.0, false),
    }
}

pub fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: &str, enabled: bool, checked: bool) -> bool {
    let (check_x, icon_x, text_x, icons) = columns(ui);
    let w = ui.available_width();
    let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, MENU_ROW_H), sense);
    let hovered = enabled && resp.hovered();
    let p = ui.painter();
    let ht = fade(ui, ("menu", label), hovered);
    if ht > 0.0 {
        p.rect_filled(rect, 6.0, menu_highlight().gamma_multiply(ht));
    }
    let col = if !enabled { pal().faint } else { mix(menu_text(), pal().ground, ht) };
    if checked && check_x > 0.0 {
        let c = egui::pos2(rect.left() + check_x, rect.center().y);
        let s = Stroke::new(1.6_f32, col);
        p.line_segment([c + egui::vec2(-4.0, 0.5), c + egui::vec2(-1.2, 3.3)], s);
        p.line_segment([c + egui::vec2(-1.2, 3.3), c + egui::vec2(4.2, -3.4)], s);
    }
    if icons {
        paint_menu_icon(ui, egui::pos2(rect.left() + icon_x, rect.center().y), label, col);
    }
    p.text(egui::pos2(rect.left() + text_x, rect.center().y), egui::Align2::LEFT_CENTER, label, ui_font(MENU_FONT), col);
    if !shortcut.is_empty() {
        let sc = if !enabled { pal().rule_strong } else { mix(pal().dim, pal().ground, ht) };
        p.text(egui::pos2(rect.right() - MENU_RIGHT, rect.center().y), egui::Align2::RIGHT_CENTER, shortcut, ui_font(MENU_FONT), sc);
    }
    resp.clicked() && enabled
}

/// A row that opens a submenu: same tick/icon/label lines as the other rows, a chevron on the right.
pub fn submenu<R>(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui) -> R) {
    let (_, icon_x, text_x, icons) = columns(ui);
    // egui draws the row and opens the submenu; its own label is left empty and its arrow is
    // covered, so the label, icon and chevron match `menu_item`.
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(8.0, 3.0);
            ui.menu_button(" ", |ui| {
                menu_style(ui);
                add(ui)
            })
        })
        .inner;
    let rect = r.response.rect;
    let active = r.response.hovered() || r.inner.is_some();
    let p = ui.painter();
    let bg = if active { menu_highlight() } else { pal().menu };
    p.rect_filled(egui::Rect::from_min_max(egui::pos2(rect.right() - 34.0, rect.top() + 1.0), egui::pos2(rect.right() - 1.0, rect.bottom() - 1.0)), 5.0, bg);
    let col = if active { pal().ground } else { menu_text() };
    if icons {
        paint_menu_icon(ui, egui::pos2(rect.left() + icon_x, rect.center().y), label, col);
    }
    p.text(egui::pos2(rect.left() + text_x, rect.center().y), egui::Align2::LEFT_CENTER, label, ui_font(MENU_FONT), col);
    let c = egui::pos2(rect.right() - MENU_RIGHT - 2.0, rect.center().y);
    let st = Stroke::new(1.5_f32, if active { pal().ground } else { pal().dim });
    p.line_segment([c + egui::vec2(-2.5, -4.5), c + egui::vec2(2.0, 0.0)], st);
    p.line_segment([c + egui::vec2(2.0, 0.0), c + egui::vec2(-2.5, 4.5)], st);
}

/// The menu shortcut for a click-through `ui.button` in menus: icon chosen from the label.
pub fn mi(ui: &mut egui::Ui, label: &str) -> bool {
    menu_item(ui, label, "", true, false)
}

/// "Convert for web": two arrows turning around a small "A" (desktop font ⇄ web font).
pub fn paint_convert_icon(p: &egui::Painter, c: egui::Pos2, r: f32, col: Color32) {
    use egui::{vec2, Align2};
    let s = Stroke::new(1.25_f32, col);
    let pt = |a: f32| c + vec2(a.cos(), a.sin()) * r;
    for start in [-2.6f32, 0.54] {
        // Each arc covers ~130°, leaving gaps for the arrowheads.
        let end = start + 2.25;
        let n = 10;
        let pts: Vec<egui::Pos2> = (0..=n).map(|i| pt(start + (end - start) * i as f32 / n as f32)).collect();
        p.add(egui::Shape::line(pts, s));
        // Arrowhead at the end, pointing along the turn.
        let tip = pt(end);
        let dir = vec2(-(end).sin(), end.cos());
        let out = vec2(end.cos(), end.sin());
        let h = r * 0.42;
        p.line_segment([tip, tip - dir * h + out * h * 0.75], s);
        p.line_segment([tip, tip - dir * h - out * h * 0.75], s);
    }
    p.text(c + vec2(0.0, 0.5), Align2::CENTER_CENTER, "A", ui_font(r * 1.15), col);
}

/// "Restart to update": one arrow turning almost all the way round.
pub fn paint_restart(p: &egui::Painter, c: egui::Pos2, r: f32, col: Color32) {
    use egui::vec2;
    let s = Stroke::new(1.4_f32, col);
    let pt = |a: f32| c + vec2(a.cos(), a.sin()) * r;
    let (start, end) = (-1.2f32, -1.2 + 5.0);
    let n = 24;
    let pts: Vec<egui::Pos2> = (0..=n).map(|i| pt(start + (end - start) * i as f32 / n as f32)).collect();
    p.add(egui::Shape::line(pts, s));
    let tip = pt(end);
    let dir = vec2(-end.sin(), end.cos());
    let out = vec2(end.cos(), end.sin());
    let h = r * 0.55;
    p.line_segment([tip, tip - dir * h + out * h * 0.8], s);
    p.line_segment([tip, tip - dir * h - out * h * 0.8], s);
}

pub const MENU_CHECK_X: f32 = 12.0;
pub const MENU_ICON_X: f32 = 30.5;

/// Icons drawn for menu items, picked by the item's label (like SF Symbols).
pub fn paint_menu_icon(ui: &egui::Ui, c: egui::Pos2, label: &str, col: Color32) {
    use egui::{vec2, Align2, Rect};
    let p = ui.painter();
    let s = Stroke::new(1.25_f32, col);
    let l = label.trim_end_matches('…').trim_end_matches("...").to_lowercase();
    let glyph = |text: &str, family: &str, size: f32| {
        p.text(c, Align2::CENTER_CENTER, text, FontId::new(size, FontFamily::Name(family.into())), col);
    };
    // Specimen letters come from different fonts, each with its own line box: `dy` sets
    // them on one optical centre.
    let glyph_at = |text: &str, family: &str, size: f32, dy: f32| {
        p.text(c + vec2(0.0, dy), Align2::CENTER_CENTER, text, FontId::new(size, FontFamily::Name(family.into())), col);
    };
    match l.as_str() {
        "activate" | "auto-activate" | "activate family" | "activate all" => {
            paint_mark(p, c, 5.0, Mark::Active, col, col);
        }
        "activate temporarily" => {
            paint_mark(p, c, 5.0, Mark::Temporary, col, col);
        }
        "deactivate" | "deactivate family" | "deactivate all" => {
            paint_mark(p, c, 5.0, Mark::Off, col, col);
        }
        "star" | "unstar" | "starred" => {
            let pts: Vec<egui::Pos2> = (0..10)
                .map(|i| {
                    let r = if i % 2 == 0 { 6.0 } else { 2.7 };
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                    c + vec2(a.cos() * r, a.sin() * r)
                })
                .collect();
            if l == "unstar" {
                p.add(egui::Shape::convex_polygon(pts, col, Stroke::NONE));
            } else {
                p.add(egui::Shape::closed_line(pts, s));
            }
        }
        "category" => {
            p.text(c, Align2::CENTER_CENTER, "Aa", ui_bold(12.0), col);
        }
        "tags" | "new tag" => {
            p.line_segment([c + vec2(-1.2, -5.5), c + vec2(-2.8, 5.5)], s);
            p.line_segment([c + vec2(2.8, -5.5), c + vec2(1.2, 5.5)], s);
            p.line_segment([c + vec2(-5.0, -1.9), c + vec2(5.4, -1.9)], s);
            p.line_segment([c + vec2(-5.4, 1.9), c + vec2(5.0, 1.9)], s);
        }
        "add to set" | "new set" | "copy into library" | "select family" | "new collection" => {
            let back = Rect::from_center_size(c + vec2(-1.8, -1.8), vec2(9.5, 9.5));
            let front = Rect::from_center_size(c + vec2(1.8, 1.8), vec2(9.5, 9.5));
            p.rect_stroke(back, 2.5, s, egui::StrokeKind::Middle);
            p.rect_filled(front, 2.5, pal().menu);
            p.rect_stroke(front, 2.5, s, egui::StrokeKind::Middle);
        }
        "export" | "save as svg" | "copy svg" | "copy svg of first icon" => {
            let b = Rect::from_center_size(c + vec2(0.0, 2.5), vec2(11.0, 8.0));
            p.line_segment([b.left_top() + vec2(3.0, 0.0), b.left_top()], s);
            p.line_segment([b.left_top(), b.left_bottom()], s);
            p.line_segment([b.left_bottom(), b.right_bottom()], s);
            p.line_segment([b.right_bottom(), b.right_top()], s);
            p.line_segment([b.right_top(), b.right_top() - vec2(3.0, 0.0)], s);
            p.line_segment([c + vec2(0.0, 3.0), c + vec2(0.0, -6.5)], s);
            p.line_segment([c + vec2(-3.0, -3.5), c + vec2(0.0, -6.5)], s);
            p.line_segment([c + vec2(3.0, -3.5), c + vec2(0.0, -6.5)], s);
        }
        "show in finder" | "reveal in finder" => {
            p.line_segment([c + vec2(-6.0, 0.0), c + vec2(5.5, 0.0)], s);
            p.line_segment([c + vec2(1.5, -4.0), c + vec2(5.5, 0.0)], s);
            p.line_segment([c + vec2(1.5, 4.0), c + vec2(5.5, 0.0)], s);
        }
        "delete from library" | "delete" | "delete tag" | "delete collection" | "remove library" | "delete sample" => {
            let b = Rect::from_min_max(c + vec2(-4.5, -3.5), c + vec2(4.5, 6.5));
            p.rect_stroke(b, 1.5, s, egui::StrokeKind::Middle);
            p.line_segment([c + vec2(-6.5, -3.5), c + vec2(6.5, -3.5)], s);
            p.line_segment([c + vec2(-2.0, -6.0), c + vec2(2.0, -6.0)], s);
            p.line_segment([c + vec2(-1.5, -1.0), c + vec2(-1.5, 4.0)], s);
            p.line_segment([c + vec2(1.5, -1.0), c + vec2(1.5, 4.0)], s);
        }
        "remove from set" | "remove from fontlist" | "remove download" => {
            p.circle_stroke(c, 6.0, s);
            p.line_segment([c + vec2(-3.0, 0.0), c + vec2(3.0, 0.0)], s);
        }
        "download" | "import fonts" | "add folder" => {
            let b = Rect::from_center_size(c + vec2(0.0, 2.5), vec2(11.0, 8.0));
            p.line_segment([b.left_top() + vec2(3.0, 0.0), b.left_top()], s);
            p.line_segment([b.left_top(), b.left_bottom()], s);
            p.line_segment([b.left_bottom(), b.right_bottom()], s);
            p.line_segment([b.right_bottom(), b.right_top()], s);
            p.line_segment([b.right_top(), b.right_top() - vec2(3.0, 0.0)], s);
            p.line_segment([c + vec2(0.0, -6.5), c + vec2(0.0, 3.0)], s);
            p.line_segment([c + vec2(-3.0, 0.0), c + vec2(0.0, 3.0)], s);
            p.line_segment([c + vec2(3.0, 0.0), c + vec2(0.0, 3.0)], s);
        }
        "view details" | "info" => {
            p.circle_stroke(c, 6.0, s);
            p.text(c + vec2(0.0, 0.5), Align2::CENTER_CENTER, "i", ui_font(9.0), col);
        }
        "convert font for web" => paint_convert_icon(p, c, 6.5, col),
        "copy @font-face css" => {
            p.text(c, Align2::CENTER_CENTER, "{ }", ui_font(9.5), col);
        }
        "copy web embed code" => {
            p.line_segment([c + vec2(-3.5, -3.5), c + vec2(-7.0, 0.0)], s);
            p.line_segment([c + vec2(-7.0, 0.0), c + vec2(-3.5, 3.5)], s);
            p.line_segment([c + vec2(3.5, -3.5), c + vec2(7.0, 0.0)], s);
            p.line_segment([c + vec2(7.0, 0.0), c + vec2(3.5, 3.5)], s);
            p.line_segment([c + vec2(1.3, -5.0), c + vec2(-1.3, 5.0)], s);
        }
        "copy name" | "copy character" | "copy unicode" | "copy all font names" | "copy family name" | "copy font name" => {
            let back = Rect::from_center_size(c + vec2(-1.5, -1.5), vec2(8.5, 10.0));
            let front = Rect::from_center_size(c + vec2(1.5, 1.5), vec2(8.5, 10.0));
            p.rect_stroke(back, 1.5, s, egui::StrokeKind::Middle);
            p.rect_filled(front, 1.5, pal().menu);
            p.rect_stroke(front, 1.5, s, egui::StrokeKind::Middle);
        }
        "rename" => {
            p.line_segment([c + vec2(-5.0, 5.0), c + vec2(4.0, -4.0)], Stroke::new(1.6_f32, col));
            p.line_segment([c + vec2(-6.0, 6.0), c + vec2(-3.0, 6.0)], s);
        }
        "new library" | "open library" => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.5), vec2(12.0, 8.5));
            p.rect_stroke(r, 2.0, s, egui::StrokeKind::Middle);
            p.line_segment([r.left_top() + vec2(1.5, -2.5), r.right_top() + vec2(-1.5, -2.5)], s);
        }
        "new smart filter" | "edit smart filter" => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.0), vec2(12.0, 9.0));
            p.rect_stroke(r, 2.0, s, egui::StrokeKind::Middle);
            p.circle_stroke(r.right_top(), 2.4, s);
        }
        "sync" | "sync google fonts" | "check missing fonts now" => {
            p.circle_stroke(c, 5.0, s);
            p.line_segment([c + vec2(5.0, -2.0), c + vec2(5.0, 1.5)], s);
        }
        "apply" => {}
        // Category submenu: the letter "S" in each kind of type.
        "uncategorized" => {
            // No category: an empty, dashed ring.
            for k in 0..6 {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                let b = a + 0.62;
                p.line_segment([c + vec2(a.cos(), a.sin()) * 5.5, c + vec2(b.cos(), b.sin()) * 5.5], s);
            }
        }
        "sans serif" => glyph("S", "bold", 13.5),
        "serif" => glyph_at("S", "icon-serif", 15.0, 0.5),
        "slab serif" => glyph_at("S", "icon-slab", 14.5, 4.5),
        "script" => glyph_at("S", "icon-script", 16.0, 1.5),
        "blackletter" => glyph_at("\u{1D505}", "icon-fraktur", 15.0, 2.0),
        "display" => {
            p.text(c, Align2::CENTER_CENTER, "D", ui_bold(14.5), col);
            p.text(c, Align2::CENTER_CENTER, "D", ui_bold(11.0), pal().menu);
        }
        "symbols" => glyph("\u{261E}", "icon-dingbat", 13.0),
        "monospaced" => glyph_at("M", "icon-mono", 14.5, 0.5),
        _ => {
            if l.starts_with("apply to selected") || l.starts_with("apply in ") {
                p.text(c, Align2::CENTER_CENTER, "T", ui_bold(12.0), col);
                p.line_segment([c + vec2(-5.0, 6.0), c + vec2(5.0, 6.0)], s);
            } else if l.starts_with("copy into") {
                let back = Rect::from_center_size(c + vec2(-1.8, -1.8), vec2(9.5, 9.5));
                p.rect_stroke(back, 2.5, s, egui::StrokeKind::Middle);
            } else if l.starts_with("save ") {
                p.line_segment([c + vec2(0.0, -6.0), c + vec2(0.0, 3.0)], s);
                p.line_segment([c + vec2(-3.0, 0.0), c + vec2(0.0, 3.0)], s);
                p.line_segment([c + vec2(3.0, 0.0), c + vec2(0.0, 3.0)], s);
                p.line_segment([c + vec2(-5.5, 6.0), c + vec2(5.5, 6.0)], s);
            }
        }
    }
}

pub fn menu_separator(ui: &mut egui::Ui) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 11.0), egui::Sense::hover());
    ui.painter().line_segment(
        [egui::pos2(rect.left() + 10.0, rect.center().y), egui::pos2(rect.right() - 10.0, rect.center().y)],
        Stroke::new(1.0_f32, pal().rule),
    );
}

/// A checkbox: an ink square with a ground tick when on, an ink frame when off.
pub fn mac_check(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    mac_check_sized(ui, on, label, 14.0, true)
}

pub fn mac_check_sized(ui: &mut egui::Ui, on: &mut bool, label: &str, size: f32, enabled: bool) -> egui::Response {
    let galley = ui.fonts(|f| f.layout_no_wrap(label.to_string(), ui_font(size), pal().ink));
    let box_s = 16.0;
    let w = box_s + if label.is_empty() { 0.0 } else { 10.0 + galley.size().x };
    let (rect, mut resp) =
        ui.allocate_exact_size(egui::vec2(w, box_s.max(galley.size().y) + 2.0), if enabled { egui::Sense::click() } else { egui::Sense::hover() });
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let b = egui::Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - box_s / 2.0), egui::vec2(box_s, box_s));
    let p = ui.painter();
    let dim = if enabled { 1.0 } else { 0.45 };
    let c = pal();
    if *on {
        p.rect_filled(b, 4.0, c.ink.gamma_multiply(dim));
        let s = Stroke::new(2.0_f32, c.ground);
        p.line_segment([b.min + egui::vec2(3.8, 8.4), b.min + egui::vec2(6.8, 11.4)], s);
        p.line_segment([b.min + egui::vec2(6.8, 11.4), b.min + egui::vec2(12.4, 4.6)], s);
    } else {
        p.rect_filled(b, 4.0, c.field);
        p.rect_stroke(b, 4.0, Stroke::new(1.5_f32, c.ink.gamma_multiply(dim)), egui::StrokeKind::Inside);
    }
    focus_ring(ui, &resp, b, 4.0);
    if !label.is_empty() {
        let col = if enabled { pal().ink } else { pal().faint };
        p.galley_with_override_text_color(egui::pos2(b.right() + 10.0, rect.center().y - galley.size().y / 2.0), galley, col);
    }
    resp
}

// ----- status marks -----

/// What a font is doing, said by shape so it never depends on colour: a filled disc is on,
/// a half disc is on for now, a disc inside a ring was switched on by a design app, an empty
/// ring is off, an open ring is downloading.
#[derive(Clone, Copy, PartialEq)]
pub enum Mark {
    Active,
    Temporary,
    Auto,
    Off,
    Busy,
}

impl Mark {
    pub fn word(self) -> &'static str {
        match self {
            Mark::Active => "Active",
            Mark::Temporary => "Temporary",
            Mark::Auto => "Auto",
            Mark::Off => "Off",
            Mark::Busy => "Downloading",
        }
    }
}

/// Draws a status mark of radius `r`; `off` is the colour of the empty ring.
pub fn paint_mark(p: &egui::Painter, c: egui::Pos2, r: f32, mark: Mark, col: Color32, off: Color32) {
    let w = (r * 0.36).max(1.3);
    match mark {
        Mark::Active => {
            p.circle_filled(c, r, col);
        }
        Mark::Temporary => {
            let n = 14;
            let mut pts: Vec<egui::Pos2> = (0..=n)
                .map(|i| {
                    let a = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / n as f32;
                    c + egui::vec2(a.cos(), a.sin()) * r
                })
                .collect();
            pts.dedup();
            p.add(egui::Shape::convex_polygon(pts, col, Stroke::NONE));
            p.circle_stroke(c, r - w / 2.0, Stroke::new(w, col));
        }
        Mark::Auto => {
            p.circle_stroke(c, r - w / 2.0, Stroke::new(w, col));
            p.circle_filled(c, (r - w - 1.2).max(1.2), col);
        }
        Mark::Off => {
            p.circle_stroke(c, r - w / 2.0, Stroke::new(w, off));
        }
        Mark::Busy => {
            let n = 16;
            let pts: Vec<egui::Pos2> = (0..=n)
                .map(|i| {
                    let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * 0.72 * i as f32 / n as f32;
                    c + egui::vec2(a.cos(), a.sin()) * (r - w / 2.0)
                })
                .collect();
            p.add(egui::Shape::line(pts, Stroke::new(w, col)));
        }
    }
}

/// A five-pointed star, outlined or filled.
pub fn paint_star(p: &egui::Painter, c: egui::Pos2, r: f32, filled: bool, col: Color32) {
    let pts: Vec<egui::Pos2> = (0..10)
        .map(|i| {
            let rr = if i % 2 == 0 { r } else { r * 0.45 };
            let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            c + egui::vec2(a.cos() * rr, a.sin() * rr)
        })
        .collect();
    if filled {
        // A star is not convex: fill it as five triangles around its centre pentagon.
        for i in 0..5 {
            let tri = vec![pts[(2 * i + 9) % 10], pts[2 * i], pts[(2 * i + 1) % 10]];
            p.add(egui::Shape::convex_polygon(tri, col, Stroke::NONE));
        }
        let inner: Vec<egui::Pos2> = (0..5).map(|i| pts[2 * i + 1]).collect();
        p.add(egui::Shape::convex_polygon(inner, col, Stroke::NONE));
    } else {
        p.add(egui::Shape::closed_line(pts, Stroke::new(1.2_f32, col)));
    }
}

/// A small cross, for clearing a field or closing.
pub fn paint_cross(p: &egui::Painter, c: egui::Pos2, r: f32, col: Color32) {
    let s = Stroke::new(1.4_f32, col);
    p.line_segment([c + egui::vec2(-r, -r), c + egui::vec2(r, r)], s);
    p.line_segment([c + egui::vec2(-r, r), c + egui::vec2(r, -r)], s);
}

/// A downward arrow into a tray: not downloaded yet.
pub fn paint_download(p: &egui::Painter, c: egui::Pos2, col: Color32) {
    let s = Stroke::new(1.3_f32, col);
    p.line_segment([c + egui::vec2(0.0, -5.0), c + egui::vec2(0.0, 2.0)], s);
    p.line_segment([c + egui::vec2(-3.0, -1.0), c + egui::vec2(0.0, 2.0)], s);
    p.line_segment([c + egui::vec2(3.0, -1.0), c + egui::vec2(0.0, 2.0)], s);
    p.line_segment([c + egui::vec2(-4.5, 5.0), c + egui::vec2(4.5, 5.0)], s);
}

// ----- keyboard focus -----

/// Remembers whether the person is moving with the keyboard (Tab) or the pointer, so focus
/// rings show for the keyboard only. Call once per frame.
pub fn track_input(ctx: &egui::Context) {
    let (tab, press) = ctx.input(|i| (i.key_pressed(egui::Key::Tab), i.pointer.any_pressed()));
    if tab || press {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("kbd-nav"), tab));
    }
}

/// The ring around the control that has keyboard focus.
pub fn focus_ring(ui: &egui::Ui, resp: &egui::Response, rect: egui::Rect, radius: f32) {
    if resp.has_focus() && ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::new("kbd-nav")).unwrap_or(false)) {
        let c = pal();
        ui.painter().rect_stroke(rect.expand(2.0), radius + 2.0, Stroke::new(2.0_f32, c.ground), egui::StrokeKind::Outside);
        ui.painter().rect_stroke(rect.expand(4.0), radius + 4.0, Stroke::new(2.0_f32, c.ink), egui::StrokeKind::Outside);
    }
}

// ----- buttons -----

#[derive(Clone, Copy, PartialEq)]
pub enum Btn {
    /// The one action a panel or dialog exists for: ink fill.
    Primary,
    /// Everything else: an ink frame.
    Plain,
    /// Deletes something: danger frame and lettering.
    Danger,
}

/// A button of the given width (0 = as wide as its label needs).
pub fn button_w(ui: &mut egui::Ui, label: &str, kind: Btn, enabled: bool, width: f32, height: f32) -> bool {
    let c = pal();
    let font = if kind == Btn::Primary { ui_bold(13.5) } else { ui_medium(13.0) };
    let g = ui.fonts(|f| f.layout_no_wrap(label.to_string(), font, c.ink));
    let w = if width > 0.0 { width } else { (g.size().x + 28.0).max(72.0) };
    let (rect, r) = ui.allocate_exact_size(egui::vec2(w, height), if enabled { egui::Sense::click() } else { egui::Sense::hover() });
    let ht = fade(ui, ("btn-h", label, kind as u8), r.hovered() && enabled);
    let dimmed = if enabled { 1.0 } else { 0.4 };
    let (fill, text, frame) = match kind {
        Btn::Primary => (mix(c.ink, c.dim, ht * 0.35), c.ground, Color32::TRANSPARENT),
        Btn::Plain => (mix(c.ground, c.panel_hi, ht), c.ink, c.ink),
        Btn::Danger => (mix(c.ground, c.panel_hi, ht), c.danger, c.danger),
    };
    let p = ui.painter();
    p.rect_filled(rect, CTRL_RADIUS, fill.gamma_multiply(if kind == Btn::Primary { dimmed } else { 1.0 }));
    if frame != Color32::TRANSPARENT {
        p.rect_stroke(rect, CTRL_RADIUS, Stroke::new(1.0_f32, frame.gamma_multiply(dimmed)), egui::StrokeKind::Inside);
    }
    let text = if kind == Btn::Primary { text } else { text.gamma_multiply(dimmed) };
    p.galley_with_override_text_color(rect.center() - g.size() / 2.0, g, text);
    focus_ring(ui, &r, rect, CTRL_RADIUS);
    r.clicked() && enabled
}

pub fn button(ui: &mut egui::Ui, label: &str, kind: Btn) -> bool {
    button_w(ui, label, kind, true, 0.0, 28.0)
}

/// A word that switches something on (a tab, an alignment): ink fill and ground lettering
/// when on.
pub fn toggle(ui: &mut egui::Ui, label: &str, on: bool) -> egui::Response {
    let c = pal();
    let g = ui.fonts(|f| f.layout_no_wrap(label.to_string(), ui_medium(12.5), c.ink));
    let (rect, r) = ui.allocate_exact_size(egui::vec2(g.size().x + 16.0, 24.0), egui::Sense::click());
    let ht = fade(ui, ("tg-h", label), r.hovered());
    let st = fade(ui, ("tg-s", label), on);
    let fill = mix(mix(Color32::TRANSPARENT, c.panel_hi, ht), c.ink, st);
    ui.painter().rect_filled(rect, CTRL_RADIUS, fill);
    ui.painter().galley_with_override_text_color(rect.center() - g.size() / 2.0, g, mix(c.ink, c.ground, st));
    focus_ring(ui, &r, rect, CTRL_RADIUS);
    r
}

// ----- eased motion -----

/// A CSS-style cubic Bézier easing curve from (0, 0) to (1, 1) through (x1, y1) and (x2, y2).
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let bez = |a: f32, b: f32, s: f32| 3.0 * a * s * (1.0 - s).powi(2) + 3.0 * b * s * s * (1.0 - s) + s * s * s;
    let d_bez = |a: f32, b: f32, s: f32| 3.0 * a * (1.0 - s).powi(2) + 6.0 * (b - a) * s * (1.0 - s) + 3.0 * (1.0 - b) * s * s;
    // Solve x(s) = t for s (Newton, with a bisection fallback), then return y(s).
    let mut s = t;
    for _ in 0..8 {
        let dx = d_bez(x1, x2, s);
        if dx.abs() < 1e-5 {
            break;
        }
        s = (s - (bez(x1, x2, s) - t) / dx).clamp(0.0, 1.0);
    }
    if (bez(x1, x2, s) - t).abs() > 1e-3 {
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..24 {
            s = (lo + hi) / 2.0;
            if bez(x1, x2, s) < t { lo = s } else { hi = s }
        }
    }
    bez(y1, y2, s)
}

/// Glide: a fast start that settles softly (cubic-bezier(0.05, 0.7, 0.1, 1)), for things that
/// travel to where the pointer just was.
pub const GLIDE: (f32, f32, f32, f32) = (0.05, 0.7, 0.1, 1.0);
pub const GLIDE_TIME: f32 = 0.16;

/// Moves a value to `target` over `secs` along `curve`; a new target starts from wherever
/// the value is now, so a quick second click never jumps.
pub fn ease_to(ctx: &egui::Context, id: egui::Id, target: f32, secs: f32, curve: (f32, f32, f32, f32)) -> f32 {
    let now = ctx.input(|i| i.time);
    // (from, to, start time)
    // Nothing stored yet: the value is already at its target, with nothing left to animate.
    let Some((from, to, start)) = ctx.data(|d| d.get_temp::<(f32, f32, f64)>(id)) else {
        ctx.data_mut(|d| d.insert_temp(id, (target, target, now - secs as f64)));
        return target;
    };
    let p = if secs <= 0.0 { 1.0 } else { ((now - start) as f32 / secs).clamp(0.0, 1.0) };
    let current = from + (to - from) * cubic_bezier(curve.0, curve.1, curve.2, curve.3, p);
    if (target - to).abs() > 0.01 {
        ctx.data_mut(|d| d.insert_temp(id, (current, target, now)));
        ctx.request_repaint();
        return current;
    }
    if p < 1.0 {
        ctx.request_repaint();
    }
    current
}
