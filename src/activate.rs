//! Turning fonts on and off for every program on the computer.
//!
//! macOS: Core Text's font manager registers the file for the whole login session, so design
//! programs (Photoshop, Illustrator, InDesign, Sketch, Figma, Affinity...) see it in their font
//! menus without the file being copied into a Fonts folder. Activated fonts are remembered and
//! registered again when the app starts.
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Scope {
    /// Available to all apps until logout (kCTFontManagerScopeSession = 3).
    Session,
    /// Remembered by the system across logins (kCTFontManagerScopePersistent = 2).
    Persistent,
}

#[cfg(target_os = "macos")]
mod mac {
    use core_foundation::base::TCFType;
    use core_foundation::url::CFURL;
    use core_foundation_sys::base::Boolean;
    use core_foundation_sys::error::CFErrorRef;
    use core_foundation_sys::url::CFURLRef;

    #[link(name = "CoreText", kind = "framework")]
    extern "C" {
        fn CTFontManagerRegisterFontsForURL(url: CFURLRef, scope: u32, error: *mut CFErrorRef) -> Boolean;
        fn CTFontManagerUnregisterFontsForURL(url: CFURLRef, scope: u32, error: *mut CFErrorRef) -> Boolean;
    }

    // kCTFontManagerErrorAlreadyRegistered = 105, kCTFontManagerErrorNotRegistered = 201
    fn code(err: CFErrorRef) -> isize {
        if err.is_null() {
            return 0;
        }
        unsafe {
            let e = core_foundation::error::CFError::wrap_under_create_rule(err);
            e.code()
        }
    }

    pub fn register(path: &std::path::Path, scope: u32) -> Result<(), String> {
        let url = CFURL::from_path(path, false).ok_or("bad path")?;
        let mut err: CFErrorRef = std::ptr::null_mut();
        let ok = unsafe { CTFontManagerRegisterFontsForURL(url.as_concrete_TypeRef(), scope, &mut err) };
        let c = code(err);
        if ok != 0 || c == 105 {
            return Ok(());
        }
        // A file registered in another
        // scope is unregistered first, then registered again with the user scope.
        for other in [1u32, 3, 2] {
            let mut e2: CFErrorRef = std::ptr::null_mut();
            unsafe { CTFontManagerUnregisterFontsForURL(url.as_concrete_TypeRef(), other, &mut e2) };
            let _ = code(e2);
        }
        let mut err: CFErrorRef = std::ptr::null_mut();
        let ok = unsafe { CTFontManagerRegisterFontsForURL(url.as_concrete_TypeRef(), scope, &mut err) };
        let c = code(err);
        if ok != 0 || c == 105 {
            Ok(())
        } else {
            Err(format!("activation refused (Core Text error {c})"))
        }
    }

    pub fn unregister(path: &std::path::Path, scope: u32) -> Result<(), String> {
        let url = CFURL::from_path(path, false).ok_or("bad path")?;
        let mut err: CFErrorRef = std::ptr::null_mut();
        let ok = unsafe { CTFontManagerUnregisterFontsForURL(url.as_concrete_TypeRef(), scope, &mut err) };
        let c = code(err);
        if ok != 0 || c == 201 {
            Ok(())
        } else {
            Err(format!("deactivation refused (Core Text error {c})"))
        }
    }
}

fn scope_value(s: Scope) -> u32 {
    match s {
        Scope::Session => 3,
        Scope::Persistent => 2,
    }
}

pub fn activate(path: &Path, scope: Scope) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        mac::register(path, scope_value(scope))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = scope_value(scope);
        // Elsewhere: copy into the user's font folder.
        let dir = dirs::font_dir().ok_or("no font folder")?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::copy(path, dir.join(path.file_name().unwrap())).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn deactivate(path: &Path, scope: Scope) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        mac::unregister(path, scope_value(scope))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = scope_value(scope);
        let dir = dirs::font_dir().ok_or("no font folder")?;
        let _ = std::fs::remove_file(dir.join(path.file_name().unwrap()));
        Ok(())
    }
}
