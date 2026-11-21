//! The panel under the search field: it opens when the field
//! gets focus and holds the quick filters (Starred, Variable, Activated, Deactivated, Color,
//! Duplicate), the Category chips and the Properties row (weight, width, x-height, figures,
//! contrast, language), with a gear menu and a pin that keeps it open.
use crate::app::KernshipApp;
use crate::model::Category;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, CornerRadius, Painter, Pos2, Rect, Sense, Stroke, Ui};

const W: f32 = 304.0;
const PAD: f32 = 12.0;
const CHIP_H: f32 = 26.0;
const GAP: f32 = 8.0;

fn chip_fill(on: bool, hover: f32) -> Color32 {
    if on {
        pal().ink
    } else {
        mix(pal().panel_hi, pal().sel_fill, hover)
    }
}

/// Small icons drawn for the chips.
fn icon(p: &Painter, c: Pos2, kind: &str, col: Color32) {
    let s = Stroke::new(1.2_f32, col);
    match kind {
        "variable" => {
            p.text(c + vec2(0.0, 0.5), Align2::CENTER_CENTER, "V", ui_bold(13.0), col);
            p.line_segment([c + vec2(-5.5, -6.5), c + vec2(5.5, -6.5)], s);
            p.line_segment([c + vec2(-5.5, 7.0), c + vec2(5.5, 7.0)], s);
        }
        "activated" | "deactivated" => {
            p.circle_stroke(c + vec2(0.0, -2.0), 4.2, s);
            p.line_segment([c + vec2(-2.0, 3.6), c + vec2(2.0, 3.6)], s);
            p.line_segment([c + vec2(-1.6, 5.6), c + vec2(1.6, 5.6)], s);
            if kind == "activated" {
                for a in [-2.4f32, -1.57, -0.74] {
                    let d = vec2(a.cos(), a.sin());
                    p.line_segment([c + vec2(0.0, -2.0) + d * 6.0, c + vec2(0.0, -2.0) + d * 7.6], s);
                }
            } else {
                p.line_segment([c + vec2(-5.5, -7.0), c + vec2(5.5, 6.0)], s);
            }
        }
        "duplicate" => paint_menu_icon_painter(p, c, col),
        "color" => {
            // Layers of colour: three overlapping rings.
            p.circle_stroke(c + vec2(0.0, -2.6), 3.6, s);
            p.circle_stroke(c + vec2(-3.0, 2.4), 3.6, s);
            p.circle_stroke(c + vec2(3.0, 2.4), 3.6, s);
        }
        "globe" => {
            p.circle_stroke(c, 6.0, s);
            p.add(egui::Shape::ellipse_stroke(c, vec2(2.6, 6.0), s));
            p.line_segment([c + vec2(-6.0, 0.0), c + vec2(6.0, 0.0)], s);
        }
        _ => {}
    }
}

fn paint_menu_icon_painter(p: &Painter, c: Pos2, col: Color32) {
    let s = Stroke::new(1.2_f32, col);
    let back = Rect::from_center_size(c + vec2(-1.5, -1.5), vec2(8.0, 9.5));
    let front = Rect::from_center_size(c + vec2(1.5, 1.5), vec2(8.0, 9.5));
    p.rect_stroke(back, 1.5, s, egui::StrokeKind::Middle);
    p.rect_stroke(front, 1.5, s, egui::StrokeKind::Middle);
}

/// One chip: icon (drawn by `draw`) and label; returns whether it was clicked.
fn chip(ui: &mut Ui, id: &str, label: &str, on: bool, tip: &str, draw: impl FnOnce(&Ui, Pos2, Color32)) -> egui::Response {
    let text_w = ui.fonts(|f| f.layout_no_wrap(label.to_string(), ui_font(13.0), pal().ground).size().x);
    let has_icon = true;
    let w = 9.0 + if has_icon { 18.0 + 4.0 } else { 0.0 } + text_w + 10.0;
    let (rect, r) = ui.allocate_exact_size(vec2(w, CHIP_H), Sense::click());
    let r = if tip.is_empty() { r } else { r.on_hover_text(tip) };
    focus_ring(ui, &r, rect, 4.0);
    let hv = fade(ui, ("sp-chip", id), r.hovered());
    let st = fade(ui, ("sp-chip-on", id), on);
    ui.painter().rect_filled(rect, 4.0, mix(chip_fill(false, hv), pal().ink, st));
    let col = mix(pal().ink, pal().ground, st);
    let mut x = rect.left() + 9.0;
    if has_icon {
        draw(ui, pos2(x + 9.0, rect.center().y), col);
        x += 22.0;
    }
    {
        ui.painter().text(pos2(x, rect.center().y), Align2::LEFT_CENTER, label, ui_medium(13.0), col);
    }
    r
}

