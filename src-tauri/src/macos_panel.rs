#[cfg(target_os = "macos")]
pub fn make_panel(window: &tauri::WebviewWindow) {
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
    use cocoa::base::{id, BOOL, YES, NO};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};

    let ns_window: id = window.ns_window().unwrap() as id;

    unsafe {
        // Keep the window visible when the app is not active
        ns_window.setHidesOnDeactivate_(NO);

        // Accept mouse moved events
        let _: () = msg_send![ns_window, setAcceptsMouseMovedEvents: YES];

        // Float above other windows
        let _: () = msg_send![ns_window, setLevel: 5i64]; // kCGFloatingWindowLevel

        // Join all spaces
        ns_window.setCollectionBehavior_(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle,
        );

        // Make the webview's NSWindow accept first mouse — this is the key fix.
        // By default, clicking an unfocused NSWindow first activates it, THEN processes
        // the click. acceptsFirstMouse makes it process the click immediately.
        // We achieve this by adding a tracking area that covers the entire window content.
        let content_view: id = msg_send![ns_window, contentView];
        let bounds: cocoa::foundation::NSRect = msg_send![content_view, bounds];

        // Create NSTrackingArea with mouse moved/entered/exited events, active always
        let tracking_opts: u64 =
            0x01   // NSTrackingMouseEnteredAndExited
            | 0x02  // NSTrackingMouseMoved
            | 0x80  // NSTrackingActiveAlways (receive events even when not key window)
            | 0x08; // NSTrackingInVisibleRect

        let tracking_class = Class::get("NSTrackingArea").unwrap();
        let tracking_area: id = msg_send![tracking_class, alloc];
        let tracking_area: id = msg_send![
            tracking_area,
            initWithRect: bounds
            options: tracking_opts
            owner: content_view
            userInfo: cocoa::base::nil
        ];

        let _: () = msg_send![content_view, addTrackingArea: tracking_area];
    }
}

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

#[cfg(not(target_os = "macos"))]
pub fn make_panel(_window: &tauri::WebviewWindow) {}
