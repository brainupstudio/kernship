//! Working with design programs.
//!
//! - Activated fonts are registered with the system font manager, so every program (Adobe,
//!   Sketch, Figma, Affinity, Pixelmator, Office...) lists them at once.
//! - Auto activation: once a second, when an enabled Adobe program is in front, the open
//!   documents are asked (through AppleScript `do javascript`) which fonts they use; the ones
//!   found in the libraries or in Google Fonts are switched on. Nothing is installed into the
//!   Adobe programs (the scripts run in place).
//! - "Apply to Selected Text" sends the chosen font to the selection of the design program
//!   that was in front last.
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct Request {
    pub app: String,
    pub fonts: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Photoshop,
    Illustrator,
    InDesign,
    InCopy,
    Sketch,
    /// Reads activated fonts directly; nothing to ask.
    Native,
}

pub struct AppInfo {
    pub name: &'static str,
    pub bundle_id: &'static str,
    pub kind: Kind,
    pub versions: &'static str,
    /// Letters and colours for the little app badge when the app's own icon isn't available.
    pub abbr: &'static str,
    pub bg: [u8; 3],
    pub fg: [u8; 3],
    /// Ticked in the person's original (iWork, iMovie and Final Cut are off there; estimated).
    pub default_on: bool,
}

/// The Auto Activation table: the design apps Kernship knows, with the version ranges shown.
/// Adobe apps are asked for their documents' fonts; the others list activated fonts themselves.
pub const APPS: [AppInfo; 22] = [
    AppInfo { name: "Adobe XD", bundle_id: "com.adobe.xd", kind: Kind::Native, versions: "", abbr: "Xd", bg: [70, 1, 55], fg: [255, 97, 246], default_on: true },
    AppInfo { name: "Affinity Designer 2", bundle_id: "com.seriflabs.affinitydesigner2", kind: Kind::Native, versions: "", abbr: "Ad", bg: [19, 72, 129], fg: [140, 210, 255], default_on: true },
    AppInfo { name: "Affinity Photo 2", bundle_id: "com.seriflabs.affinityphoto2", kind: Kind::Native, versions: "", abbr: "Ap", bg: [74, 34, 120], fg: [214, 150, 255], default_on: true },
    AppInfo { name: "Affinity Publisher 2", bundle_id: "com.seriflabs.affinitypublisher2", kind: Kind::Native, versions: "", abbr: "Pu", bg: [120, 40, 30], fg: [255, 150, 110], default_on: true },
    AppInfo { name: "After Effects", bundle_id: "com.adobe.AfterEffects", kind: Kind::Native, versions: "25.2.0 - 26", abbr: "Ae", bg: [0, 0, 91], fg: [153, 153, 255], default_on: true },
    AppInfo { name: "Animate", bundle_id: "com.adobe.Animate", kind: Kind::Native, versions: "CC - CC 2023", abbr: "An", bg: [0, 0, 91], fg: [255, 79, 26], default_on: true },
    AppInfo { name: "Excel", bundle_id: "com.microsoft.Excel", kind: Kind::Native, versions: "", abbr: "X", bg: [16, 124, 65], fg: [255, 255, 255], default_on: true },
    AppInfo { name: "Figma", bundle_id: "com.figma.Desktop", kind: Kind::Native, versions: "", abbr: "F", bg: [245, 245, 245], fg: [242, 78, 30], default_on: true },
    AppInfo { name: "Final Cut Pro", bundle_id: "com.apple.FinalCut", kind: Kind::Native, versions: "", abbr: "Fc", bg: [30, 30, 30], fg: [230, 230, 230], default_on: false },
    AppInfo { name: "Illustrator", bundle_id: "com.adobe.illustrator", kind: Kind::Illustrator, versions: "CS6 - CC 2026", abbr: "Ai", bg: [51, 0, 0], fg: [255, 154, 0], default_on: true },
    AppInfo { name: "iMovie", bundle_id: "com.apple.iMovieApp", kind: Kind::Native, versions: "", abbr: "★", bg: [96, 64, 200], fg: [255, 255, 255], default_on: false },
    AppInfo { name: "InCopy", bundle_id: "com.adobe.InCopy", kind: Kind::InCopy, versions: "CS6 - CC 2026", abbr: "Ic", bg: [73, 2, 31], fg: [255, 51, 160], default_on: true },
    AppInfo { name: "InDesign", bundle_id: "com.adobe.InDesign", kind: Kind::InDesign, versions: "CS6 - CC 2026", abbr: "Id", bg: [73, 2, 31], fg: [255, 51, 102], default_on: true },
    AppInfo { name: "Keynote", bundle_id: "com.apple.iWork.Keynote", kind: Kind::Native, versions: "", abbr: "K", bg: [30, 120, 230], fg: [255, 255, 255], default_on: false },
    AppInfo { name: "Numbers", bundle_id: "com.apple.iWork.Numbers", kind: Kind::Native, versions: "", abbr: "N", bg: [40, 170, 70], fg: [255, 255, 255], default_on: false },
    AppInfo { name: "Pages", bundle_id: "com.apple.iWork.Pages", kind: Kind::Native, versions: "", abbr: "P", bg: [245, 150, 30], fg: [255, 255, 255], default_on: false },
    AppInfo { name: "Photoshop", bundle_id: "com.adobe.Photoshop", kind: Kind::Photoshop, versions: "CS6 - CC 2026", abbr: "Ps", bg: [0, 30, 54], fg: [49, 168, 255], default_on: true },
    AppInfo { name: "Pixelmator Pro", bundle_id: "com.pixelmatorteam.pixelmator.x", kind: Kind::Native, versions: "", abbr: "Px", bg: [40, 40, 50], fg: [120, 200, 255], default_on: true },
    AppInfo { name: "PowerPoint", bundle_id: "com.microsoft.Powerpoint", kind: Kind::Native, versions: "", abbr: "P", bg: [196, 62, 28], fg: [255, 255, 255], default_on: true },
    AppInfo { name: "QuarkXPress", bundle_id: "com.quark.QuarkXPress", kind: Kind::Native, versions: "10 - 2026", abbr: "Q", bg: [20, 90, 160], fg: [255, 255, 255], default_on: true },
    AppInfo { name: "Sketch", bundle_id: "com.bohemiancoding.sketch3", kind: Kind::Sketch, versions: "", abbr: "S", bg: [253, 173, 0], fg: [255, 255, 255], default_on: true },
    AppInfo { name: "Word", bundle_id: "com.microsoft.Word", kind: Kind::Native, versions: "", abbr: "W", bg: [24, 90, 189], fg: [255, 255, 255], default_on: true },
];

