//! macOS-only helpers: dragging SVG files out of the window into other apps, and putting SVG on
//! the clipboard in a form design apps paste as artwork.
#![allow(unused_unsafe)]

use std::path::PathBuf;

#[cfg(target_os = "macos")]
mod imp {
    use objc2::rc::Retained;
    use objc2::runtime::{NSObject, ProtocolObject};
    use objc2::{define_class, msg_send, AllocAnyThread, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession, NSDraggingSource,
        NSPasteboard, NSView, NSWorkspace,
    };
    use objc2_foundation::{NSArray, NSData, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL};
    use std::cell::RefCell;
    use std::path::PathBuf;

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "KernshipDragSource"]
        struct DragSource;

        unsafe impl NSObjectProtocol for DragSource {}

        unsafe impl NSDraggingSource for DragSource {
            #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
            fn source_operation(&self, _session: &NSDraggingSession, _context: NSDraggingContext) -> NSDragOperation {
                NSDragOperation::Copy
            }
        }
    );

    /// The system Open panel for a folder. With `packages`, folders that macOS shows as a single
    /// item (a library package) can be picked too; anything that is not a folder on
    /// disk is refused.
    pub fn choose_folder(message: &str, packages: bool) -> Option<PathBuf> {
        let mtm = MainThreadMarker::new()?;
        let panel = objc2_app_kit::NSOpenPanel::openPanel(mtm);
        panel.setCanChooseDirectories(true);
        // A package counts as a file in the Open panel, so files must be allowed to pick one.
        panel.setCanChooseFiles(packages);
        panel.setTreatsFilePackagesAsDirectories(false);
        panel.setAllowsMultipleSelection(false);
        panel.setMessage(Some(&NSString::from_str(message)));
        panel.setPrompt(Some(&NSString::from_str("Open")));
        // NSModalResponseOK
        if panel.runModal() != 1 {
            return None;
        }
        let url = panel.URLs().firstObject()?;
        let path = PathBuf::from(url.path()?.to_string());
        path.is_dir().then_some(path)
    }

    thread_local! {
        static SOURCE: RefCell<Option<Retained<DragSource>>> = const { RefCell::new(None) };
    }

    /// Starts a native drag of files from the window's view, using the mouse event being handled.
    pub fn drag_files(ns_view: *mut std::ffi::c_void, files: &[PathBuf], at: (f64, f64)) -> bool {
        let Some(mtm) = MainThreadMarker::new() else { return false };
        if ns_view.is_null() || files.is_empty() {
            return false;
        }
        unsafe {
            let view: &NSView = &*(ns_view as *const NSView);
            let app = NSApplication::sharedApplication(mtm);
            let Some(event) = app.currentEvent() else { return false };
            let source = SOURCE.with(|s| {
                s.borrow_mut()
                    .get_or_insert_with(|| {
                        let obj = DragSource::alloc(mtm).set_ivars(());
                        msg_send![super(obj), init]
                    })
                    .clone()
            });
            let ws = NSWorkspace::sharedWorkspace();
            let bounds = view.bounds();
            let mut items = Vec::new();
            for (i, f) in files.iter().enumerate() {
                let path = NSString::from_str(&f.to_string_lossy());
                let url = NSURL::fileURLWithPath(&path);
                let item = NSDraggingItem::initWithPasteboardWriter(NSDraggingItem::alloc(), ProtocolObject::from_ref(&*url));
                let icon = ws.iconForFile(&path);
                // AppKit view coordinates may be flipped; place the image under the pointer.
                let y = if view.isFlipped() { at.1 } else { bounds.size.height - at.1 };
                let frame = NSRect::new(NSPoint::new(at.0 - 24.0 + i as f64 * 6.0, y - 24.0), NSSize::new(48.0, 48.0));
                item.setDraggingFrame_contents(frame, Some(&icon));
                items.push(item);
            }
            let array = NSArray::from_retained_slice(&items);
            let _session = view.beginDraggingSessionWithItems_event_source(&array, &event, ProtocolObject::from_ref(&*source));
            true
        }
    }

    /// Puts SVG on the clipboard as both an SVG image and text.
    pub fn copy_svg(svg: &str) {
        unsafe {
            let pb = NSPasteboard::generalPasteboard();
            pb.clearContents();
            let data = NSData::with_bytes(svg.as_bytes());
            let svg_type = NSString::from_str("public.svg-image");
            let _ = pb.setData_forType(Some(&data), &svg_type);
            let text_type = NSString::from_str("public.utf8-plain-text");
            let _ = pb.setString_forType(&NSString::from_str(svg), &text_type);
        }
    }

    /// The app's own icon (as installed on this Mac), as RGBA pixels `px` wide.
    pub fn app_icon_rgba(bundle_id: &str, px: u32) -> Option<(Vec<u8>, u32, u32)> {
        unsafe {
            let ws = NSWorkspace::sharedWorkspace();
            let url = ws.URLForApplicationWithBundleIdentifier(&NSString::from_str(bundle_id))?;
            let path = url.path()?;
            let img = ws.iconForFile(&path);
            img.setSize(NSSize::new(px as f64, px as f64));
            let tiff = img.TIFFRepresentation()?;
            // Let AppKit turn it into PNG (its TIFF uses encodings the image crate can't always read).
            let rep = objc2_app_kit::NSBitmapImageRep::imageRepWithData(&tiff)?;
            let props = objc2_foundation::NSDictionary::new();
            let png = rep.representationUsingType_properties(objc2_app_kit::NSBitmapImageFileType::PNG, &props)?;
            let bytes = png.to_vec();
            let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?;
            let small = decoded.resize(px, px, image::imageops::FilterType::Triangle).to_rgba8();
            let (w, h) = small.dimensions();
            Some((small.into_raw(), w, h))
        }
    }

    fn window_of(ns_view: *mut std::ffi::c_void) -> Option<Retained<objc2_app_kit::NSWindow>> {
        if ns_view.is_null() {
            return None;
        }
        let view: &NSView = unsafe { &*(ns_view as *const NSView) };
        view.window()
    }

    /// Puts the three window buttons at `x` (left edge of the first) and centres them on
    /// `center_y` points from the top, level with the sidebar's top row.
    /// macOS lays them out again on resize, so this runs every frame.
    pub fn position_traffic_lights(ns_view: *mut std::ffi::c_void, x: f64, center_y: f64) {
        use objc2_app_kit::NSWindowButton;
        let Some(window) = window_of(ns_view) else { return };
        unsafe {
            let (Some(first), Some(second), Some(third)) = (
                window.standardWindowButton(NSWindowButton::CloseButton),
                window.standardWindowButton(NSWindowButton::MiniaturizeButton),
                window.standardWindowButton(NSWindowButton::ZoomButton),
            ) else {
                return;
            };
            let Some(container) = first.superview().and_then(|v| v.superview()) else { return };
            let bh = first.frame().size.height;
            let top = center_y - bh / 2.0;
            let mut bar = container.frame();
            bar.size.height = bh + top;
            bar.origin.y = window.frame().size.height - bar.size.height;
            container.setFrame(bar);
            let spacing = 20.0;
            for (i, b) in [first, second, third].iter().enumerate() {
                let mut o = b.frame().origin;
                o.x = x + i as f64 * spacing;
                b.setFrameOrigin(o);
            }
        }
    }

    /// The window only moves when the app asks (empty toolbar areas), so dragging the size
    /// slider or any control near the top never drags the whole window.
    pub fn set_window_movable(ns_view: *mut std::ffi::c_void, movable: bool) {
        if let Some(window) = window_of(ns_view) {
            unsafe {
                window.setMovable(movable);
                window.setMovableByWindowBackground(false);
            }
        }
    }

    fn walk_menu(menu: &objc2_app_kit::NSMenu, f: &mut dyn FnMut(&objc2_app_kit::NSMenuItem)) {
        unsafe {
            for item in menu.itemArray().iter() {
                f(&item);
                if let Some(sub) = item.submenu() {
                    walk_menu(&sub, f);
                }
            }
        }
    }

    /// SF Symbols next to the menu bar's items.
    pub fn decorate_menus(icons: &[(&str, &str)]) {
        let Some(mtm) = MainThreadMarker::new() else { return };
        let app = NSApplication::sharedApplication(mtm);
        let Some(menu) = (unsafe { app.mainMenu() }) else { return };
        walk_menu(&menu, &mut |item| unsafe {
            let title = item.title().to_string();
            if let Some((_, symbol)) = icons.iter().find(|(t, _)| *t == title) {
                let name = NSString::from_str(symbol);
                let image = if symbol.starts_with("NSStatus") {
                    objc2_app_kit::NSImage::imageNamed(&name)
                } else {
                    objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(&name, None)
                };
                if let Some(img) = image {
                    item.setImage(Some(&img));
                }
            }
        });
    }

    /// Ticks menu bar items (as Grid, Expand Font Families…).
    pub fn set_menu_checked(states: &[(&str, bool)]) {
        let Some(mtm) = MainThreadMarker::new() else { return };
        let app = NSApplication::sharedApplication(mtm);
        let Some(menu) = (unsafe { app.mainMenu() }) else { return };
        walk_menu(&menu, &mut |item| unsafe {
            let title = item.title().to_string();
            if let Some((_, on)) = states.iter().find(|(t, _)| *t == title) {
                item.setState(if *on { objc2_app_kit::NSControlStateValueOn } else { objc2_app_kit::NSControlStateValueOff });
            }
        });
    }

    static MAIN_VIEW: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    pub fn set_main_view(ns_view: *mut std::ffi::c_void) {
        MAIN_VIEW.store(ns_view as usize, std::sync::atomic::Ordering::Relaxed);
    }

    /// Closes the window to the menu bar: the app keeps running (auto activation goes on)
    /// and its Dock icon goes away until the window comes back.
    pub fn hide_window() {
        let Some(mtm) = MainThreadMarker::new() else { return };
        if let Some(w) = window_of(MAIN_VIEW.load(std::sync::atomic::Ordering::Relaxed) as *mut _) {
            unsafe {
                w.orderOut(None);
                NSApplication::sharedApplication(mtm).setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory);
            }
        }
    }

    pub fn show_window() {
        let Some(mtm) = MainThreadMarker::new() else { return };
        if let Some(w) = window_of(MAIN_VIEW.load(std::sync::atomic::Ordering::Relaxed) as *mut _) {
            unsafe {
                let app = NSApplication::sharedApplication(mtm);
                app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Regular);
                #[allow(deprecated)]
                app.activateIgnoringOtherApps(true);
                w.makeKeyAndOrderFront(None);
            }
        }
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "KernshipStatusTarget"]
        struct StatusTarget;

        unsafe impl NSObjectProtocol for StatusTarget {}

        impl StatusTarget {
            /// Left click: the window. Right click (or Control-click): the menu.
            #[unsafe(method(clicked:))]
            fn clicked(&self, _sender: Option<&objc2::runtime::AnyObject>) {
                let mtm = self.mtm();
                let ev = NSApplication::sharedApplication(mtm).currentEvent();
                let menu_wanted = ev.is_some_and(|e| {
                    e.r#type() == objc2_app_kit::NSEventType::RightMouseUp
                        || e.modifierFlags().contains(objc2_app_kit::NSEventModifierFlags::Control)
                });
                if menu_wanted {
                    // Cloned out: the menu runs its own loop, during which the icon may be removed.
                    let item = STATUS.with(|s| s.borrow().clone());
                    (|| {
                        let Some(item) = item else { return };
                        let menu = STATUS_MENU.load(std::sync::atomic::Ordering::Relaxed);
                        let Some(m) = (unsafe { Retained::retain(menu as *mut objc2_app_kit::NSMenu) }) else { return };
                        // The menu opens under the icon while it is attached; detach it afterwards
                        // so the next left click goes to the window again.
                        item.setMenu(Some(&m));
                        if let Some(b) = item.button(mtm) {
                            unsafe { b.performClick(None) };
                        }
                        item.setMenu(None);
                    })();
                } else {
                    show_window();
                    ON_OPEN.with(|f| {
                        if let Some(f) = f.borrow().as_ref() {
                            f();
                        }
                    });
                }
            }
        }
    );

    static STATUS_MENU: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    thread_local! {
        static STATUS: RefCell<Option<Retained<objc2_app_kit::NSStatusItem>>> = const { RefCell::new(None) };
        static STATUS_TARGET: RefCell<Option<Retained<StatusTarget>>> = const { RefCell::new(None) };
        static ON_OPEN: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
    }

    /// The Kernship icon in the menu bar: a left click brings the window back (then calls
    /// `on_open`), a right click opens `menu` (an NSMenu).
    pub fn set_status_item(on: bool, menu: *mut std::ffi::c_void, on_open: Box<dyn Fn()>) {
        let Some(mtm) = MainThreadMarker::new() else { return };
        STATUS_MENU.store(menu as usize, std::sync::atomic::Ordering::Relaxed);
        ON_OPEN.with(|f| *f.borrow_mut() = Some(on_open));
        STATUS.with(|s| unsafe {
            let mut s = s.borrow_mut();
            let bar = objc2_app_kit::NSStatusBar::systemStatusBar();
            if !on {
                if let Some(item) = s.take() {
                    bar.removeStatusItem(&item);
                }
                return;
            }
            if s.is_some() || menu.is_null() {
                return;
            }
            let item = bar.statusItemWithLength(-1.0);
            if let Some(button) = item.button(mtm) {
                // The Kernship K as a template image: macOS draws it black on a light menu bar
                // and white on a dark one (and white when the menu is open), like its own icons.
                let img = crate::brand::menubar_icon_png([0, 0, 0]).and_then(|(png, w, h)| {
                    let data = NSData::with_bytes(&png);
                    let img = objc2_app_kit::NSImage::initWithData(objc2_app_kit::NSImage::alloc(), &data)?;
                    img.setSize(NSSize::new(w as f64 / 2.0, h as f64 / 2.0));
                    Some(img)
                });
                let img = img.or_else(|| {
                    objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
                        &NSString::from_str("k.square"),
                        Some(&NSString::from_str("Kernship")),
                    )
                });
                if let Some(img) = img {
                    img.setTemplate(true);
                    button.setImage(Some(&img));
                }
                let target = STATUS_TARGET.with(|t| {
                    t.borrow_mut()
                        .get_or_insert_with(|| unsafe { msg_send![StatusTarget::alloc(mtm), init] })
                        .clone()
                });
                button.setTarget(Some(&target));
                button.setAction(Some(objc2::sel!(clicked:)));
                button.sendActionOn(
                    objc2_app_kit::NSEventMask::LeftMouseUp | objc2_app_kit::NSEventMask::RightMouseUp,
                );
            }
            *s = Some(item);
        });
    }

    #[allow(dead_code)]
    fn _assert_any_thread() {
        let _ = <NSURL as AllocAnyThread>::alloc;
    }
}

