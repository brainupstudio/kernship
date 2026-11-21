//! Settings window, a centred title, a
//! toolbar of four panes (General, Sidebar, Import, Auto Activation) and the pane below.
use crate::app::{PrefTab, KernshipApp};
use crate::design_apps;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Rect, RichText, Sense, Stroke, StrokeKind, Ui};

const WIDTH: f32 = 560.0;

fn tab_icon(p: &egui::Painter, c: egui::Pos2, tab: PrefTab, col: Color32) {
    let s = Stroke::new(1.6_f32, col);
    match tab {
        PrefTab::General => {
            // Gear: a ring with eight teeth.
            p.circle_stroke(c, 7.5, s);
            p.circle_stroke(c, 3.0, s);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let d = vec2(a.cos(), a.sin());
                p.line_segment([c + d * 7.5, c + d * 10.5], Stroke::new(2.6_f32, col));
            }
        }
        PrefTab::Sidebar => {
            let r = Rect::from_center_size(c, vec2(24.0, 19.0));
            p.rect_stroke(r, 4.0, s, StrokeKind::Middle);
            p.line_segment([pos2(r.left() + 8.0, r.top()), pos2(r.left() + 8.0, r.bottom())], s);
            for k in 0..3 {
                let y = r.top() + 5.0 + k as f32 * 3.5;
                p.line_segment([pos2(r.left() + 2.5, y), pos2(r.left() + 5.5, y)], Stroke::new(1.2_f32, col));
            }
        }
        PrefTab::Import => {
            let b = Rect::from_center_size(c + vec2(0.0, 4.0), vec2(20.0, 12.0));
            p.line_segment([b.left_top() + vec2(5.0, 0.0), b.left_top()], s);
            p.line_segment([b.left_top(), b.left_bottom()], s);
            p.line_segment([b.left_bottom(), b.right_bottom()], s);
            p.line_segment([b.right_bottom(), b.right_top()], s);
            p.line_segment([b.right_top(), b.right_top() - vec2(5.0, 0.0)], s);
            p.line_segment([c + vec2(0.0, -11.0), c + vec2(0.0, 5.0)], s);
            p.line_segment([c + vec2(-4.5, 0.5), c + vec2(0.0, 5.0)], s);
            p.line_segment([c + vec2(4.5, 0.5), c + vec2(0.0, 5.0)], s);
        }
        PrefTab::AutoActivation => {
            // Burst of short rays (the "sparkle" activation symbol).
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0 - std::f32::consts::FRAC_PI_2;
                let d = vec2(a.cos(), a.sin());
                p.line_segment([c + d * 5.0, c + d * 10.0], Stroke::new(1.9_f32, col));
            }
        }
    }
}

fn tab_name(t: PrefTab) -> &'static str {
    match t {
        PrefTab::General => "General",
        PrefTab::Sidebar => "Sidebar",
        PrefTab::Import => "Import",
        PrefTab::AutoActivation => "Auto Activation",
    }
}

fn heading(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).font(ui_bold(14.0)).color(pal().ink));
    ui.add_space(6.0);
}

fn note(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(13.0).color(pal().dim));
}

fn app_badge(app: &mut KernshipApp, ui: &mut Ui, a: &design_apps::AppInfo) {
    let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
    if !app.app_icons.contains_key(a.bundle_id) {
        let tex = crate::macos::app_icon_rgba(a.bundle_id, 64).map(|(px, w, h)| {
            let img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &px);
            ui.ctx().load_texture(format!("appicon-{}", a.bundle_id), img, egui::TextureOptions::LINEAR)
        });
        app.app_icons.insert(a.bundle_id.to_string(), tex);
    }
    match app.app_icons.get(a.bundle_id).and_then(|t| t.clone()) {
        Some(t) => {
            ui.painter().image(t.id(), rect.expand(1.0), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            let bg = Color32::from_rgb(a.bg[0], a.bg[1], a.bg[2]);
            let fg = Color32::from_rgb(a.fg[0], a.fg[1], a.fg[2]);
            ui.painter().rect_filled(rect, 4.5, bg);
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, a.abbr, ui_bold(10.0), fg);
        }
    }
}