impl AppInfo {
    pub fn label(&self) -> String {
        if self.versions.is_empty() {
            self.name.to_string()
        } else {
            format!("{} ({})", self.name, self.versions)
        }
    }
}

pub fn app_by_bundle(bid: &str) -> Option<&'static AppInfo> {
    APPS.iter().find(|a| bid.eq_ignore_ascii_case(a.bundle_id) || bid.starts_with(a.bundle_id))
}

/// Fonts every program asks for that are never "missing" (they are skipped).
const SKIP: [&str; 6] = ["SFUIText", "SFUIDisplay", "SFCompact", "SFProText", "SFProDisplay", "HelveticaNeueUI"];

fn query_js(kind: Kind) -> &'static str {
    match kind {
        Kind::InDesign | Kind::InCopy => "var n=[];for(var j=0;j<app.documents.length;j++){var fs=app.documents[j].fonts.everyItem().getElements();for(var i=0;i<fs.length;i++){n.push(fs[i].postscriptName||fs[i].name);}}n.join('\\n');",
        Kind::Illustrator => "var n=[];for(var j=0;j<app.documents.length;j++){var x=app.documents[j].XMPString;var re=/<stFnt:fontName>([^<]+)<\\/stFnt:fontName>/g;var m;while((m=re.exec(x))!==null){n.push(m[1]);}}n.join('\\n');",
        Kind::Photoshop => "var n=[];for(var j=0;j<app.documents.length;j++){var x=app.documents[j].xmpMetadata.rawData;var re=/<stFnt:fontName>([^<]+)<\\/stFnt:fontName>/g;var m;while((m=re.exec(x))!==null){n.push(m[1]);}}n.join('\\n');",
        _ => "",
    }
}

fn run_js(bid: &str, kind: Kind, js: &str) -> Option<String> {
    let js = js.replace('\\', "\\\\").replace('"', "\\\"");
    let script = match kind {
        Kind::InDesign | Kind::InCopy => format!("tell application id \"{bid}\" to do script \"{js}\" language javascript"),
        Kind::Photoshop | Kind::Illustrator => format!("tell application id \"{bid}\" to do javascript \"{js}\""),
        _ => return None,
    };
    let o = Command::new("osascript").arg("-e").arg(script).output().ok()?;
    if o.status.success() {
        Some(String::from_utf8_lossy(&o.stdout).to_string())
    } else {
        None
    }
}

