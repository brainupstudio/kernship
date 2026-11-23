//! Info pane (inspector), details window, settings and the small dialogs.
use crate::app::{Dialog, RemoveWhat, RenameWhat, KernshipApp};
use crate::model::*;
use crate::render::RenderKey;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Rect, RichText, Sense, Ui};

const TABS: [&str; 5] = ["Preview", "Sizes", "Glyphs", "Features", "Info"];
const QUICK_LOOK: &str = "ABCDEFGHIJKLM\nNOPQRSTUVWXYZ\nabcdefghijklm\nnopqrstuvwxyz\n1234567890";

/// One line of the font's own lettering at `size` pt, drawn into the UI.
fn sample_line(app: &mut KernshipApp, ui: &mut Ui, f: &FontFace, text: &str, size: f32, width: f32) -> f32 {
    let ppp = ui.ctx().pixels_per_point();
    let key = RenderKey {
        font_id: f.id.clone(),
        text: text.to_string(),
        px: (size * ppp).round() as u32,
        max_w: (width * ppp) as u32,
        spacing: 0,
    };
    let loc = app.locator(f);
    let h = size * 1.35;
    let (rect, _) = ui.allocate_exact_size(vec2(width, h), Sense::hover());
    match app.renderer.get(&key, loc) {
        Some(Some((tex, sz))) => {
            let r = Rect::from_min_size(rect.min, sz / ppp);
            ui.painter().with_clip_rect(rect).image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), pal().ink);
        }
        Some(None) => {
            ui.painter().text(rect.left_center(), Align2::LEFT_CENTER, "Preview unavailable", ui_font(12.0), pal().faint);
        }
        None => {
            ui.painter().rect_filled(Rect::from_min_size(rect.min + vec2(0.0, h * 0.25), vec2(width * 0.5, h * 0.4)), 4.0, pal().panel_hi);
        }
    }
    h
}

fn info_rows(app: &KernshipApp, f: &FontFace) -> Vec<(&'static str, String)> {
    let sets: Vec<String> = app
        .st
        .libraries
        .iter()
        .flat_map(|l| l.collections.iter().filter(|c| c.font_ids.contains(&f.id)).map(move |c| format!("{} › {}", l.name, c.name)))
        .collect();
    let tags: Vec<String> = app.st.tags.iter().filter(|t| t.font_ids.contains(&f.id)).map(|t| t.name.clone()).collect();
    let status = match (app.is_active(f), app.activated_by(&f.id)) {
        (false, _) => "Deactivated",
        (true, Some(ActivatedBy::Temporary)) => "Temporarily Activated",
        (true, Some(ActivatedBy::Auto)) => "Auto Activated",
        (true, _) => "Activated",
    };
    vec![
        ("Name", f.display_name()),
        ("Family", f.family.clone()),
        ("Style", f.style.clone()),
        ("Status", status.into()),
        ("Format", if f.path.is_some() { f.format_name().into() } else { "Google Fonts (not downloaded)".into() }),
        ("PostScript Name", f.ps_name.clone()),
        ("Unique Name", f.full_name.clone()),
        ("Location", f.path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "—".into())),
        ("Font Sets", if sets.is_empty() { "—".into() } else { sets.join(", ") }),
        ("Variable Font", if f.variable { "Yes".into() } else { "No".into() }),
        ("Category", f.category.label().into()),
        ("Weight", format!("{} ({})", f.weight, weight_name(f.weight))),
        ("Version", f.version.clone()),
        ("Glyph Count", if f.glyph_count > 0 { f.glyph_count.to_string() } else { "—".into() }),
        ("Tags", if tags.is_empty() { "—".into() } else { tags.join(", ") }),
        ("License", f.license.clone()),
        ("Copyright", f.copyright.clone()),
        ("Designer", f.designer.clone()),
        ("Manufacturer", f.manufacturer.clone()),
    ]
}