/// A macOS segmented control; the selection slides over 200 ms.
fn segmented<T: Copy + PartialEq>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) {
    let seg_w = 118.0;
    let h = 26.0;
    let (rect, _) = ui.allocate_exact_size(vec2(seg_w * options.len() as f32 + 4.0, h + 4.0), Sense::hover());
    let p = ui.painter().clone();
    p.rect_filled(rect, CTRL_RADIUS + 2.0, pal().panel_hi);
    let idx = options.iter().position(|(v, _)| *v == *value).unwrap_or(0) as f32;
    let x = ui.ctx().animate_value_with_time(ui.id().with("seg-x"), idx, ANIM);
    let knob = Rect::from_min_size(pos2(rect.left() + 2.0 + x * seg_w, rect.top() + 2.0), vec2(seg_w, h));
    p.rect_filled(knob, CTRL_RADIUS, pal().ink);
    for (i, (v, label)) in options.iter().enumerate() {
        let r = Rect::from_min_size(pos2(rect.left() + 2.0 + i as f32 * seg_w, rect.top() + 2.0), vec2(seg_w, h));
        let resp = ui.interact(r, ui.id().with(("seg", i)), Sense::click());
        if resp.clicked() {
            *value = *v;
        }
        let on = (i as f32 - x).abs() < 0.5;
        p.text(r.center(), Align2::CENTER_CENTER, *label, ui_medium(13.0), if on { pal().ground } else { pal().ink });
        focus_ring(ui, &resp, r, CTRL_RADIUS);
    }
}

fn general(app: &mut KernshipApp, ui: &mut Ui) {
    let mut login = app.st.prefs.launch_on_login;
    if mac_check(ui, &mut login, "Launch Kernship on startup").changed() {
        app.set_launch_on_login(login);
    }
    ui.add_space(10.0);
    mac_check(ui, &mut app.st.prefs.run_in_background, "Run Kernship in background");
    ui.horizontal(|ui| {
        ui.add_space(28.0);
        note(ui, "Keep app icon in the menu bar when Kernship window is closed.");
    });
    ui.add_space(14.0);
    heading(ui, "Appearance");
    segmented(
        ui,
        &mut app.st.prefs.appearance,
        &[
            (crate::store::Appearance::System, "System Default"),
            (crate::store::Appearance::Light, "Light"),
            (crate::store::Appearance::Dark, "Dark"),
        ],
    );
    ui.add_space(14.0);
    heading(ui, "Libraries");
    let root = crate::store::libraries_root();
    let shown = match dirs::home_dir().and_then(|h| root.strip_prefix(h).ok().map(|p| p.to_path_buf())) {
        Some(rel) => format!("~/{}", rel.display()),
        None => root.display().to_string(),
    };
    note(ui, &format!("Libraries are kept in {shown}"));
    ui.add_space(4.0);
    if button(ui, "Show in Finder", Btn::Plain) {
        let _ = std::process::Command::new("open").arg(crate::store::libraries_root()).spawn();
    }
    ui.add_space(14.0);
    heading(ui, "Google Fonts");
    ui.horizontal(|ui| {
        ui.label(RichText::new("Refresh the catalog").size(14.0));
        let p = &mut app.st.prefs;
        egui::ComboBox::from_id_salt("gf-refresh").icon(crate::theme::popup_icon)
            .width(130.0)
            .selected_text(match p.catalog_refresh_days {
                1 => "Every Day",
                30 => "Every Month",
                _ => "Every Week",
            })
            .show_ui(ui, |ui| {
                menu_style(ui);
                ui.selectable_value(&mut p.catalog_refresh_days, 1, "Every Day");
                ui.selectable_value(&mut p.catalog_refresh_days, 7, "Every Week");
                ui.selectable_value(&mut p.catalog_refresh_days, 30, "Every Month");
            });
        if button(ui, "Sync Now", Btn::Plain) {
            app.start_loading(true);
        }
    });
    note(ui, &format!("{} families, {} styles.", crate::ui_main::thousands(app.catalog.len()), crate::ui_main::thousands(app.all.iter().filter(|f| f.is_google() && !f.icon_font).count())));
    ui.add_space(14.0);
    updates(app, ui);
}