pub fn frontmost_bundle_id() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWorkspace;
        #[allow(unused_unsafe)]
        unsafe {
            let ws = NSWorkspace::sharedWorkspace();
            let app = ws.frontmostApplication()?;
            let bid = app.bundleIdentifier()?;
            Some(bid.to_string())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[derive(Default)]
struct Shared {
    enabled: bool,
    enabled_apps: Vec<String>,
    last_design_app: Option<String>,
    seen: std::collections::HashMap<String, String>,
}

pub struct DesignWatcher {
    shared: Arc<Mutex<Shared>>,
    rx: Receiver<Request>,
    tx: Sender<Request>,
    started: bool,
}

impl DesignWatcher {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { shared: Arc::new(Mutex::new(Shared::default())), rx, tx, started: false }
    }

    /// Starts the once-a-second check of the program in front.
    pub fn start(&mut self, ctx: egui::Context) {
        if self.started {
            return;
        }
        self.started = true;
        let shared = self.shared.clone();
        let tx = self.tx.clone();
        let own = std::process::id();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let Some(bid) = frontmost_bundle_id() else { continue };
            let _ = own;
            let Some(info) = app_by_bundle(&bid) else { continue };
            let (enabled, allowed) = {
                let mut s = shared.lock().unwrap();
                s.last_design_app = Some(bid.clone());
                (s.enabled, s.enabled_apps.iter().any(|a| a == info.bundle_id))
            };
            if !enabled || !allowed || matches!(info.kind, Kind::Native | Kind::Sketch) {
                continue;
            }
            if let Some(out) = run_js(info.bundle_id, info.kind, query_js(info.kind)) {
                let fonts: Vec<String> = out
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !SKIP.iter().any(|s| l.starts_with(s)))
                    .collect();
                let sig = fonts.join("|");
                let changed = {
                    let mut s = shared.lock().unwrap();
                    let prev = s.seen.insert(info.bundle_id.to_string(), sig.clone());
                    prev.as_deref() != Some(sig.as_str())
                };
                if changed && !fonts.is_empty() {
                    let _ = tx.send(Request { app: info.name.to_string(), fonts });
                    ctx.request_repaint();
                }
            }
        });
    }

    pub fn configure(&self, enabled: bool, apps: Vec<String>) {
        let mut s = self.shared.lock().unwrap();
        s.enabled = enabled;
        s.enabled_apps = apps;
    }

    pub fn poll_requests(&mut self) -> Vec<Request> {
        self.rx.try_iter().collect()
    }

    pub fn last_design_app(&self) -> Option<&'static AppInfo> {
        let s = self.shared.lock().unwrap();
        s.last_design_app.as_deref().and_then(app_by_bundle)
    }

    /// "Check Missing Fonts" now, for every running Adobe program.
    pub fn check_now(&self) {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            for a in APPS.iter().filter(|a| matches!(a.kind, Kind::Photoshop | Kind::Illustrator | Kind::InDesign | Kind::InCopy)) {
                if !is_running(a.bundle_id) {
                    continue;
                }
                if let Some(out) = run_js(a.bundle_id, a.kind, query_js(a.kind)) {
                    let fonts: Vec<String> = out.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
                    let _ = tx.send(Request { app: a.name.to_string(), fonts });
                }
            }
        });
    }
}

pub fn is_running(bundle_id: &str) -> bool {
    Command::new("osascript")
        .arg("-e")
        .arg(format!("application id \"{bundle_id}\" is running"))
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false)
}

/// Sends a font to the current selection of a design program ("Apply to Selected Text/Layer").
pub fn apply_font(app: &AppInfo, ps_name: &str, family: &str, style: &str) -> Result<(), String> {
    let ps = ps_name.replace('\'', "\\'");
    let fam = family.replace('\'', "\\'");
    let sty = style.replace('\'', "\\'");
    let js = match app.kind {
        Kind::Photoshop => format!(
            "var d=app.activeDocument;var l=d.activeLayer;if(l.kind==LayerKind.TEXT){{l.textItem.font='{ps}';}}"
        ),
        Kind::Illustrator => format!(
            "var f=app.textFonts.getByName('{ps}');var s=app.activeDocument.selection;if(s.typename=='TextRange'){{s.characterAttributes.textFont=f;}}else{{for(var i=0;i<s.length;i++){{if(s[i].typename=='TextFrame'){{s[i].textRange.characterAttributes.textFont=f;}}}}}}"
        ),
        Kind::InDesign | Kind::InCopy => format!(
            "var s=app.selection;for(var i=0;i<s.length;i++){{try{{s[i].appliedFont='{fam}\\t{sty}';}}catch(e){{}}}}"
        ),
        Kind::Sketch | Kind::Native => {
            return Err(format!("{} reads activated fonts directly: pick it in its font menu.", app.name));
        }
    };
    run_js(app.bundle_id, app.kind, &js).map(|_| ()).ok_or_else(|| format!("{} didn't accept the font", app.name))
}
