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
        // Don't check proactively — return unknown/true to avoid blocking
        // The FRE uses request_microphone() to trigger the system prompt on button click
        true
    }

    pub fn request_microphone() {
        // Trigger the system mic permission dialog by briefly accessing the mic
        std::thread::spawn(|| {
            use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
            let host = cpal::default_host();
            if let Some(device) = host.default_input_device() {
                if let Ok(config) = device.default_input_config() {
                    if let Ok(stream) = device.build_input_stream(
                        &config.into(),
                        |_data: &[f32], _: &cpal::InputCallbackInfo| {},
                        |_err| {},
                        None,
                    ) {
                        stream.play().ok();
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        // Stream drops here, releasing the mic
                    }
                }
            }
        });
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