/// Updates: this version, what the last check found, and whether to check at start.
fn updates(app: &mut KernshipApp, ui: &mut Ui) {
    use crate::update::DOWNLOAD_PAGE;
    let available = app.update_available();
    let checking = app.update.checking();
    let downloading = available.is_some() && app.update.downloading() == available.as_deref();
    let ready = available.is_some() && app.update.ready == available;
    let status = if checking {
        "Checking…".to_string()
    } else if let (true, Some(v)) = (ready, &available) {
        format!("Kernship {v} is ready. It installs when you quit Kernship, or now with Restart and Update. Your fonts and libraries stay where they are.")
    } else if let (true, Some(v)) = (downloading, &available) {
        format!("Downloading Kernship {v}… It installs when you quit Kernship.")
    } else if let Some(v) = &available {
        let why = app.update.install_error.as_deref().or(crate::update::why_not_itself());
        match why {
            Some(why) => format!("Kernship {v} is out. {why} Download it and install it over this one; your fonts and libraries stay where they are."),
            None => format!("Kernship {v} is out. Your fonts and libraries stay where they are."),
        }
    } else if let (true, Some(e)) = (app.update.asked, &app.update.error) {
        e.clone()
    } else if app.st.prefs.update_checked > 0 {
        let when = std::process::Command::new("date")
            .args(["-r", &app.st.prefs.update_checked.to_string(), "+%d/%m/%Y %H:%M"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        format!("This is the latest version · checked {when}")
    } else {
        "Not checked yet.".to_string()
    };
    heading(ui, "Updates");
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Kernship {}", crate::update::current())).size(14.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ready {
                if button(ui, "Restart and Update", Btn::Primary) {
                    app.restart_to_update();
                }
            } else if downloading {
                button_w(ui, "Downloading…", Btn::Plain, false, 0.0, 28.0);
            } else if available.is_some() {
                if button(ui, "Download", Btn::Primary) {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(DOWNLOAD_PAGE));
                }
            } else {
                ui.add_enabled_ui(!checking, |ui| {
                    if button(ui, "Check Now", Btn::Plain) {
                        app.update.start(true);
                    }
                });
            }
        });
    });
    note(ui, &status);
    ui.add_space(10.0);
    mac_check(ui, &mut app.st.prefs.update_check, "Check when Kernship opens");
    ui.horizontal(|ui| {
        ui.add_space(28.0);
        note(ui, "Once a day it asks GitHub for the number of the latest version. Nothing about you or your fonts is sent.");
    });
    ui.horizontal(|ui| {
        ui.add_space(28.0);
        note(ui, "A new version downloads on its own and installs when you quit Kernship.");
    });
}

fn sidebar(app: &mut KernshipApp, ui: &mut Ui) {
    ui.label(RichText::new("Show these items in the sidebar:").size(14.0));
    ui.add_space(8.0);
    let p = &mut app.st.prefs;
    ui.indent("sb-items", |ui| {
        ui.spacing_mut().item_spacing.y = 10.0;
        mac_check(ui, &mut p.show_all_fonts, "All Fonts");
        mac_check(ui, &mut p.show_system, "System Fonts");
        mac_check(ui, &mut p.show_starred, "Starred");
        mac_check(ui, &mut p.show_recents, "Recents");
        mac_check(ui, &mut p.show_google, "Google Fonts");
        mac_check(ui, &mut p.show_icon_fonts, "Icon Fonts");
        mac_check(ui, &mut p.show_smart_filters, "Smart Filters");
    });
}

fn import(app: &mut KernshipApp, ui: &mut Ui) {
    ui.label(RichText::new("When Importing:").size(14.0));
    ui.add_space(8.0);
    let p = &mut app.st.prefs;
    ui.indent("imp-items", |ui| {
        ui.spacing_mut().item_spacing.y = 10.0;
        mac_check(ui, &mut p.skip_duplicates, "Skip duplicate fonts (same PostScript name)");
        mac_check(ui, &mut p.activate_after_import, "Activate fonts automatically");
        mac_check(ui, &mut p.sets_from_folders, "Create sets from folder structure");
        mac_check(ui, &mut p.copy_into_library, "Copy font files into the library");
    });
}