fn glyph_grid(app: &mut KernshipApp, ui: &mut Ui, f: &FontFace, cell: f32, limit: usize) {
    let Some((path, idx)) = f.path.clone().map(|p| (p, f.index)).or_else(|| {
        let p = crate::store::google_cache_dir().join(crate::google::file_name(&f.family, f.gf_variant.as_deref().unwrap_or("400")));
        p.exists().then_some((p, 0))
    }) else {
        ui.label(RichText::new("Glyphs show once the font is downloaded or previewed.").color(pal().dim));
        return;
    };
    let Some(font) = app.renderer.font(&path, idx) else { return };
    let chars = crate::render::codepoints(&font, limit);
    let ppp = ui.ctx().pixels_per_point();
    let cols = ((ui.available_width() + 5.0) / (cell + 5.0)).floor().max(1.0) as usize;
    egui::Grid::new(("glyphs", &f.id)).spacing(vec2(5.0, 5.0)).show(ui, |ui| {
        for (i, ch) in chars.iter().enumerate() {
            let (rect, resp) = ui.allocate_exact_size(vec2(cell, cell), Sense::click());
            ui.painter().rect_filled(rect, 6.0, pal().panel);
            let key = RenderKey { font_id: f.id.clone(), text: ch.to_string(), px: (cell * 0.55 * ppp) as u32, max_w: (cell * ppp) as u32, spacing: 0 };
            let p2 = path.clone();
            if let Some(Some((tex, sz))) = app.renderer.get(&key, move || Some((p2, idx))) {
                let s = sz / ppp;
                ui.painter().image(tex.id(), Rect::from_center_size(rect.center() - vec2(0.0, 4.0), s), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), pal().ink);
            }
            ui.painter().text(rect.center_bottom() - vec2(0.0, 7.0), Align2::CENTER_CENTER, format!("{:04X}", *ch as u32), ui_font(8.5), pal().faint);
            if resp.on_hover_text(format!("U+{:04X} — click to copy", *ch as u32)).clicked() {
                ui.ctx().copy_text(ch.to_string());
            }
            if (i + 1) % cols == 0 {
                ui.end_row();
            }
        }
    });
}

fn features(app: &KernshipApp, f: &FontFace) -> Vec<String> {
    let Some(path) = &f.path else { return vec![] };
    let _ = app;
    let Ok(data) = std::fs::read(path) else { return vec![] };
    let Ok(face) = ttf_parser::Face::parse(&data, f.index) else { return vec![] };
    let mut out: Vec<String> = Vec::new();
    for table in [face.tables().gsub, face.tables().gpos].into_iter().flatten() {
        for feat in table.features {
            let t = feat.tag.to_string();
            if !out.contains(&t) {
                out.push(t);
            }
        }
    }
    out.sort();
    out
}

fn feature_name(tag: &str) -> &'static str {
    match tag {
        "liga" => "Standard Ligatures",
        "dlig" => "Discretionary Ligatures",
        "kern" => "Kerning",
        "smcp" => "Small Capitals",
        "c2sc" => "Capitals to Small Capitals",
        "onum" => "Oldstyle Figures",
        "lnum" => "Lining Figures",
        "tnum" => "Tabular Figures",
        "pnum" => "Proportional Figures",
        "frac" => "Fractions",
        "sups" => "Superscript",
        "subs" => "Subscript",
        "ordn" => "Ordinals",
        "case" => "Case-Sensitive Forms",
        "calt" => "Contextual Alternates",
        "salt" => "Stylistic Alternates",
        "swsh" => "Swash",
        "zero" => "Slashed Zero",
        "locl" => "Localized Forms",
        "mark" => "Mark Positioning",
        "mkmk" => "Mark to Mark Positioning",
        "ccmp" => "Glyph Composition",
        t if t.starts_with("ss") => "Stylistic Set",
        t if t.starts_with("cv") => "Character Variant",
        _ => "",
    }
}

fn inspector_body(app: &mut KernshipApp, ui: &mut Ui, f: &FontFace, width: f32, now: f64) {
    match app.inspector_tab {
        0 => {
            let text = if app.st.preview_text.trim().is_empty() { QUICK_LOOK.to_string() } else { app.st.preview_text.clone() };
            for line in text.lines() {
                sample_line(app, ui, f, line, 26.0, width);
            }
        }
        1 => {
            let text = if app.st.preview_text.trim().is_empty() { "The quick brown fox".to_string() } else { app.st.preview_text.clone() };
            for size in [9.0, 10.0, 12.0, 14.0, 18.0, 24.0, 30.0, 36.0, 48.0, 60.0, 72.0] {
                ui.label(RichText::new(format!("{size} pt")).small().color(pal().faint));
                sample_line(app, ui, f, &text, size, width);
            }
        }
        2 => glyph_grid(app, ui, f, 52.0, 600),
        3 => {
            let feats = features(app, f);
            if feats.is_empty() {
                ui.label(RichText::new("No OpenType features found.").color(pal().dim));
            }
            for t in feats {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&t).monospace().color(pal().ink));
                    ui.label(RichText::new(feature_name(&t)).color(pal().dim));
                });
            }
        }
        _ => {
            for (k, v) in info_rows(app, f) {
                if v.is_empty() {
                    continue;
                }
                ui.label(RichText::new(k).small().color(pal().dim));
                ui.add(egui::Label::new(RichText::new(v).color(pal().ink)).wrap());
                ui.add_space(4.0);
            }
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                let active = app.is_active(f);
                if f.source != Source::System
                    && button(ui, if active { "Deactivate" } else { "Activate" }, Btn::Primary) {
                        app.set_active(std::slice::from_ref(&f.id), !active, now);
                    }
                if f.path.is_some() && button(ui, "Show in Finder", Btn::Plain) {
                    app.reveal(&f.id);
                }
                if f.is_google() && button(ui, "Google Fonts page", Btn::Plain) {
                    let url = format!("https://fonts.google.com/specimen/{}", f.family.replace(' ', "+"));
                    let _ = std::process::Command::new("open").arg(url).spawn();
                }
            });
        }
    }
}