/// A joined group of icon cells (the Properties row); returns the clicked cell.
fn segments(ui: &mut Ui, id: &str, on: &[bool], tips: &[&str], cell_w: f32, draw: impl Fn(&Painter, usize, Pos2, Color32)) -> Option<usize> {
    let n = on.len();
    let (rect, _) = ui.allocate_exact_size(vec2(cell_w * n as f32, CHIP_H), Sense::hover());
    let p = ui.painter().clone();
    p.rect_filled(rect, 4.0, chip_fill(false, 0.0));
    let mut clicked = None;
    for i in 0..n {
        let cell = Rect::from_min_size(pos2(rect.left() + cell_w * i as f32, rect.top()), vec2(cell_w, CHIP_H));
        let r = ui.interact(cell, ui.id().with((id, i)), Sense::click()).on_hover_text(tips[i]);
        let hv = fade(ui, ("sp-seg", id, i), r.hovered());
        let st = fade(ui, ("sp-seg-on", id, i), on[i]);
        let cr = CornerRadius {
            nw: if i == 0 { 4 } else { 0 },
            sw: if i == 0 { 4 } else { 0 },
            ne: if i + 1 == n { 4 } else { 0 },
            se: if i + 1 == n { 4 } else { 0 },
        };
        p.rect_filled(cell, cr, mix(chip_fill(false, hv), pal().ink, st));
        if i > 0 && st < 0.5 && !on[i - 1] {
            p.line_segment([pos2(cell.left(), cell.top() + 5.0), pos2(cell.left(), cell.bottom() - 5.0)], Stroke::new(1.0_f32, pal().rule_strong));
        }
        draw(&p, i, cell.center(), mix(pal().ink, pal().ground, st));
        if r.clicked() {
            clicked = Some(i);
        }
    }
    clicked
}

/// "A" drawn with strokes of the given width (Light, Medium, Bold).
fn letter_a(p: &Painter, c: Pos2, w: f32, col: Color32) {
    let s = Stroke::new(w, col);
    let (top, l, r) = (c + vec2(0.0, -6.5), c + vec2(-5.2, 6.5), c + vec2(5.2, 6.5));
    p.line_segment([l, top], s);
    p.line_segment([top, r], s);
    p.line_segment([c + vec2(-3.0, 2.0), c + vec2(3.0, 2.0)], Stroke::new(w * 0.85, col));
}

/// "M" in a narrow, normal or wide width.
fn letter_m(p: &Painter, c: Pos2, half: f32, col: Color32) {
    let pts = vec![c + vec2(-half, 6.5), c + vec2(-half, -6.5), c + vec2(0.0, 3.0), c + vec2(half, -6.5), c + vec2(half, 6.5)];
    p.add(egui::Shape::line(pts, Stroke::new(1.4_f32, col)));
}

/// "h" with a low, regular or high x-height (`xh` above the baseline, of 13).
fn letter_h(p: &Painter, c: Pos2, xh: f32, col: Color32) {
    let s = Stroke::new(1.4_f32, col);
    let base = c.y + 6.5;
    let (l, r) = (c.x - 4.0, c.x + 4.0);
    p.line_segment([pos2(l, c.y - 6.5), pos2(l, base)], s);
    let top = base - xh;
    let arc: Vec<Pos2> = (0..=8)
        .map(|i| {
            let a = std::f32::consts::PI * (1.0 - i as f32 / 8.0);
            pos2(c.x + 4.0 * a.cos(), top + 2.6 - 2.6 * a.sin())
        })
        .collect();
    p.add(egui::Shape::line(arc, s));
    p.line_segment([pos2(r, top + 2.6), pos2(r, base)], s);
}

/// "o" with high, regular or low stroke contrast.
fn letter_o(p: &Painter, c: Pos2, inner: egui::Vec2, col: Color32, bg: Color32) {
    p.add(egui::Shape::ellipse_filled(c, vec2(5.2, 6.4), col));
    p.add(egui::Shape::ellipse_filled(c, inner, bg));
}

