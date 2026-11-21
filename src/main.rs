mod activate;
mod app;
mod brand;
mod design_apps;
mod google;
mod icons;
mod macos;
mod menubar;
mod model;
mod props;
mod render;
mod scan;
mod store;
mod theme;
mod ui_main;
mod ui_icons;
mod ui_list;
mod ui_panels;
mod ui_search;
mod ui_settings;
mod ui_sidebar;
mod webfont;
mod ui_webfont;
mod update;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            screenshot = Some(PathBuf::from(&args[i + 1]));
            i += 1;
        }
        i += 1;
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Kernship")
        .with_inner_size([1150.0, 800.0])
        .with_min_inner_size([760.0, 480.0])
        .with_app_id("Kernship")
        .with_fullsize_content_view(true)
        .with_titlebar_shown(false)
        .with_title_shown(false);
    if let Some(icon) = brand::icon() {
        viewport = viewport.with_icon(icon);
    }
    if screenshot.is_some() {
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }

    let quiet = screenshot.is_some();
    let mut options = eframe::NativeOptions {
        viewport,
        run_and_return: false,
        ..Default::default()
    };
    #[cfg(target_os = "macos")]
    if quiet {
        options.event_loop_builder = Some(Box::new(|builder| {
            use winit_mac::EventLoopBuilderExtMacOS;
            builder.with_activate_ignoring_other_apps(false);
        }));
    }
    #[cfg(not(target_os = "macos"))]
    let _ = quiet;

    eframe::run_native(
        "Kernship",
        options,
        Box::new(move |cc| Ok(Box::new(app::KernshipApp::new(cc, screenshot)))),
    )
}

#[cfg(target_os = "macos")]
mod winit_mac {
    pub use winit::platform::macos::EventLoopBuilderExtMacOS;
}
