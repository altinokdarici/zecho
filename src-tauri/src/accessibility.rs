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
        // Try to enumerate audio input devices — if mic is denied, this returns no devices
        match std::panic::catch_unwind(|| {
            use cpal::traits::{DeviceTrait, HostTrait};
            let host = cpal::default_host();
            host.default_input_device()
                .and_then(|d| d.default_input_config().ok())
                .is_some()
        }) {
            Ok(result) => result,
            Err(_) => false,
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
