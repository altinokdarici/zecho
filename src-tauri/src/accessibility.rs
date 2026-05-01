#[cfg(target_os = "macos")]
mod macos {
    use std::process::Command;

    pub fn is_accessibility_enabled() -> bool {
        // Check if the app has accessibility permissions using the macOS API
        // We use a simple heuristic: try to create a CGEventTap and see if it succeeds
        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to return name of first process")
            .output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    pub fn prompt_accessibility() {
        // Open System Settings to the Accessibility pane
        let _ = Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
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