fn tab_bar(app: &mut KernshipApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        // The five tabs share the pane's width: what is left over goes between them.
        let total: f32 = TABS.iter().map(|t| ui.fonts(|f| f.layout_no_wrap(t.to_string(), ui_medium(12.5), pal().ink).size().x) + 16.0).sum();
        ui.spacing_mut().item_spacing.x = ((ui.available_width() - total) / (TABS.len() as f32 - 1.0)).clamp(0.0, 12.0);
        for (i, t) in TABS.iter().enumerate() {
            if toggle(ui, t, app.inspector_tab == i as u8).clicked() {
                app.inspector_tab = i as u8;
            }
        }
    });
}

/// The Info pane on the right (⌘I).
pub fn info_panel(app: &mut KernshipApp, ui: &mut Ui, panel: Rect, now: f64) {
    ui.painter().rect_filled(panel, 0.0, pal().panel);
    ui.painter().line_segment([panel.left_top(), panel.left_bottom()], egui::Stroke::new(1.0_f32, pal().rule));
    let inner = panel.shrink2(vec2(18.0, 16.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    child.set_clip_rect(panel);
    let ui = &mut child;
    if app.st.info_mode == crate::store::InfoMode::Web {
        crate::ui_webfont::web_panel(app, ui, now);
        return;
    }
    let Some(f) = app.selected.first().and_then(|i| app.font(i).cloned()) else {
        ui.add_space(inner.height() / 2.0 - 20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No Selection").size(15.0).color(pal().dim));
            ui.label(RichText::new("Select a font to see its details.").color(pal().faint));
        });
        return;
    };
    ui.label(RichText::new(&f.family).font(ui_bold(16.0)).color(pal().ink));
    ui.label(RichText::new(&f.style).color(pal().dim));
    if app.selected.len() > 1 {
        ui.label(RichText::new(format!("{} fonts selected", app.selected.len())).small().color(pal().faint));
    }
    ui.add_space(6.0);
    tab_bar(app, ui);
    ui.separator();
    egui::ScrollArea::vertical().id_salt("inspector").auto_shrink([false, false]).show(ui, |ui| {
        inspector_body(app, ui, &f, inner.width(), now);
    });
}

fn modal(ctx: &egui::Context, title: &str, width: f32, add: impl FnOnce(&mut Ui)) -> bool {
    let mut open = true;
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, -40.0))
        .fixed_size(vec2(width, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_width(width);
            add(ui)
        });
    open
}