fn shortcut_label(ev: &egui::Event) -> Option<String> {
    if let egui::Event::Key { key, pressed: true, modifiers: m, .. } = ev {
        if !(m.ctrl || m.alt || m.command) {
            return None;
        }
        let mut s = String::new();
        if m.ctrl {
            s.push('⌃');
        }
        if m.alt {
            s.push('⌥');
        }
        if m.shift {
            s.push('⇧');
        }
        if m.command {
            s.push('⌘');
        }
        s.push_str(key.name());
        return Some(s);
    }
    None
}

/// The shortcut text of a key press, to compare with the recorded one.
pub fn pressed_shortcut(ctx: &egui::Context) -> Option<String> {
    ctx.input(|i| i.events.iter().find_map(shortcut_label))
}

fn auto_activation(app: &mut KernshipApp, ui: &mut Ui) {
    ui.columns(2, |cols| {
        let ui = &mut cols[0];
        let mut on = app.st.prefs.enable_auto_activation;
        if mac_check_sized(ui, &mut on, "Auto Activation", 15.5, true).changed() {
            app.st.prefs.enable_auto_activation = on;
        }
        ui.add_space(8.0);
        let enabled = app.st.prefs.enable_auto_activation;
        egui::Frame::NONE
            .fill(pal().panel)
            .stroke(Stroke::new(1.0_f32, pal().rule))
            .inner_margin(egui::Margin::same(1))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().id_salt("aa-apps").max_height(300.0).min_scrolled_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for (i, a) in design_apps::APPS.iter().enumerate() {
                        let w = ui.available_width();
                        let (row, _) = ui.allocate_exact_size(vec2(w, 24.0), Sense::hover());
                        if i % 2 == 1 {
                            ui.painter().rect_filled(row, 0.0, pal().panel_hi);
                        }
                        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(row.shrink2(vec2(10.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
                        child.spacing_mut().item_spacing.x = 8.0;
                        let v = app.st.prefs.auto_activation_apps.entry(a.bundle_id.to_string()).or_insert(a.default_on);
                        let mut val = *v;
                        if mac_check_sized(&mut child, &mut val, "", 14.0, enabled).changed() {
                            *v = val;
                        }
                        app_badge(app, &mut child, a);
                        let col = if enabled { pal().ink } else { pal().faint };
                        child.label(RichText::new(a.label()).size(14.0).color(col));
                    }
                });
            });
        ui.add_space(10.0);
        note(ui, "Automatically activates missing fonts when\nopening documents in enabled apps.");

        let ui = &mut cols[1];
        ui.add_space(2.0);
        heading(ui, "Manual Check");
        let p = &mut app.st.prefs;
        mac_check(ui, &mut p.check_shortcut_enabled, "Shortcut to check missing fonts");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_space(28.0);
            let label = if app.recording_shortcut {
                "Type the shortcut…".to_string()
            } else {
                app.st.prefs.check_shortcut.clone().unwrap_or_else(|| "Click to record shortcut".into())
            };
            let enabled = app.st.prefs.check_shortcut_enabled;
            let b = ui.add_enabled(enabled, egui::Button::new(RichText::new(label).size(14.0)).min_size(vec2(200.0, 30.0)));
            if b.clicked() {
                app.recording_shortcut = !app.recording_shortcut;
            }
            if b.secondary_clicked() {
                app.st.prefs.check_shortcut = None;
            }
        });
        if app.recording_shortcut {
            if let Some(s) = pressed_shortcut(ui.ctx()) {
                app.st.prefs.check_shortcut = Some(s);
                app.recording_shortcut = false;
            }
        }
        ui.horizontal(|ui| {
            ui.add_space(28.0);
            note(ui, "(Adobe CC & QuarkXPress only)");
        });
        ui.add_space(18.0);
        heading(ui, "Activation Options");
        let p = &mut app.st.prefs;
        ui.spacing_mut().item_spacing.y = 10.0;
        mac_check(ui, &mut p.activate_entire_family, "Activate entire font family");
        mac_check(ui, &mut p.silent_mode, "Silent Mode (no alerts)");
        ui.add_space(16.0);
        heading(ui, "Auto Deactivation");
        note(ui, "Deactivate when Kernship quits:");
        ui.add_space(4.0);
        mac_check(ui, &mut p.deactivate_auto_on_quit, "Auto-activated fonts");
        mac_check(ui, &mut p.deactivate_manual_on_quit, "Manually activated fonts");
    });
}

