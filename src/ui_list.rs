//! List view as a table: a header with columns (Name, Preview, Styles, ☆,
//! Format, Category, Date Added) that sort when clicked, one row per family with a disclosure
//! arrow and its styles indented below, striped rows.
use crate::app::KernshipApp;
use crate::model::{Category, FontFace, Source};
use crate::store::SortBy;
use crate::theme::*;
use crate::ui_main::{handle_click, preview_image_with, status_mark};
use egui::{pos2, vec2, Align2, Rect, Sense, Stroke, Ui};

const HEADER: f32 = 26.0;
/// Name column: status dot, disclosure arrow, family name, and the
/// style names indented under it.
const DOT_X: f32 = 14.0;
const ARROW_X: f32 = 34.0;
const NAME_X: f32 = 46.0;
const STYLE_X: f32 = 68.0;

#[derive(Clone, Copy, PartialEq)]
enum Col {
    Name,
    Preview,
    Styles,
    Star,
    Format,
    Category,
    Date,
}

struct Layout {
    left: f32,
    cols: Vec<(Col, f32, f32)>,
}

impl Layout {
    fn new(left: f32, width: f32) -> Self {
        // Fixed columns from the right; the preview takes the rest (dropped right-to-left
        // when the window is narrow).
        let mut fixed = vec![(Col::Styles, 90.0), (Col::Star, 30.0), (Col::Format, 160.0), (Col::Category, 120.0), (Col::Date, 170.0)];
        let name_w = 250.0;
        while !fixed.is_empty() && width - name_w - fixed.iter().map(|c| c.1).sum::<f32>() < 240.0 {
            fixed.pop();
        }
        let preview_w = width - name_w - fixed.iter().map(|c| c.1).sum::<f32>();
        let mut cols = Vec::new();
        let mut x = left;
        cols.push((Col::Name, x, name_w));
        x += name_w;
        cols.push((Col::Preview, x, preview_w));
        x += preview_w;
        for (c, w) in fixed {
            cols.push((c, x, w));
            x += w;
        }
        Self { left, cols }
    }
    fn get(&self, c: Col) -> Option<(f32, f32)> {
        self.cols.iter().find(|x| x.0 == c).map(|x| (x.1, x.2))
    }
}

fn title(c: Col) -> &'static str {
    match c {
        Col::Name => "Name",
        Col::Preview => "Preview",
        Col::Styles => "Styles",
        Col::Star => "",
        Col::Format => "Format",
        Col::Category => "Category",
        Col::Date => "Date Added",
    }
}

fn sort_for(c: Col) -> Option<SortBy> {
    match c {
        Col::Name => Some(SortBy::Name),
        Col::Styles => Some(SortBy::Styles),
        Col::Star => Some(SortBy::Starred),
        Col::Format => Some(SortBy::Format),
        Col::Category => Some(SortBy::Category),
        Col::Date => Some(SortBy::DateAdded),
        Col::Preview => None,
    }
}

/// "28 Jan 2020 at 17:45" in local time.
fn fmt_time(secs: i64) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let t: libc::time_t = secs as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&t, &mut tm) };
    format!("{} {} {} at {:02}:{:02}", tm.tm_mday, MONTHS[(tm.tm_mon as usize).min(11)], tm.tm_year + 1900, tm.tm_hour, tm.tm_min)
}

fn date_of(app: &KernshipApp, f: &FontFace) -> String {
    if let Some(d) = app.dates.borrow().get(&f.id) {
        return d.clone();
    }
    let d = if f.added > 0 {
        fmt_time(f.added as i64)
    } else if !f.date_added.is_empty() {
        // Google's catalog: "2012-09-30".
        let parts: Vec<&str> = f.date_added.split('-').collect();
        const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        match parts.as_slice() {
            [y, m, d] => format!("{} {} {}", d.trim_start_matches('0'), MONTHS[m.parse::<usize>().unwrap_or(1).clamp(1, 12) - 1], y),
            _ => f.date_added.clone(),
        }
    } else {
        f.path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.created().or_else(|_| m.modified()).ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| fmt_time(d.as_secs() as i64))
            .unwrap_or_default()
    };
    app.dates.borrow_mut().insert(f.id.clone(), d.clone());
    d
}

enum Row {
    /// Family row: group index; `single` when the family has one style.
    Family(usize),
    /// A style under its family: (group, face index in the group).
    Style(usize, usize),
}

fn collapsed(app: &KernshipApp, family: &str) -> bool {
    app.list_collapsed.contains(family) == app.st.expand_families
}

