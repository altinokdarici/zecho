#[cfg(target_os = "macos")]
mod macos {
    pub fn is_accessibility_enabled() -> bool {
        unsafe {
            // AXIsProcessTrusted() returns true if THIS process has accessibility permissions
            extern "C" {
                fn AXIsProcessTrusted() -> bool;
            }
            AXIsProcessTrusted()
        }
    }

    pub fn prompt_accessibility() {
        // Open Input Monitoring (where FN key permission lives)
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
            .spawn();
    }
}

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(target_os = "macos"))]
pub fn is_accessibility_enabled() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_accessibility() {}