/// Draws the Settings window; returns false when it is closed.
pub fn window(app: &mut KernshipApp, ctx: &egui::Context, tab: &mut PrefTab, now: f64) -> bool {
    let mut open = true;
    let before = serde_json::to_string(&app.st.prefs).ok();
    let frame = egui::Frame::window(&ctx.style())
        .fill(pal().menu)
        .stroke(Stroke::new(1.0_f32, pal().rule_strong))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::ZERO);
    egui::Window::new("Settings")
        .title_bar(false)
        .frame(frame)
        .resizable(false)
        .collapsible(false)
        .fixed_size(vec2(WIDTH, 0.0))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, -10.0))
        .show(ctx, |ui| {
            ui.set_width(WIDTH);
            // Title bar: a close control and the pane's name.
            let (bar, _) = ui.allocate_exact_size(vec2(WIDTH, 38.0), Sense::hover());
            let p = ui.painter().clone();
            let close = Rect::from_center_size(pos2(bar.left() + 22.0, bar.center().y), vec2(24.0, 24.0));
            let cr = ui.interact(close, ui.id().with("settings-close"), Sense::click()).on_hover_text("Close");
            if cr.hovered() {
                p.rect_filled(close, CTRL_RADIUS, pal().panel_hi);
            }
            paint_cross(&p, close.center(), 4.5, pal().ink);
            focus_ring(ui, &cr, close, CTRL_RADIUS);
            if cr.clicked() {
                open = false;
            }
            p.text(bar.center(), Align2::CENTER_CENTER, tab_name(*tab), ui_bold(14.0), pal().ink);

            // Toolbar of panes.
            let tabs = [PrefTab::General, PrefTab::Sidebar, PrefTab::Import, PrefTab::AutoActivation];
            let widths: Vec<f32> = tabs
                .iter()
                .map(|t| ui.fonts(|f| f.layout_no_wrap(tab_name(*t).to_string(), ui_font(13.5), pal().ink).size().x).max(40.0) + 22.0)
                .collect();
            let total: f32 = widths.iter().sum::<f32>() + 6.0 * (tabs.len() as f32 - 1.0);
            let (tb, _) = ui.allocate_exact_size(vec2(WIDTH, 66.0), Sense::hover());
            let mut x = tb.center().x - total / 2.0;
            for (t, w) in tabs.iter().zip(widths) {
                let r = Rect::from_min_size(pos2(x, tb.top() + 2.0), vec2(w, 58.0));
                x += w + 6.0;
                let resp = ui.interact(r, ui.id().with(("pref-tab", tab_name(*t))), Sense::click());
                let sel = *tab == *t;
                if sel {
                    p.rect_filled(r, CTRL_RADIUS, pal().ink);
                } else if resp.hovered() {
                    p.rect_filled(r, CTRL_RADIUS, pal().panel_hi);
                }
                focus_ring(ui, &resp, r, CTRL_RADIUS);
                let col = if sel { pal().ground } else { pal().ink };
                tab_icon(&p, pos2(r.center().x, r.top() + 20.0), *t, col);
                p.text(pos2(r.center().x, r.bottom() - 13.0), Align2::CENTER_CENTER, tab_name(*t), ui_font(13.5), col);
                if resp.clicked() {
                    *tab = *t;
                    app.recording_shortcut = false;
                }
            }
            p.line_segment([pos2(tb.left(), tb.bottom()), pos2(tb.right(), tb.bottom())], Stroke::new(1.0_f32, pal().rule));

            egui::Frame::NONE.inner_margin(egui::Margin { left: 26, right: 26, top: 22, bottom: 22 }).show(ui, |ui| {
                ui.set_width(WIDTH - 52.0);
                ui.style_mut().visuals.override_text_color = Some(pal().ink);
                match tab {
                    PrefTab::General => general(app, ui),
                    PrefTab::Sidebar => sidebar(app, ui),
                    PrefTab::Import => import(app, ui),
                    PrefTab::AutoActivation => auto_activation(app, ui),
                }
            });
        });
    if serde_json::to_string(&app.st.prefs).ok() != before {
        app.apply_watcher_settings();
        app.dirty = true;
        app.save_soon(now);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !app.recording_shortcut {
        open = false;
    }
    open
}