pub fn dialogs(app: &mut KernshipApp, ctx: &egui::Context, now: f64) {
    let mut d = app.dialog.clone();
    let mut close = false;
    let open = match &mut d {
        Dialog::None => return,
        Dialog::NewLibrary { name, folder } => modal(ctx, "New Library", 340.0, |ui| {
            ui.label("Library name:");
            let r = ui.text_edit_singleline(name);
            r.request_focus();
            if let Some(f) = folder {
                ui.label(RichText::new(format!("Fonts from {}", f.display())).small().color(pal().dim));
            }
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 152.0).max(0.0));
                if button(ui, "Cancel", Btn::Plain) {
                    close = true;
                }
                if (button(ui, "Create", Btn::Primary) || ui.input(|i| i.key_pressed(egui::Key::Enter))) && !name.trim().is_empty() {
                    let id = app.create_library(name.trim(), folder.clone(), now);
                    app.set_nav(Nav::Library(id));
                    close = true;
                }
            });
        }),
        Dialog::NewCollection { library, parent, name } => modal(ctx, "New Set", 320.0, |ui| {
            ui.label("Set name:");
            ui.text_edit_singleline(name).request_focus();
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 152.0).max(0.0));
                if button(ui, "Cancel", Btn::Plain) {
                    close = true;
                }
                if (button(ui, "Create", Btn::Primary) || ui.input(|i| i.key_pressed(egui::Key::Enter))) && !name.trim().is_empty() {
                    let lib = library.clone();
                    if let Some(l) = app.st.libraries.iter_mut().find(|l| l.id == lib) {
                        l.collections.push(Collection { id: new_id(), name: name.trim().into(), parent: parent.clone(), font_ids: app.selected.clone() });
                    }
                    app.expanded.insert(lib.clone());
                    if let Some(p) = parent {
                        app.expanded.insert(format!("{lib}/{p}"));
                    }
                    app.save_soon(now);
                    app.dirty = true;
                    close = true;
                }
            });
        }),
        Dialog::NewTag { name } => modal(ctx, "New Tag", 300.0, |ui| {
            ui.label("Tag name:");
            ui.text_edit_singleline(name).request_focus();
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 152.0).max(0.0));
                if button(ui, "Cancel", Btn::Plain) {
                    close = true;
                }
                if (button(ui, "Create", Btn::Primary) || ui.input(|i| i.key_pressed(egui::Key::Enter))) && !name.trim().is_empty() {
                    let ids = app.selected.clone();
                    app.st.tags.push(Tag { id: new_id(), name: name.trim().into(), font_ids: ids });
                    app.save_soon(now);
                    close = true;
                }
            });
        }),
        Dialog::Rename { what, name } => modal(ctx, "Rename", 300.0, |ui| {
            ui.text_edit_singleline(name).request_focus();
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 152.0).max(0.0));
                if button(ui, "Cancel", Btn::Plain) {
                    close = true;
                }
                if (button(ui, "Rename", Btn::Primary) || ui.input(|i| i.key_pressed(egui::Key::Enter))) && !name.trim().is_empty() {
                    let n = name.trim().to_string();
                    match what {
                        RenameWhat::Library(id) => {
                            if let Some(l) = app.st.libraries.iter_mut().find(|l| &l.id == id) {
                                l.name = n;
                            }
                        }
                        RenameWhat::Tag(id) => {
                            if let Some(t) = app.st.tags.iter_mut().find(|t| &t.id == id) {
                                t.name = n;
                            }
                        }
                        RenameWhat::Collection(l, c) => {
                            if let Some(col) = app.st.libraries.iter_mut().find(|x| &x.id == l).and_then(|x| x.collections.iter_mut().find(|y| &y.id == c)) {
                                col.name = n;
                            }
                        }
                    }
                    app.save_soon(now);
                    close = true;
                }
            });
        }),
        Dialog::SmartFilter { filter, is_new } => modal(ctx, if *is_new { "New Smart Filter" } else { "Edit Smart Filter" }, 520.0, |ui| {
            ui.horizontal(|ui| {
                ui.label("Smart Filter name:");
                ui.text_edit_singleline(&mut filter.name);
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("sf-all").icon(crate::theme::popup_icon).width(70.0).selected_text(if filter.match_all { "All" } else { "Any" }).show_ui(ui, |ui| {
                        crate::theme::menu_style(ui);
                    ui.selectable_value(&mut filter.match_all, false, "Any");
                    ui.selectable_value(&mut filter.match_all, true, "All");
                });
                ui.label("of the following are true");
            });
            let mut remove = None;
            for (i, r) in filter.rules.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(("sf-f", i)).icon(crate::theme::popup_icon).width(120.0).selected_text(r.field.label()).show_ui(ui, |ui| {
                        crate::theme::menu_style(ui);
                        for f in RuleField::ALL {
                            if ui.selectable_value(&mut r.field, f, f.label()).changed() {
                                r.value = f.choices().map(|c| c[0].to_string()).unwrap_or_default();
                                r.op = RuleOp::Is;
                            }
                        }
                    });
                    let ops: &[RuleOp] = if r.field.choices().is_some() {
                        &[RuleOp::Is, RuleOp::IsNot]
                    } else {
                        &[RuleOp::Contains, RuleOp::BeginsWith, RuleOp::EndsWith, RuleOp::Is, RuleOp::IsNot]
                    };
                    egui::ComboBox::from_id_salt(("sf-o", i)).icon(crate::theme::popup_icon).width(110.0).selected_text(r.op.label()).show_ui(ui, |ui| {
                        crate::theme::menu_style(ui);
                        for o in ops {
                            ui.selectable_value(&mut r.op, *o, o.label());
                        }
                    });
                    if let Some(choices) = r.field.choices() {
                        egui::ComboBox::from_id_salt(("sf-v", i)).icon(crate::theme::popup_icon).width(150.0).selected_text(r.value.clone()).show_ui(ui, |ui| {
                        crate::theme::menu_style(ui);
                            for c in choices {
                                ui.selectable_value(&mut r.value, c.to_string(), c);
                            }
                        });
                    } else {
                        ui.add(egui::TextEdit::singleline(&mut r.value).desired_width(150.0));
                    }
                    if ui.small_button("−").clicked() {
                        remove = Some(i);
                    }
                });
            }
            if let Some(i) = remove {
                filter.rules.remove(i);
            }
            if ui.small_button("+ Add Rule").clicked() {
                filter.rules.push(Rule { field: RuleField::Name, op: RuleOp::Contains, value: String::new() });
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 152.0).max(0.0));
                if button(ui, "Cancel", Btn::Plain) {
                    close = true;
                }
                if button(ui, "OK", Btn::Primary) {
                    if let Some(x) = app.st.smart_filters.iter_mut().find(|x| x.id == filter.id) {
                        *x = filter.clone();
                    } else {
                        app.st.smart_filters.push(filter.clone());
                    }
                    app.expanded.insert("smart".into());
                    app.set_nav(Nav::SmartFilter(filter.id.clone()));
                    app.dirty = true;
                    app.save_soon(now);
                    close = true;
                }
            });
        }),
        Dialog::NewSample { name, text } => modal(ctx, "Preview Text", 380.0, |ui| {
            ui.label("Name");
            ui.add(egui::TextEdit::singleline(name).hint_text("e.g. abc, lorem, symbols"));
            ui.label("Sample preview text");
            ui.add(egui::TextEdit::multiline(text).desired_rows(3));
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 152.0).max(0.0));
                if button(ui, "Cancel", Btn::Plain) {
                    close = true;
                }
                if button(ui, "Save", Btn::Primary) && !name.trim().is_empty() && !text.trim().is_empty() {
                    app.st.samples.push(Sample { name: name.trim().into(), text: text.clone() });
                    app.save_soon(now);
                    close = true;
                }
            });
        }),
        Dialog::ConfirmDelete { ids } => modal(ctx, "Delete from Library", 360.0, |ui| {
            ui.label(format!("Delete {} font(s)? They will be deactivated and their files removed from the library.", ids.len()));
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 152.0).max(0.0));
                if button(ui, "Cancel", Btn::Plain) {
                    close = true;
                }
                if button(ui, "Delete", Btn::Danger) {
                    let ids = ids.clone();
                    app.delete_from_library(&ids, now);
                    close = true;
                }
            });
        }),
        Dialog::ConfirmRemove { what, name } => {
            let (title, message) = match what {
                RemoveWhat::Library(_) => ("Delete Library", format!("Delete the library \u{201c}{name}\u{201d}? Its fonts are deactivated and leave Kernship. The font files stay on disk.")),
                RemoveWhat::Collection(..) => ("Delete Set", format!("Delete the set \u{201c}{name}\u{201d} and the sets inside it? The fonts stay in the library.")),
                RemoveWhat::Tag(_) => ("Delete Tag", format!("Delete the tag \u{201c}{name}\u{201d}? The fonts keep everything else.")),
                RemoveWhat::SmartFilter(_) => ("Delete Smart Filter", format!("Delete the smart filter \u{201c}{name}\u{201d}?")),
            };
            modal(ctx, title, 380.0, |ui| {
                ui.add(egui::Label::new(message).wrap());
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 152.0).max(0.0));
                    if button(ui, "Cancel", Btn::Plain) {
                        close = true;
                    }
                    if button(ui, "Delete", Btn::Danger) {
                        match what {
                            RemoveWhat::Library(id) => {
                                let ids: Vec<String> = app.all.iter().filter(|f| f.source == Source::Library(id.clone())).map(|f| f.id.clone()).collect();
                                app.set_active(&ids, false, now);
                                app.st.library_fonts.retain(|f| f.source != Source::Library(id.clone()));
                                app.st.libraries.retain(|l| &l.id != id);
                                app.set_nav(Nav::AllFonts);
                            }
                            RemoveWhat::Collection(lib, id) => {
                                if let Some(l) = app.st.libraries.iter_mut().find(|l| &l.id == lib) {
                                    l.collections.retain(|x| &x.id != id && x.parent.as_deref() != Some(id.as_str()));
                                }
                            }
                            RemoveWhat::Tag(id) => {
                                app.st.tags.retain(|x| &x.id != id);
                                if app.nav == Nav::Tag(id.clone()) {
                                    app.set_nav(Nav::AllFonts);
                                }
                            }
                            RemoveWhat::SmartFilter(id) => {
                                app.st.smart_filters.retain(|x| &x.id != id);
                                if app.nav == Nav::SmartFilter(id.clone()) {
                                    app.set_nav(Nav::AllFonts);
                                }
                            }
                        }
                        app.dirty = true;
                        app.save_soon(now);
                        close = true;
                    }
                });
            })
        }
        Dialog::MissingFonts { app: dapp, names } => modal(ctx, "Missing Fonts", 400.0, |ui| {
            ui.label(format!(
                "Kernship detected {} missing {} in this document ({}).",
                names.len(),
                if names.len() == 1 { "font" } else { "fonts" },
                dapp
            ));
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for n in names.iter() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(n).monospace());
                        if ui.small_button("Copy Font Name").clicked() {
                            ui.ctx().copy_text(n.clone());
                        }
                    });
                }
            });
            ui.horizontal(|ui| {
                if button(ui, "Copy All Font Names", Btn::Plain) {
                    ui.ctx().copy_text(names.join("\n"));
                }
                if button(ui, "Search Google Fonts", Btn::Plain) {
                    app.search = names.first().map(|n| n.split('-').next().unwrap_or(n).to_string()).unwrap_or_default();
                    app.set_nav(Nav::GoogleFonts);
                    app.dirty = true;
                    close = true;
                }
                if button(ui, "OK", Btn::Primary) {
                    close = true;
                }
            });
        }),
        Dialog::Details { font_id } => {
            let Some(f) = app.font(font_id).cloned() else {
                app.dialog = Dialog::None;
                return;
            };
            let mut open = true;
            egui::Window::new(f.display_name())
                .open(&mut open)
                .default_size(vec2(640.0, 520.0))
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    tab_bar(app, ui);
                    ui.separator();
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        let w = ui.available_width() - 8.0;
                        inspector_body(app, ui, &f, w, now);
                    });
                });
            open
        }
        Dialog::Preferences { tab } => crate::ui_settings::window(app, ctx, tab, now),
        Dialog::Welcome => modal(ctx, "Welcome to Kernship", 420.0, |ui| {
            // The mark in the appearance's own version, then the name and the tagline.
            let light = light_t() > 0.5;
            let key = egui::Id::new(("kernship-mark", light));
            let tex = ui.ctx().memory_mut(|m| m.data.get_temp::<egui::TextureHandle>(key));
            let tex = tex.or_else(|| {
                let t = crate::brand::logo_image(light, 384).map(|img| ui.ctx().load_texture("kernship-mark", img, egui::TextureOptions::LINEAR))?;
                ui.ctx().memory_mut(|m| m.data.insert_temp(key, t.clone()));
                Some(t)
            });
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if let Some(t) = tex {
                    ui.add(egui::Image::new(&t).fit_to_exact_size(vec2(112.0, 112.0)).corner_radius(24.0));
                }
                ui.add_space(10.0);
                ui.label(RichText::new(crate::brand::NAME).font(ui_bold(24.0)).color(pal().ink));
                ui.label(RichText::new(crate::brand::TAGLINE).size(14.0).color(pal().dim));
            });
            ui.add_space(8.0);
            ui.label("Browse every Google Fonts family, preview it in your own text and switch fonts on for Photoshop, Illustrator, InDesign, Sketch, Figma and every other app.");
            ui.add_space(6.0);
            ui.label(RichText::new("• Click the mark of a font (or press ⌘R) to activate it.\n• Google styles download on first activation.\n• Drop font files on the window to import them into a library.\n• Auto Activation turns on the fonts an Adobe document needs.").color(pal().dim));
            ui.add_space(8.0);
            if button(ui, "Get Started", Btn::Primary) {
                app.st.welcomed = true;
                app.save_soon(now);
                close = true;
            }
        }),
    };
    if !open || close {
        if matches!(app.dialog, Dialog::Welcome) {
            app.st.welcomed = true;
            app.save_soon(now);
        }
        app.dialog = Dialog::None;
    } else if app.dialog != Dialog::None && std::mem::discriminant(&app.dialog) == std::mem::discriminant(&d) {
        app.dialog = d;
    }
}