#[cfg(target_os = "macos")]
pub use imp::{
    app_icon_rgba, choose_folder, copy_svg, decorate_menus, drag_files, hide_window, position_traffic_lights, set_main_view, set_menu_checked, set_status_item,
    set_window_movable, show_window,
};

#[cfg(not(target_os = "macos"))]
pub fn choose_folder(_message: &str, _packages: bool) -> Option<PathBuf> {
    None
}
#[cfg(not(target_os = "macos"))]
pub fn position_traffic_lights(_v: *mut std::ffi::c_void, _x: f64, _y: f64) {}
#[cfg(not(target_os = "macos"))]
pub fn set_window_movable(_v: *mut std::ffi::c_void, _m: bool) {}
#[cfg(not(target_os = "macos"))]
pub fn decorate_menus(_icons: &[(&str, &str)]) {}
#[cfg(not(target_os = "macos"))]
pub fn set_menu_checked(_states: &[(&str, bool)]) {}
#[cfg(not(target_os = "macos"))]
pub fn set_main_view(_v: *mut std::ffi::c_void) {}
#[cfg(not(target_os = "macos"))]
pub fn hide_window() {}
#[cfg(not(target_os = "macos"))]
pub fn show_window() {}

#[cfg(not(target_os = "macos"))]
pub fn drag_files(_ns_view: *mut std::ffi::c_void, _files: &[PathBuf], _at: (f64, f64)) -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn copy_svg(_svg: &str) {}

#[cfg(not(target_os = "macos"))]
pub fn app_icon_rgba(_bundle_id: &str, _px: u32) -> Option<(Vec<u8>, u32, u32)> {
    None
}

/// Folder for the SVG files of a drag.
pub fn drag_dir() -> PathBuf {
    let d = std::env::temp_dir().join("Kernship").join("drag");
    let _ = std::fs::create_dir_all(&d);
    d
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    #[test]
    fn finder_icon() {
        let r = super::app_icon_rgba("com.apple.finder", 64);
        println!("finder icon: {:?}", r.as_ref().map(|x| (x.1, x.2)));
        let r = super::app_icon_rgba("com.figma.Desktop", 64);
        println!("figma icon: {:?}", r.as_ref().map(|x| (x.1, x.2)));
    }
}
