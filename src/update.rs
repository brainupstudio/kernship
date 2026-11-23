//! Is a newer Kernship out? One read of the latest release of the public repository where
//! the packages are published. Nothing about the person or their fonts is sent: the request
//! carries only "Kernship/<version>" as its user agent.
//!
//! The app also updates itself: it downloads the disk image for its processor, checks it
//! against the published SHA256SUMS.txt, copies the new Kernship.app into its cache, and leaves
//! a small script waiting for the app to end. When it ends (Quit, ⌘Q, logout, or "Restart and
//! Update") the script swaps the app in place, and reopens it if asked. A file the app downloads
//! itself is not marked as downloaded from the internet, so macOS opens the new version without
//! the warning shown for unsigned apps.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// This build's version.
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

const PRETEND: &str = "KERNSHIP_PRETEND_VERSION";

/// The version this copy compares against: its own, or the one in KERNSHIP_PRETEND_VERSION, so
/// the whole update can be tried from a build that is not older than the latest release.
pub fn current() -> String {
    std::env::var(PRETEND).ok().filter(|v| parse(v).is_some()).unwrap_or_else(|| CURRENT.to_string())
}

/// The Kernship.app this process runs from, when it runs from one (not a development build).
pub fn installed_app() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?.canonicalize().ok()?;
    // .../Kernship.app/Contents/MacOS/Kernship
    let app = exe.parent()?.parent()?.parent()?.to_path_buf();
    (app.extension().is_some_and(|e| e == "app") && app.join("Contents/Info.plist").is_file()).then_some(app)
}

/// The app can replace itself: it runs from a Kernship.app in a folder it can write to.
pub fn can_update_itself() -> Option<PathBuf> {
    let app = installed_app()?;
    let probe = app.parent()?.join(format!(".kernship-write-test-{}", std::process::id()));
    std::fs::File::create(&probe).ok()?;
    let _ = std::fs::remove_file(&probe);
    Some(app)
}

/// Why the app cannot replace itself, for the Updates settings; None when it can.
pub fn why_not_itself() -> Option<&'static str> {
    if installed_app().is_none() {
        Some("This copy does not run from Kernship.app, so it cannot update itself.")
    } else if can_update_itself().is_none() {
        Some("Kernship cannot write to its folder, so it cannot update itself.")
    } else {
        None
    }
}

/// Where a downloaded version waits to be installed.
fn update_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("Kernship").join("update")
}

/// The prepared copy of `version`, when it is there and complete.
pub fn staged(version: &str) -> Option<PathBuf> {
    let app = update_dir().join(version).join("Kernship.app");
    app.join("Contents/Info.plist").is_file().then_some(app)
}

/// The file that asks the waiting script to reopen the app after the swap.
fn restart_flag() -> PathBuf {
    update_dir().join("restart")
}

/// Ask the waiting script to reopen Kernship once it has installed the new version.
pub fn request_restart() -> Result<(), String> {
    std::fs::create_dir_all(update_dir()).and_then(|_| std::fs::write(restart_flag(), b"1")).map_err(|e| e.to_string())
}

/// The disk image for this processor.
fn dmg_name(version: &str) -> String {
    let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };
    format!("Kernship-{version}-{arch}.dmg")
}

/// Download `url` into `to`, following GitHub's redirects to its file storage.
fn fetch(url: &str, to: &Path) -> Result<(), String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--connect-timeout", "30", "--max-time", "600", "-o"])
        .arg(to)
        .args(["-A", &format!("Kernship/{CURRENT}")])
        .arg(url)
        .output()
        .map_err(|_| "No connection to GitHub.".to_string())?;
    match out.status.code() {
        Some(0) => Ok(()),
        Some(22) => Err("GitHub did not answer as expected.".into()),
        _ => Err("No connection to GitHub, or the download did not finish.".into()),
    }
}

fn sha256(file: &Path) -> Option<String> {
    let out = Command::new("shasum").args(["-a", "256"]).arg(file).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    out.status.success().then(|| text.split_whitespace().next().unwrap_or("").to_lowercase())
}

/// The checksum SHA256SUMS.txt gives for `file`.
fn expected_sum(sums: &str, file: &str) -> Option<String> {
    sums.lines().find_map(|l| {
        let mut it = l.split_whitespace();
        let (hash, name) = (it.next()?, it.next()?.trim_start_matches('*'));
        (name == file).then(|| hash.to_lowercase())
    })
}