pub fn draw(app: &mut KernshipApp, ui: &mut Ui, area: Rect, now: f64) {
    let size = app.st.preview_size();
    let row_h = (size * 1.3 + 5.0).max(24.0);
    let lay = Layout::new(area.left() + 4.0, area.width() - 14.0);

    // Header (stays put while the rows scroll).
    let header = Rect::from_min_size(pos2(area.left(), area.top() + 2.0), vec2(area.width() - 6.0, HEADER));
    let p = ui.painter().clone();
    p.rect_filled(header, 0.0, pal().panel);
    p.line_segment([header.left_bottom(), header.right_bottom()], Stroke::new(1.0_f32, pal().rule));
    for (c, x, w) in lay.cols.clone() {
        let r = Rect::from_min_size(pos2(x, header.top()), vec2(w, HEADER));
        let resp = ui.interact(r, ui.id().with(("list-head", title(c), x as i32)), Sense::click());
        let active = sort_for(c) == Some(app.st.sort_by) && c != Col::Preview;
        let ht = fade(ui, ("list-head-h", title(c)), resp.hovered() && sort_for(c).is_some());
        if ht > 0.0 {
            p.rect_filled(r, 0.0, pal().panel_hi.gamma_multiply(ht));
        }
        let tx = if c == Col::Name { x + NAME_X } else { x + 8.0 };
        if c == Col::Star {
            paint_star(&p, r.center(), 5.5, false, pal().dim);
        } else {
            p.text(pos2(tx, r.center().y), Align2::LEFT_CENTER, title(c), if active { ui_bold(11.5) } else { ui_medium(11.5) }, if active { pal().ink } else { pal().dim });
        }
        if active {
            let ax = r.right() - 12.0;
            let up = app.st.sort_ascending;
            let cy = r.center().y;
            let (a, b) = if up { (-2.0, 2.0) } else { (2.0, -2.0) };
            p.line_segment([pos2(ax - 3.5, cy + b), pos2(ax, cy + a)], Stroke::new(1.2_f32, pal().dim));
            p.line_segment([pos2(ax, cy + a), pos2(ax + 3.5, cy + b)], Stroke::new(1.2_f32, pal().dim));
        }
        if c != Col::Name {
            p.line_segment([pos2(x, header.top() + 6.0), pos2(x, header.bottom() - 6.0)], Stroke::new(1.0_f32, pal().rule));
        }
        if resp.clicked() {
            if let Some(sb) = sort_for(c) {
                if app.st.sort_by == sb {
                    app.st.sort_ascending = !app.st.sort_ascending;
                } else {
                    app.st.sort_by = sb;
                    app.st.sort_ascending = true;
                }
                app.dirty = true;
                app.save_soon(now);
            }
        }
    }

    // Rows: each family, then its styles when it is open.
    let mut rows: Vec<Row> = Vec::new();
    for (gi, g) in app.groups.iter().enumerate() {
        rows.push(Row::Family(gi));
        if g.faces.len() > 1 && !collapsed(app, &g.family) {
            for k in 0..g.faces.len() {
                rows.push(Row::Style(gi, k));
            }
        }
    }
    let body = Rect::from_min_max(pos2(area.left(), header.bottom() + 2.0), area.max);
    let total = rows.len() as f32 * row_h + 90.0;

    let mut target = None;
    if app.center_selected > 0 {
        app.center_selected -= 1;
        if let Some(sel) = app.selected.first().and_then(|id| app.by_id.get(id).copied()) {
            if let Some(i) = rows.iter().position(|r| match *r {
                Row::Style(gi, k) => app.groups[gi].faces[k] == sel,
                Row::Family(gi) => app.groups[gi].faces.len() == 1 && app.groups[gi].faces[0] == sel,
            }) {
                target = Some((i as f32 * row_h + row_h / 2.0 - body.height() / 2.0).clamp(0.0, (total - body.height()).max(0.0)));
            }
        }
        ui.ctx().request_repaint();
    }

    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
    child.set_clip_rect(body);
    let mut sa = egui::ScrollArea::vertical().id_salt(format!("list-{:?}", app.nav)).auto_shrink([false, false]);
    if let Some(t) = target {
        sa = sa.vertical_scroll_offset(t);
    }
    sa.show_viewport(&mut child, |ui, vp| {
        ui.set_height(total);
        let top0 = ui.min_rect().top();
        let ppp = ui.ctx().pixels_per_point();
        let first = (vp.min.y / row_h).floor().max(0.0) as usize;
        let last = ((vp.max.y / row_h).ceil() as usize + 1).min(rows.len());
        for i in first..last {
            let rect = Rect::from_min_size(pos2(area.left() + 2.0, top0 + i as f32 * row_h), vec2(area.width() - 12.0, row_h));
            if i % 2 == 1 {
                ui.painter().rect_filled(rect, 4.0, pal().panel);
            }
            match rows[i] {
                Row::Family(gi) => family_row(app, ui, &lay, rect, gi, size, ppp, now),
                Row::Style(gi, k) => {
                    let idx = app.groups[gi].faces[k];
                    style_row(app, ui, &lay, rect, idx, true, size, ppp, now);
                }
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn cells(app: &mut KernshipApp, ui: &Ui, lay: &Layout, rect: Rect, idx: usize, styles: Option<usize>, starred: bool) {
    let p = ui.painter();
    let cy = rect.center().y;
    let f = app.all[idx].clone();
    if let (Some(n), Some((x, _))) = (styles, lay.get(Col::Styles)) {
        p.text(pos2(x + 8.0, cy), Align2::LEFT_CENTER, format!("{n} style{}", if n == 1 { "" } else { "s" }), ui_font(12.5), pal().dim);
    }
    if let Some((x, w)) = lay.get(Col::Star) {
        if starred {
            paint_star(p, pos2(x + w / 2.0, cy), 5.5, true, pal().ink);
        }
    }
    if let Some((x, _)) = lay.get(Col::Format) {
        let fmt = if f.path.is_some() || f.source != Source::Google { f.format_name() } else { "Google Fonts" };
        p.text(pos2(x + 8.0, cy), Align2::LEFT_CENTER, fmt, ui_font(12.5), pal().dim);
    }
    if let Some((x, _)) = lay.get(Col::Category) {
        if f.category != Category::Unknown {
            p.text(pos2(x + 8.0, cy), Align2::LEFT_CENTER, f.category.label(), ui_font(12.5), pal().dim);
        }
    }
    if let Some((x, _)) = lay.get(Col::Date) {
        let d = date_of(app, &f);
        p.text(pos2(x + 8.0, cy), Align2::LEFT_CENTER, d, ui_font(12.5), pal().dim);
    }
}

fn row_fill(ui: &Ui, rect: Rect, id: &str, hovered: bool, selected: bool) {
    let ht = fade(ui, ("lrow-h", id), hovered);
    let st = fade(ui, ("lrow-s", id), selected);
    if ht > 0.0 {
        ui.painter().rect_filled(rect, 4.0, pal().panel_hi.gamma_multiply(ht * (1.0 - st)));
    }
    if st > 0.0 {
        ui.painter().rect_filled(rect, 4.0, pal().sel_fill.gamma_multiply(st));
    }
}

#[allow(clippy::too_many_arguments)]
fn style_row(app: &mut KernshipApp, ui: &mut Ui, lay: &Layout, rect: Rect, idx: usize, indented: bool, size: f32, ppp: f32, now: f64) {
    let id = app.all[idx].id.clone();
    let resp = ui.interact(rect, ui.id().with(("lrow", &id)), Sense::click_and_drag());
    let selected = app.selected.contains(&id);
    row_fill(ui, rect, &id, resp.hovered(), selected);
    let cy = rect.center().y;
    let dot = pos2(lay.left + DOT_X, cy);
    paint_mark(ui.painter(), dot, 4.5, status_mark(app, idx), pal().ink, pal().dim);
    let f = &app.all[idx];
    let (nx, nw) = lay.get(Col::Name).unwrap();
    let label = if indented { f.style.clone() } else { f.family.clone() };
    let tx = if indented { nx + STYLE_X } else { nx + NAME_X };
    let g = ui.fonts(|fo| {
        let mut job = egui::text::LayoutJob::simple_singleline(label, ui_font(13.0), pal().ink);
        job.wrap = egui::text::TextWrapping::truncate_at_width(nx + nw - tx - 8.0);
        fo.layout_job(job)
    });
    ui.painter().galley(pos2(tx, cy - g.size().y / 2.0), g, pal().ink);
    let (px, pw) = lay.get(Col::Preview).unwrap();
    let text_rect = Rect::from_min_max(pos2(px + 8.0, cy - size * 0.62), pos2(px + pw - 8.0, cy + size * 0.7));
    preview_image_with(app, ui, idx, None, text_rect, ppp, pal().ink);
    let starred = app.st.starred.contains(&id);
    cells(app, ui, lay, rect, idx, if indented { None } else { Some(1) }, starred);
    if resp.drag_started() {
        if !selected {
            app.selected = vec![id.clone()];
        }
        ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("dragging-fonts"), true));
    }
    handle_click(app, ui, &resp, idx, Rect::from_center_size(dot, vec2(12.0, 12.0)), now);
}

#[allow(clippy::too_many_arguments)]
fn family_row(app: &mut KernshipApp, ui: &mut Ui, lay: &Layout, rect: Rect, gi: usize, size: f32, ppp: f32, now: f64) {
    let g = &app.groups[gi];
    if g.faces.len() == 1 {
        let idx = g.faces[0];
        return style_row(app, ui, lay, rect, idx, false, size, ppp, now);
    }
    let family = g.family.clone();
    let ids: Vec<String> = g.faces.iter().map(|&i| app.all[i].id.clone()).collect();
    let n = ids.len();
    let rep = crate::ui_main::representative(app, gi);
    let key = format!("fam:{family}");
    let resp = ui.interact(rect, ui.id().with(("lfam", &family)), Sense::click_and_drag());
    let selected = !ids.is_empty() && ids.iter().all(|i| app.selected.contains(i));
    row_fill(ui, rect, &key, resp.hovered(), selected);
    let cy = rect.center().y;
    let p = ui.painter().clone();
    let any_active = g.faces.iter().any(|&i| app.is_active(&app.all[i]));
    let dot = pos2(lay.left + DOT_X, cy);
    paint_mark(&p, dot, 4.5, if any_active { Mark::Active } else { Mark::Off }, pal().ink, pal().dim);
    let (nx, nw) = lay.get(Col::Name).unwrap();
    // Disclosure arrow, turning over 200 ms.
    let open = !collapsed(app, &family);
    let t = fade(ui, ("lfam-open", &family), open);
    let ac = pos2(nx + ARROW_X, cy);
    let a = t * std::f32::consts::FRAC_PI_2;
    let rot = |v: egui::Vec2| vec2(v.x * a.cos() - v.y * a.sin(), v.x * a.sin() + v.y * a.cos());
    let s = Stroke::new(1.3_f32, pal().dim);
    p.line_segment([ac + rot(vec2(-1.8, -3.5)), ac + rot(vec2(1.8, 0.0))], s);
    p.line_segment([ac + rot(vec2(1.8, 0.0)), ac + rot(vec2(-1.8, 3.5))], s);
    let tx = nx + NAME_X;
    let gal = ui.fonts(|fo| {
        let mut job = egui::text::LayoutJob::simple_singleline(family.clone(), ui_font(13.0), pal().ink);
        job.wrap = egui::text::TextWrapping::truncate_at_width(nx + nw - tx - 8.0);
        fo.layout_job(job)
    });
    p.galley(pos2(tx, cy - gal.size().y / 2.0), gal, pal().ink);
    let (px, pw) = lay.get(Col::Preview).unwrap();
    let text_rect = Rect::from_min_max(pos2(px + 8.0, cy - size * 0.62), pos2(px + pw - 8.0, cy + size * 0.7));
    preview_image_with(app, ui, rep, Some(&family), text_rect, ppp, pal().ink);
    let starred = ids.iter().any(|i| app.st.starred.contains(i));
    cells(app, ui, lay, rect, rep, Some(n), starred);

    if resp.clicked() {
        let pos = resp.interact_pointer_pos().unwrap_or_default();
        if (pos.x - ac.x).abs() < 10.0 {
            if !app.list_collapsed.remove(&family) {
                app.list_collapsed.insert(family.clone());
            }
        } else if (pos - dot).length() < 11.0 {
            app.set_active(&ids, !any_active, now);
        } else if ui.input(|i| i.modifiers.command) {
            for id in &ids {
                if !app.selected.contains(id) {
                    app.selected.push(id.clone());
                }
            }
        } else {
            app.selected = ids.clone();
        }
    }
    if resp.double_clicked()
        && !app.list_collapsed.remove(&family) {
            app.list_collapsed.insert(family.clone());
        }
    if resp.secondary_clicked() && !selected {
        app.selected = ids.clone();
    }
    if resp.drag_started() {
        app.selected = ids.clone();
        ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("dragging-fonts"), true));
    }
    resp.context_menu(|ui| {
        menu_style(ui);
        crate::ui_main::font_menu(app, ui, now)
    });
}
