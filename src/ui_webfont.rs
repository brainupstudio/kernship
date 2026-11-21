//! The "Convert Font for Web" pane: the right-hand sidebar in its web mode (toolbar button next
//! to Info, or "Convert Font for Web" in the font menu).
use crate::app::KernshipApp;
use crate::theme::*;
use crate::webfont::{self, DISPLAYS, SUBSETS};
use egui::{vec2, RichText, Ui};

fn section(ui: &mut Ui, title: &str) {
    ui.add_space(7.0);
    let w = ui.available_width();
    let (line, _) = ui.allocate_exact_size(vec2(w, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(line, 0.0, pal().rule);
    ui.add_space(6.0);
    ui.label(RichText::new(title).font(ui_bold(12.0)).color(pal().dim));
    ui.add_space(2.0);
}

fn check(ui: &mut Ui, on: &mut bool, label: &str, enabled: bool) -> bool {
    mac_check_sized(ui, on, label, 13.0, enabled).changed()
}

fn human(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{:.0} KB", (bytes as f64 / 1000.0).max(1.0))
    }
}

/// A full-width button; the primary one is the pane's single ink-filled action.
fn wide_button(ui: &mut Ui, label: &str, primary: bool, enabled: bool) -> bool {
    let w = ui.available_width();
    button_w(ui, label, if primary { Btn::Primary } else { Btn::Plain }, enabled, w, if primary { 38.0 } else { 32.0 })
}

pub fn web_panel(app: &mut KernshipApp, ui: &mut Ui, now: f64) {
    // Pick up a finished conversion.
    if let Some(job) = app.web_job.clone() {
        let done = job.lock().done.take();
        if let Some(res) = done {
            app.web_job = None;
            match &res {
                Ok(r) => app.toast(format!("Web fonts saved to {}", r.folder.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()), now),
                Err(e) => app.toast(format!("Conversion failed: {e}"), now),
            }
            app.web_result = Some(res);
        } else {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
        paint_convert_icon(ui.painter(), r.center(), 8.0, pal().ink);
        ui.label(RichText::new("Convert for Web").font(ui_bold(16.0)).color(pal().ink));
    });
    let inputs = app.web_inputs();
    if inputs.is_empty() {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No Selection").size(15.0).color(pal().dim));
            ui.label(RichText::new("Select one or more fonts to turn them into web fonts.").color(pal().dim));
        });
        return;
    }
    let sub = if inputs.len() == 1 {
        format!("{} {}", inputs[0].0.family, inputs[0].0.style)
    } else {
        format!("{} fonts selected", inputs.len())
    };
    ui.label(RichText::new(sub).color(pal().dim));
    if inputs.iter().any(|i| i.1.is_some()) {
        ui.label(RichText::new("Google styles are downloaded first.").small().color(pal().dim));
    }

    let before = app.st.webfont.clone();
    let running = app.web_job.is_some();
    let footer = if running { 126.0 } else { 90.0 };
    egui::ScrollArea::vertical().id_salt("webfont").auto_shrink([false, false]).max_height((ui.available_height() - footer).max(60.0)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let o = &mut app.st.webfont;

        section(ui, "Formats");
        egui::Grid::new("wf-formats").num_columns(2).spacing(vec2(18.0, 6.0)).show(ui, |ui| {
            check(ui, &mut o.woff2, "WOFF2", true);
            check(ui, &mut o.woff, "WOFF", true);
            ui.end_row();
            check(ui, &mut o.ttf, "TTF / OTF", true);
            check(ui, &mut o.eot, "EOT", true);
            ui.end_row();
        });

        section(ui, "Characters");
        let mut all = o.subsets.is_empty() && o.custom_chars.trim().is_empty();
        if check(ui, &mut all, "All glyphs", true) && all {
            o.subsets.clear();
            o.custom_chars.clear();
        }
        ui.add_space(2.0);
        for (name, _) in SUBSETS {
            let mut on = o.subsets.contains(name);
            if check(ui, &mut on, name, true) {
                if on {
                    o.subsets.insert(name.to_string());
                } else {
                    o.subsets.remove(name);
                }
            }
        }
        ui.add_space(4.0);
        ui.add(
            egui::TextEdit::singleline(&mut o.custom_chars)
                .hint_text(RichText::new("Extra characters, e.g. €£¥").color(pal().dim))
                .margin(egui::Margin::symmetric(8, 6))
                .desired_width(f32::INFINITY),
        );
        ui.label(RichText::new("Subsetting keeps only these characters (TrueType outlines).").small().color(pal().dim));

        section(ui, "Stylesheet");
        ui.allocate_ui_with_layout(vec2(ui.available_width(), 28.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            // Painted rather than laid out, so it shares the dropdown's text line.
            let g = ui.fonts(|f| f.layout_no_wrap("font-display".to_string(), ui_font(13.0), pal().dim));
            let (r, _) = ui.allocate_exact_size(vec2(g.size().x, 28.0), egui::Sense::hover());
            ui.painter().galley(egui::pos2(r.left(), r.center().y - g.size().y / 2.0 + 3.5), g, pal().dim);
            egui::ComboBox::from_id_salt("wf-display").icon(popup_icon).width(110.0).selected_text(o.display.clone()).show_ui(ui, |ui| {
                for d in DISPLAYS {
                    ui.selectable_value(&mut o.display, d.to_string(), d);
                }
            });
        });
        ui.add_space(4.0);
        let hint = inputs[0].0.family.clone();
        ui.add(
            egui::TextEdit::singleline(&mut o.family_override)
                .hint_text(RichText::new(format!("font-family: {hint}")).color(pal().dim))
                .margin(egui::Margin::symmetric(8, 6))
                .desired_width(f32::INFINITY),
        );
        ui.add_space(4.0);
        check(ui, &mut o.local, "Use installed copy first (local())", true);
        check(ui, &mut o.base64, "Embed WOFF/WOFF2 in CSS (base64)", true);

        section(ui, "Output");
        check(ui, &mut o.demo, "Demo page (demo.html)", true);
        check(ui, &mut o.zip, "ZIP archive", true);
        ui.horizontal(|ui| {
            let dir = o.save_dir();
            let name = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| dir.display().to_string());
            ui.label(RichText::new("Save to").color(pal().dim));
            ui.label(RichText::new(name).color(pal().ink)).on_hover_text(dir.display().to_string());
            if ui.small_button("Change…").clicked() {
                if let Some(p) = choose_folder() {
                    o.save_to = p.display().to_string();
                }
            }
        });

        if let Some(res) = app.web_result.clone() {
            ui.add_space(10.0);
            match res {
                Ok(r) => {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(12.0, 14.0), egui::Sense::hover());
                        paint_mark(ui.painter(), r.center(), 5.0, Mark::Active, pal().ink, pal().ink);
                        ui.label(RichText::new("Converted").font(ui_bold(12.5)).color(pal().ink));
                    });
                    let total: u64 = r.files.iter().filter(|f| f.0.ends_with(".woff2")).map(|f| f.1).sum();
                    for (name, size) in &r.files {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(name).small().color(pal().ink));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(human(*size)).small().color(pal().dim));
                            });
                        });
                    }
                    if total > 0 && r.original_bytes > 0 {
                        let saved = 100.0 - total as f64 * 100.0 / r.original_bytes as f64;
                        ui.label(RichText::new(format!("WOFF2 is {saved:.0}% smaller than the original {}", human(r.original_bytes))).small().color(pal().dim));
                    }
                    for n in &r.notes {
                        ui.label(RichText::new(n).small().color(pal().dim));
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Show in Finder").clicked() {
                            let target = r.zip.clone().unwrap_or_else(|| r.folder.clone());
                            let _ = std::process::Command::new("open").arg("-R").arg(target).spawn();
                        }
                        let demo = r.folder.join("demo.html");
                        if demo.exists() && ui.button("Open Demo").clicked() {
                            let _ = std::process::Command::new("open").arg(demo).spawn();
                        }
                    });
                }
                Err(e) => {
                    ui.label(RichText::new(e).color(pal().danger));
                }
            }
        }

        // What the stylesheet will say.
        section(ui, "@font-face");
        let faces: Vec<webfont::Input> = inputs.iter().take(2).map(|i| i.0.clone()).collect();
        let mut css = webfont::css_preview(&faces, &app.st.webfont);
        if inputs.len() > 2 {
            css.push_str(&format!("/* … {} more */\n", inputs.len() - 2));
        }
        egui::Frame::new().fill(pal().field).stroke(egui::Stroke::new(1.0_f32, pal().rule)).corner_radius(CTRL_RADIUS).inner_margin(10.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(egui::Label::new(RichText::new(css).monospace().size(11.0).color(pal().ink)).wrap());
        });
        ui.add_space(10.0);
    });
    ui.add_space(10.0);
    let o = &app.st.webfont;
    let any_format = o.woff2 || o.woff || o.ttf || o.eot;
    let n = inputs.len();
    let go = if running { "Converting…".to_string() } else if n == 1 { "Convert 1 font".to_string() } else { format!("Convert {n} fonts") };
    if wide_button(ui, &go, true, any_format && !running) {
        app.start_web_convert(ui.ctx());
    }
    ui.add_space(6.0);
    if wide_button(ui, "Copy @font-face CSS", false, any_format) {
        // The rules of the last conversion of this selection (they may embed base64 data),
        // otherwise the rules the conversion will write.
        let faces: Vec<webfont::Input> = inputs.iter().map(|i| i.0.clone()).collect();
        let css = match &app.web_result {
            Some(Ok(r)) if !r.css.is_empty() && app.st.webfont == before => r.css.clone(),
            _ => webfont::css_preview(&faces, &app.st.webfont),
        };
        ui.ctx().copy_text(css);
        app.toast("@font-face CSS copied", now);
    }

    if let Some(job) = &app.web_job {
        let j = job.lock();
        ui.add_space(8.0);
        ui.label(RichText::new(&j.stage).small().color(pal().dim));
        ui.add(egui::ProgressBar::new(j.progress).desired_height(6.0));
    }
    if app.st.webfont != before {
        app.save_soon(now);
    }
}

fn choose_folder() -> Option<std::path::PathBuf> {
    let o = std::process::Command::new("osascript")
        .arg("-e")
        .arg("POSIX path of (choose folder with prompt \"Save web fonts to\")")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (!s.is_empty()).then(|| std::path::PathBuf::from(s))
}