pub fn search_popover(app: &mut KernshipApp, ui: &mut Ui, search: Rect, now: f64) {
    let ctx = ui.ctx().clone();
    let t = ctx.animate_bool_with_time(egui::Id::new("search-pop"), app.search_pop, ANIM);
    if t <= 0.0 {
        return;
    }
    let screen = ctx.screen_rect();
    let x = (search.left() + 3.0).min(screen.right() - W - 8.0);
    let pos = pos2(x, search.bottom() + 7.0 - (1.0 - t) * 6.0);
    let before = (app.quick, app.category_filter.clone(), app.props_filter);
    let mut refocus = false;
    let area = egui::Area::new(egui::Id::new("search-pop-area")).order(egui::Order::Foreground).fixed_pos(pos).show(&ctx, |ui| {
        ui.set_opacity(t);
        egui::Frame::new()
            .fill(pal().menu)
            .stroke(Stroke::new(1.0_f32, pal().rule_strong))
            .corner_radius(8.0)
            .shadow(egui::epaint::Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(90) })
            .inner_margin(PAD)
            .show(ui, |ui| {
                ui.set_width(W - PAD * 2.0);
                ui.spacing_mut().item_spacing = vec2(GAP, GAP);

                // Quick filters (SearchSmartTag*).
                ui.horizontal_wrapped(|ui| {
                    let q = &mut app.quick;
                    let mut toggle = |ui: &mut Ui, id: &str, label: &str, v: &mut bool, tip: &str| {
                        let on = *v;
                        let kind = id.to_string();
                        if chip(ui, id, label, on, tip, |ui, c, col| {
                            if kind == "starred" {
                                paint_menu_icon(ui, c, "Starred", col);
                            } else {
                                icon(ui.painter(), c, &kind, col);
                            }
                        })
                        .clicked()
                        {
                            *v = !on;
                            refocus = true;
                        }
                    };
                    toggle(ui, "starred", "Starred", &mut q.starred, "");
                    toggle(ui, "variable", "Variable", &mut q.variable, "");
                    toggle(ui, "activated", "Activated", &mut q.activated, "");
                    toggle(ui, "deactivated", "Deactivated", &mut q.deactivated, "");
                    toggle(ui, "color", "Color", &mut q.color, "Fonts with colour glyphs");
                    toggle(ui, "duplicate", "Duplicate", &mut q.duplicate, "Fonts with the same name");
                });

                ui.add_space(2.0);
                ui.label(egui::RichText::new("Category").font(ui_bold(12.0)).color(pal().dim));
                ui.horizontal_wrapped(|ui| {
                    for c in Category::CHIPS {
                        let label = match c {
                            Category::SansSerif => "Sans Serif",
                            Category::SlabSerif => "Slab",
                            Category::Monospace => "Mono",
                            _ => c.chip(),
                        };
                        let on = app.category_filter.contains(&c);
                        if chip(ui, &format!("cat-{}", c.code()), label, on, "", |ui, p, col| paint_menu_icon(ui, p, c.label(), col)).clicked() {
                            if on {
                                app.category_filter.remove(&c);
                            } else {
                                app.category_filter.insert(c);
                            }
                            refocus = true;
                        }
                    }
                });

                ui.add_space(2.0);
                ui.label(egui::RichText::new("Properties").font(ui_bold(12.0)).color(pal().dim));
                let bg_on = |on: bool| if on { pal().ink } else { chip_fill(false, 0.0) };
                ui.horizontal_wrapped(|ui| {
                    let pf = &mut app.props_filter;
                    let flip = |v: &mut [bool], i: Option<usize>| {
                        if let Some(i) = i {
                            v[i] = !v[i];
                            true
                        } else {
                            false
                        }
                    };
                    let w = pf.weight;
                    refocus |= flip(&mut pf.weight, segments(ui, "weight", &w, &["Light Weight", "Medium Weight", "Bold Weight"], 26.0, |p, i, c, col| letter_a(p, c, [0.9, 1.6, 2.6][i], col)));
                    let w = pf.width;
                    refocus |= flip(&mut pf.width, segments(ui, "width", &w, &["Narrow Width", "Normal Width", "Expanded Width"], 26.0, |p, i, c, col| letter_m(p, c, [3.0, 5.0, 7.5][i], col)));
                    let w = pf.xheight;
                    refocus |= flip(&mut pf.xheight, segments(ui, "xheight", &w, &["Low x-Height", "Regular x-Height", "High x-Height"], 26.0, |p, i, c, col| letter_h(p, c, [4.5, 7.0, 9.5][i], col)));
                    let w = pf.figures;
                    refocus |= flip(&mut pf.figures, segments(ui, "figures", &w, &["Old Style Numbers", "Lining Numbers"], 36.0, |p, i, c, col| {
                        let fam = if i == 0 { "icon-oldstyle" } else { "icon-lining" };
                        p.text(c + vec2(0.0, 0.5), Align2::CENTER_CENTER, "296", egui::FontId::new(15.0, egui::FontFamily::Name(fam.into())), col);
                    }));
                    let w = pf.contrast;
                    refocus |= flip(&mut pf.contrast, segments(ui, "contrast", &w, &["High Contrast", "Regular Contrast", "Low Contrast"], 26.0, |p, i, c, col| {
                        let inner = [vec2(2.3, 5.6), vec2(3.2, 5.0), vec2(3.9, 5.1)][i];
                        letter_o(p, c, inner, col, bg_on(w[i]));
                    }));
                    // Language: a menu of languages.
                    let lang = pf.language;
                    let label = lang.map(|l| crate::props::LANGUAGES[l].1).unwrap_or("");
                    let resp = chip(ui, "language", label, lang.is_some(), "Language", |ui, c, col| icon(ui.painter(), c, "globe", col));
                    crate::ui_main::click_menu(ui, &resp, "sp-language", |ui| {
                        crate::theme::menu_layout(ui, true, false);
                        if crate::theme::menu_item(ui, "All Languages", "", true, lang.is_none()) {
                            app.props_filter.language = None;
                        }
                        crate::theme::menu_separator(ui);
                        egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                            for (i, l) in crate::props::LANGUAGES.iter().enumerate() {
                                if crate::theme::menu_item(ui, l.1, "", true, lang == Some(i)) {
                                    app.props_filter.language = Some(i);
                                }
                            }
                        });
                    });
                });

                // Bottom row: Clear on the left; gear (Search by / Sort By) and pin on the right.
                let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
                let p = ui.painter().clone();
                if app.filters_on() {
                    let cr = Rect::from_min_size(row.left_top(), vec2(44.0, 18.0));
                    let r = ui.interact(cr, ui.id().with("sp-clear"), Sense::click());
                    let col = if r.hovered() { pal().ink } else { pal().dim };
                    p.text(cr.left_center(), Align2::LEFT_CENTER, "Clear", ui_font(12.0), col);
                    if r.clicked() {
                        app.clear_filters();
                        refocus = true;
                    }
                }
                let pin_c = pos2(row.right() - 8.0, row.center().y);
                let gear_c = pos2(row.right() - 36.0, row.center().y);
                let gr = ui.interact(Rect::from_center_size(gear_c, vec2(20.0, 20.0)), ui.id().with("sp-gear"), Sense::click()).on_hover_text("Search Options");
                let gcol = if gr.hovered() { pal().ink } else { pal().dim };
                p.circle_stroke(gear_c, 4.6, Stroke::new(1.3_f32, gcol));
                p.circle_stroke(gear_c, 1.6, Stroke::new(1.2_f32, gcol));
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::FRAC_PI_4;
                    let d = vec2(a.cos(), a.sin());
                    p.line_segment([gear_c + d * 4.6, gear_c + d * 6.8], Stroke::new(2.0_f32, gcol));
                }
                crate::ui_main::click_menu(ui, &gr, "sp-gear-menu", |ui| crate::ui_main::filter_menu(app, ui));
                let pr = ui.interact(Rect::from_center_size(pin_c, vec2(20.0, 20.0)), ui.id().with("sp-pin"), Sense::click()).on_hover_text(if app.search_pinned { "Unpin" } else { "Keep Open" });
                let pcol = if app.search_pinned || pr.hovered() { pal().ink } else { pal().dim };
                let head = Rect::from_center_size(pin_c + vec2(0.0, -3.0), vec2(5.0, 6.5));
                if app.search_pinned {
                    p.rect_filled(head, 1.0, pcol);
                } else {
                    p.rect_stroke(head, 1.0, Stroke::new(1.2_f32, pcol), egui::StrokeKind::Middle);
                }
                p.line_segment([pin_c + vec2(-4.5, 0.6), pin_c + vec2(4.5, 0.6)], Stroke::new(1.3_f32, pcol));
                p.line_segment([pin_c + vec2(0.0, 0.6), pin_c + vec2(0.0, 7.0)], Stroke::new(1.2_f32, pcol));
                if pr.clicked() {
                    app.search_pinned = !app.search_pinned;
                }
            });
    });
    let changed = before != (app.quick, app.category_filter.clone(), app.props_filter);
    if changed {
        app.dirty = true;
        if app.props_filter.needs_measuring() || app.quick.color {
            app.props_queued = 0; // measure anything new now
        }
    }
    if refocus {
        app.focus_search = true;
    }
    // Closes on a click outside the panel and the field, or Escape, unless pinned.
    let rect = area.response.rect;
    if app.search_pop && !app.search_pinned {
        let outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().map(|p| !rect.contains(p) && !search.contains(p)).unwrap_or(false));
        let popup_open = ctx.memory(|m| m.any_popup_open());
        if (outside && !popup_open) || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            app.search_pop = false;
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                ctx.memory_mut(|m| m.surrender_focus(ui.id()));
            }
        }
    }
    let _ = now;
}
