#[cfg(target_os = "macos")]
mod macos {
    pub fn is_accessibility_enabled() -> bool {
        unsafe {
            extern "C" {
                fn AXIsProcessTrusted() -> bool;
            }
            AXIsProcessTrusted()
        }
    }

    pub fn prompt_accessibility() {
        unsafe {
            extern "C" {
                fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
            }
            use core_foundation::base::TCFType;
            use core_foundation::boolean::CFBoolean;
            use core_foundation::dictionary::CFDictionary;
            use core_foundation::string::CFString;

            let key = CFString::new("AXTrustedCheckOptionPrompt");
            let value = CFBoolean::true_value();
            let options = CFDictionary::from_CFType_pairs(&[(key, value)]);
            AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as *const _);
        }
    }

    pub fn is_microphone_enabled() -> bool {
        // Check AVCaptureDevice authorization status without triggering a prompt
        // Uses NSAppleScript to query status passively
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("use framework \"AVFoundation\"\nset status to current application's AVCaptureDevice's authorizationStatusForMediaType:(current application's AVMediaTypeAudio)\nreturn status as integer")
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let status = String::from_utf8_lossy(&o.stdout).trim().to_string();
                status == "3" // 3 = authorized
            }
            _ => false,
        }
    }

    pub fn open_mic_settings() {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
            .spawn();
    }
}

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(target_os = "macos"))]
pub fn is_microphone_enabled() -> bool { true }

#[cfg(not(target_os = "macos"))]
pub fn open_mic_settings() {}

#[cfg(not(target_os = "macos"))]
pub fn is_accessibility_enabled() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_accessibility() {}
