#[cfg(target_os = "macos")]
pub fn make_panel(window: &tauri::WebviewWindow) {
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
    use cocoa::base::id;
    use objc::msg_send;
    use objc::sel;
    use objc::sel_impl;

    let ns_window: id = window.ns_window().unwrap() as id;
    unsafe {
        // Accept mouse events without becoming the key window
        let _: () = objc::msg_send![ns_window, setAcceptsMouseMovedEvents: true];

        // Float above everything (level 3 = floating, +1 to be above other floating windows)
        let _: () = objc::msg_send![ns_window, setLevel: 3i64 + 1];

        // Join all spaces, don't appear in app switcher
        ns_window.setCollectionBehavior_(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle,
        );

        // NSNonactivatingPanelMask = 1 << 7 = 128
        // This makes the window receive mouse events without stealing focus from other apps
        let current_mask: u64 = objc::msg_send![ns_window, styleMask];
        let _: () = objc::msg_send![ns_window, setStyleMask: current_mask | (1u64 << 7)];

        // Also set ignoresMouseEvents to NO explicitly
        let _: () = objc::msg_send![ns_window, setIgnoresMouseEvents: false];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn make_panel(_window: &tauri::WebviewWindow) {}
