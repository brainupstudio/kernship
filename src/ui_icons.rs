//! The Icon Fonts view: icon sets shown as icons (not fonts), with their names.
//! Cell = icon size + 52, square; spacing 12 across and 16 down;
//! set header 32 tall.
use crate::app::KernshipApp;
use crate::model::Nav;
use crate::render::RenderKey;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Rect, Sense, Stroke, StrokeKind, Ui};

const CELL_EXTRA: f32 = 52.0;
const GAP_X: f32 = 12.0;
const GAP_Y: f32 = 16.0;
const HEADER: f32 = 32.0;
const PAD: f32 = 12.0;

enum Row {
    Header(usize),
    Icons(usize, usize, usize),
}

fn visible_sets(app: &KernshipApp) -> Vec<usize> {
    app.icon_sets
        .iter()
        .enumerate()
        .filter(|(_, s)| match &app.nav {
            Nav::IconFamily(id) => &s.id == id,
            _ => true,
        })
        .map(|(i, _)| i)
        .collect()
}

fn filtered(app: &KernshipApp, set: usize) -> Vec<usize> {
    let q = app.search.to_lowercase();
    app.icon_sets[set]
        .glyphs
        .iter()
        .enumerate()
        .filter(|(_, g)| q.is_empty() || g.name.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

pub fn draw(app: &mut KernshipApp, ui: &mut Ui, area: Rect, now: f64) {
    let size = app.st.icon_size;
    let cell = size + CELL_EXTRA;
    let inner_w = area.width() - PAD - 4.0;
    let cols = (((inner_w + GAP_X) / (cell + GAP_X)).floor() as usize).max(1);

    let sets = visible_sets(app);
    let lists: Vec<(usize, Vec<usize>)> = sets.iter().map(|&s| (s, filtered(app, s))).collect();
    let mut rows: Vec<(f32, f32, Row)> = Vec::new();
    let mut y = 0.0;
    for (k, (s, list)) in lists.iter().enumerate() {
        let remote = app.icon_sets[*s].remote.is_some();
        if !remote && list.is_empty() && !app.search.is_empty() {
            continue;
        }
        rows.push((y, HEADER, Row::Header(k)));
        y += HEADER;
        let mut i = 0;
        while i < list.len() {
            let end = (i + cols).min(list.len());
            rows.push((y, cell + GAP_Y, Row::Icons(k, i, end)));
            y += cell + GAP_Y;
            i = end;
        }
        y += 8.0;
    }
    let total = y + 90.0;

    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area));
    child.set_clip_rect(area);
    if rows.is_empty() {
        let msg = if app.loading_icons { "Loading icon sets…" } else { "No Icons" };
        child.painter().text(area.center(), Align2::CENTER_CENTER, msg, ui_bold(18.0), pal().ink);
        return;
    }
    let ppp = ui.ctx().pixels_per_point();
    egui::ScrollArea::vertical()
        .id_salt(format!("icons-{:?}", app.nav))
        .auto_shrink([false, false])
        .show_viewport(&mut child, |ui, vp| {
            ui.set_height(total);
            let origin = ui.min_rect().min + vec2(PAD - 8.0, 0.0);
            for (ry, rh, row) in rows.iter() {
                if ry + rh < vp.min.y - 4.0 || *ry > vp.max.y + 4.0 {
                    continue;
                }
                let top = origin.y + ry;
                match *row {
                    Row::Header(k) => header(app, ui, &lists, k, Rect::from_min_size(pos2(origin.x, top), vec2(inner_w, HEADER))),
                    Row::Icons(k, a, b) => {
                        let (s, list) = &lists[k];
                        for (c, gi) in list[a..b].iter().enumerate() {
                            let r = Rect::from_min_size(pos2(origin.x + c as f32 * (cell + GAP_X), top), vec2(cell, cell));
                            icon_cell(app, ui, *s, *gi, r, size, ppp, now);
                        }
                    }
                }
            }
        });
}

fn header(app: &mut KernshipApp, ui: &mut Ui, lists: &[(usize, Vec<usize>)], k: usize, r: Rect) {
    let (s, list) = &lists[k];
    let set = app.icon_sets[*s].clone();
    let cy = r.center().y;
    ui.painter().text(pos2(r.left() + 4.0, cy), Align2::LEFT_CENTER, &set.name, ui_bold(14.0), pal().ink);
    if let Some(remote) = &set.remote {
        let busy = app.icon_downloads.contains(&set.id);
        let label = if busy { "Downloading…" } else { "Download" };
        let b = Rect::from_min_size(pos2(r.right() - 104.0, cy - 11.0), vec2(100.0, 22.0));
        let resp = ui.interact(b, ui.id().with(("icon-dl", &set.id)), Sense::click());
        ui.painter().rect_filled(b, CTRL_RADIUS, if resp.hovered() && !busy { pal().dim } else { pal().ink });
        ui.painter().text(b.center(), Align2::CENTER_CENTER, label, ui_bold(12.0), pal().ground);
        if resp.clicked() && !busy {
            app.download_icon_set(&set.id);
        }
        let source = if remote.font_url.is_some() { "Free icon set" } else { "Google Fonts" };
        ui.painter().text(pos2(b.left() - 10.0, cy), Align2::RIGHT_CENTER, source, ui_font(11.5), pal().dim);
    } else {
        let n = list.len();
        ui.painter().text(pos2(r.right() - 2.0, cy), Align2::RIGHT_CENTER, format!("{} icons", crate::ui_main::thousands(n)), ui_font(11.5), pal().dim);
    }
}