/// Download `version`, check it, and copy its Kernship.app into the update folder. Returns the
/// prepared app.
pub fn download(version: &str) -> Result<PathBuf, String> {
    if let Some(app) = staged(version) {
        return Ok(app);
    }
    let dir = update_dir();
    // only the version being prepared is kept
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            if e.path().is_dir() {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
    let work = dir.join(format!("{version}.part"));
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let result = prepare(version, &dir, &work);
    let _ = std::fs::remove_dir_all(&work);
    result
}

fn prepare(version: &str, dir: &Path, work: &Path) -> Result<PathBuf, String> {
    let file = dmg_name(version);
    let base = format!("https://github.com/{REPO}/releases/download/v{version}");
    let dmg = work.join(&file);
    let sums_file = work.join("SHA256SUMS.txt");
    fetch(&format!("{base}/{file}"), &dmg)?;
    fetch(&format!("{base}/SHA256SUMS.txt"), &sums_file)?;
    let sums = std::fs::read_to_string(&sums_file).map_err(|_| "GitHub sent an unreadable answer.".to_string())?;
    let expected = expected_sum(&sums, &file).ok_or_else(|| "The new version has no published checksum, so it was not installed.".to_string())?;
    if sha256(&dmg).as_deref() != Some(expected.as_str()) {
        return Err("The download does not match its checksum, so it was not installed.".into());
    }
    let mount = work.join("volume");
    std::fs::create_dir_all(&mount).map_err(|e| e.to_string())?;
    let attached = Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-readonly", "-quiet", "-mountpoint"])
        .arg(&mount)
        .arg(&dmg)
        .status()
        .is_ok_and(|s| s.success());
    if !attached {
        return Err("The new version could not be opened.".into());
    }
    let target = dir.join(version);
    let _ = std::fs::create_dir_all(&target);
    let copied = Command::new("ditto").arg(mount.join("Kernship.app")).arg(target.join("Kernship.app")).status().is_ok_and(|s| s.success());
    let _ = Command::new("hdiutil").args(["detach", "-quiet"]).arg(&mount).status();
    if !copied {
        let _ = std::fs::remove_dir_all(&target);
        return Err("The new version could not be prepared.".into());
    }
    staged(version).ok_or_else(|| "The new version could not be prepared.".to_string())
}

/// Leave a script waiting for this process to end; then it replaces `app` with `new_app` and,
/// when a restart was asked for, opens it again. If the copy fails the old app is put back.
pub fn install_when_closed(app: &Path, new_app: &Path) -> Result<(), String> {
    let dir = update_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // a fresh start: no restart unless asked for again
    let _ = std::fs::remove_file(restart_flag());
    let q = |p: &Path| format!("'{}'", p.display().to_string().replace('\'', "'\\''"));
    let script = format!(
        r#"#!/bin/sh
# Kernship update: waits for Kernship to end, then replaces it with the prepared version.
PID={pid}
APP={app}
NEW={new}
FLAG={flag}
while kill -0 "$PID" 2>/dev/null; do sleep 0.5; done
[ -d "$NEW" ] || exit 0
OLD="$APP.previous"
rm -rf "$OLD"
mv "$APP" "$OLD" || exit 1
if ditto "$NEW" "$APP"; then
    rm -rf "$OLD" "$(dirname "$NEW")"
else
    rm -rf "$APP"
    mv "$OLD" "$APP"
fi
xattr -dr com.apple.quarantine "$APP" 2>/dev/null
if [ -f "$FLAG" ]; then
    rm -f "$FLAG"
    open "$APP"
fi
rm -f "$0"
"#,
        pid = std::process::id(),
        app = q(app),
        new = q(new_app),
        flag = q(&restart_flag()),
    );
    let path = dir.join("install.sh");
    std::fs::write(&path, script).map_err(|e| e.to_string())?;
    Command::new("/bin/sh")
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The public repository that holds the releases (the source repository is private).
pub const REPO: &str = "brainupstudio/kernship";

/// Where people download a new version, with the install command that avoids the warning.
pub const DOWNLOAD_PAGE: &str = "https://github.com/brainupstudio/kernship/releases/latest";

/// While Kernship stays open (it can run in the background for days), it asks again this often.
pub const CHECK_EVERY_SECS: i64 = 24 * 60 * 60;

/// The latest published version ("0.2.0", without the "v"), from the redirect of
/// github.com/<repo>/releases/latest to .../releases/tag/v<version>, the way install.sh finds
/// it. One HEAD request whose redirect is not followed: no page is downloaded, and unlike the
/// GitHub API (60 requests an hour per address without a login, soon used up on an office
/// network) it has no hourly limit.
pub fn latest() -> Result<String, String> {
    let unreadable = || "GitHub sent an unreadable answer.".to_string();
    // No -L: the 302 and its Location header are the answer.
    let out = Command::new("curl")
        .args(["-sS", "-I", "--max-time", "15", "-o", "/dev/null", "-w", "%{http_code} %{redirect_url}"])
        .args(["-A", &format!("Kernship/{CURRENT}")])
        .arg(format!("https://github.com/{REPO}/releases/latest"))
        .output()
        .map_err(|_| "No connection to GitHub.".to_string())?;
    if !out.status.success() {
        return Err("No connection to GitHub.".into());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let (code, location) = text.trim().split_once(' ').unwrap_or((text.trim(), ""));
    if !code.starts_with('3') {
        return Err("GitHub did not answer as expected.".into());
    }
    version_from_url(location).ok_or_else(unreadable)
}

/// "https://github.com/o/r/releases/tag/v0.2.0" -> "0.2.0"; anything else (for instance the
/// releases list, where GitHub sends you when there is no release yet) -> None.
fn version_from_url(url: &str) -> Option<String> {
    let path = url.trim().split(['?', '#']).next()?.trim_end_matches('/');
    let (head, tag) = path.rsplit_once('/')?;
    if !head.ends_with("/releases/tag") {
        return None;
    }
    let version = tag.trim().trim_start_matches(['v', 'V']).to_string();
    parse(&version)?;
    Some(version)
}

/// "1.2.3" (or "1.2") as numbers; anything after a "-" or "+" is ignored.
fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let core = v.trim().trim_start_matches(['v', 'V']).split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
    let major = parts.next()??;
    let minor = parts.next().unwrap_or(Some(0))?;
    let patch = parts.next().unwrap_or(Some(0))?;
    Some((major, minor, patch))
}

/// `latest` is a later version than `current`.
pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse(latest), parse(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

/// A check running in the background, and how the last one asked for by hand went; then the
/// download of the version found, and whether it waits to be installed.
#[derive(Default)]
pub struct Checker {
    pub rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    /// The check was asked for with "Check Now", so its error is worth showing.
    pub asked: bool,
    pub error: Option<String>,
    pub download: Option<(String, std::sync::mpsc::Receiver<Result<PathBuf, String>>)>,
    /// The version prepared, with the script waiting for Kernship to end.
    pub ready: Option<String>,
    pub install_error: Option<String>,
    /// When the last check started, so a long-running Kernship asks again once a day.
    pub last_try: Option<std::time::Instant>,
    /// The ready version whose "Restart now?" question was already asked in this run.
    pub prompted: Option<String>,
}

impl Checker {
    pub fn checking(&self) -> bool {
        self.rx.is_some()
    }

    /// The version being downloaded.
    pub fn downloading(&self) -> Option<&str> {
        self.download.as_ref().map(|(v, _)| v.as_str())
    }

    pub fn start(&mut self, asked: bool) {
        if self.checking() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(latest());
        });
        self.rx = Some(rx);
        self.last_try = Some(std::time::Instant::now());
        self.asked = asked;
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_version_from_the_release_address() {
        assert_eq!(version_from_url("https://github.com/o/r/releases/tag/v0.1.2").as_deref(), Some("0.1.2"));
        assert_eq!(version_from_url("https://github.com/o/r/releases/tag/0.10.0/").as_deref(), Some("0.10.0"));
        assert_eq!(version_from_url("https://github.com/o/r/releases"), None);
        assert_eq!(version_from_url("https://github.com/o/r/releases/tag/latest-build"), None);
        assert_eq!(version_from_url(""), None);
    }

    /// Asks GitHub for real: cargo test --release -- --ignored latest_release --nocapture
    #[test]
    #[ignore]
    fn latest_release_from_github() {
        let v = latest().expect("an answer from GitHub");
        assert!(parse(&v).is_some(), "{v}");
        println!("latest: {v}");
    }

    #[test]
    fn finds_the_checksum_of_a_file() {
        let sums = "abc123  Kernship-0.2.2-arm64.dmg\nDEF456 *Kernship-0.2.2-x86_64.dmg\n";
        assert_eq!(expected_sum(sums, "Kernship-0.2.2-arm64.dmg").as_deref(), Some("abc123"));
        assert_eq!(expected_sum(sums, "Kernship-0.2.2-x86_64.dmg").as_deref(), Some("def456"));
        assert_eq!(expected_sum(sums, "Kernship-0.2.2-arm64.zip"), None);
    }

    /// Downloads and prepares the latest release for real, in the update folder:
    /// cargo test --release -- --ignored prepares_a_release --nocapture
    #[test]
    #[ignore]
    fn prepares_a_release() {
        let v = latest().expect("an answer from GitHub");
        let app = download(&v).expect("a prepared app");
        assert!(app.join("Contents/MacOS/Kernship").is_file(), "{}", app.display());
        println!("prepared: {}", app.display());
    }

    #[test]
    fn compares_versions_as_numbers() {
        assert!(is_newer("0.2.0", "0.1.2"));
        assert!(is_newer("v0.10.0", "0.9.3"));
        assert!(is_newer("1.0", "0.9.9"));
        assert!(!is_newer("0.1.2", "0.1.2"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        assert!(!is_newer("0.2.0-beta", "0.2.0"));
        assert!(is_newer("0.3.0-beta", "0.2.0"));
        assert!(!is_newer("garbage", "0.1.0"));
    }
}
