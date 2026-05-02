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
        unsafe {
            #[link(name = "AVFoundation", kind = "framework")]
            extern "C" {}

            // Use AVCaptureDevice authorizationStatus — but simpler: just try to list audio devices
            // If mic access is denied, cpal won't find input devices
            use cpal::traits::{DeviceTrait, HostTrait};
            let host = cpal::default_host();
            if let Some(device) = host.default_input_device() {
                // If we can get a config, mic is accessible
                device.default_input_config().is_ok()
            } else {
                false
            }
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