#[allow(clippy::too_many_arguments)]
fn icon_cell(app: &mut KernshipApp, ui: &mut Ui, s: usize, gi: usize, r: Rect, size: f32, ppp: f32, now: f64) {
    let set = &app.icon_sets[s];
    let set_id = set.id.clone();
    let g = set.glyphs[gi].clone();
    let path = set.path.clone();
    let index = set.index;
    let key = (set_id.clone(), g.ch);
    let selected = app.icon_selected.contains(&key);
    let resp = ui.interact(r, ui.id().with(("icon", &set_id, g.ch as u32)), Sense::click_and_drag());
    let p = ui.painter().clone();
    let ht = fade(ui, ("icon-h", &set_id, g.ch as u32), resp.hovered());
    let st = fade(ui, ("icon-s", &set_id, g.ch as u32), selected);
    if ht > 0.0 {
        p.rect_filled(r, CARD_RADIUS, pal().panel.gamma_multiply(ht * (1.0 - st)));
    }
    if st > 0.0 {
        p.rect_stroke(r, CARD_RADIUS, Stroke::new(2.5 * st, pal().ink), StrokeKind::Inside);
    }
    let icon_area = Rect::from_min_size(r.min + vec2(0.0, 8.0), vec2(r.width(), size + 18.0));
    if let Some(path) = path {
        let rk = RenderKey { font_id: set_id.clone(), text: g.ch.to_string(), px: (size * ppp).round() as u32, max_w: (r.width() * ppp) as u32, spacing: 0 };
        if let Some(Some((tex, sz))) = app.renderer.get(&rk, move || Some((path, index))) {
            let s2 = sz / ppp;
            p.image(tex.id(), Rect::from_center_size(icon_area.center(), s2), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), pal().ink);
        }
    }
    let galley = ui.fonts(|f| {
        let mut job = egui::text::LayoutJob::simple_singleline(g.name.clone(), ui_font(11.0), pal().dim);
        job.wrap = egui::text::TextWrapping::truncate_at_width(r.width() - 10.0);
        f.layout_job(job)
    });
    p.galley(pos2(r.center().x - galley.size().x / 2.0, r.bottom() - 20.0), galley, pal().dim);

    if resp.clicked() {
        let m = ui.input(|i| i.modifiers);
        if m.command {
            if let Some(i) = app.icon_selected.iter().position(|x| *x == key) {
                app.icon_selected.remove(i);
            } else {
                app.icon_selected.push(key.clone());
            }
        } else if m.shift && !app.icon_selected.is_empty() {
            let glyphs: Vec<char> = app.icon_sets[s].glyphs.iter().map(|g| g.ch).collect();
            let a = app.icon_selected.iter().rev().find(|(sid, _)| *sid == set_id).and_then(|(_, c)| glyphs.iter().position(|x| x == c));
            if let Some(a) = a {
                let (lo, hi) = (a.min(gi), a.max(gi));
                app.icon_selected = glyphs[lo..=hi].iter().map(|c| (set_id.clone(), *c)).collect();
            }
        } else {
            app.icon_selected = vec![key.clone()];
        }
    }
    if resp.double_clicked() {
        app.icon_selected = vec![key.clone()];
        app.copy_selected_svg(now);
    }
    if resp.secondary_clicked() && !selected {
        app.icon_selected = vec![key.clone()];
    }
    if resp.drag_started() {
        if !selected {
            app.icon_selected = vec![key.clone()];
        }
        // Drag the icons out as SVG files.
        let dir = crate::macos::drag_dir();
        let files = app.export_selected_svgs(&dir, 0.0);
        let pos = resp.interact_pointer_pos().unwrap_or(r.center());
        let ok = crate::macos::drag_files(app.ns_view as *mut std::ffi::c_void, &files, (pos.x as f64, pos.y as f64));
        if !ok {
            app.copy_selected_svg(now);
        }
    }
    resp.context_menu(|ui| {
        menu_style(ui);
        let n = app.icon_selected.len();
        if menu_item(ui, if n > 1 { "Copy SVG of First Icon" } else { "Copy SVG" }, "⌘C", true, false) {
            app.copy_selected_svg(now);
            ui.close_menu();
        }
        if menu_item(ui, &if n > 1 { format!("Save {n} Icons as SVG…") } else { "Save as SVG…".into() }, "", true, false) {
            if let Some(dir) = crate::ui_main::pick_folder() {
                app.export_selected_svgs(&dir, now);
            }
            ui.close_menu();
        }
        menu_separator(ui);
        if menu_item(ui, "Copy Name", "", true, false) {
            ui.ctx().copy_text(g.name.clone());
            ui.close_menu();
        }
        if menu_item(ui, "Copy Character", "", true, false) {
            ui.ctx().copy_text(g.ch.to_string());
            ui.close_menu();
        }
        if menu_item(ui, "Copy Unicode", "", true, false) {
            ui.ctx().copy_text(format!("U+{:04X}", g.ch as u32));
            ui.close_menu();
        }
    });
}
