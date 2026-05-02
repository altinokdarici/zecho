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
        // Trigger the system prompt by requesting accessibility with prompt option
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
}

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(target_os = "macos"))]
pub fn is_accessibility_enabled() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_accessibility() {}
