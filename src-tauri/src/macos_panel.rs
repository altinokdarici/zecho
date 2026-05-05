#[cfg(target_os = "macos")]
use tauri::{Emitter, Manager};
#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, TrackingAreaOptions,
    WebviewWindowExt,
};

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(ZechoPanel {
        config: {
            can_become_main_window: false,
            can_become_key_window: true,
            becomes_key_only_if_needed: true,
            is_floating_panel: true
        }
        with: {
            tracking_area: {
                options: TrackingAreaOptions::new()
                    .active_always()
                    .mouse_entered_and_exited()
                    .mouse_moved()
                    .cursor_update(),
                auto_resize: true
            }
        }
    })

    panel_event!(ZechoPanelEventHandler {})
}

#[cfg(target_os = "macos")]
pub fn make_panel(app: &tauri::App) {
    use tauri::Manager;

    let window = match app.get_webview_window("pill") {
        Some(w) => w,
        None => {
            eprintln!("Pill window not found for NSPanel conversion");
            return;
        }
    };

    let panel = match window.to_panel::<ZechoPanel>() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to convert pill to NSPanel: {:?}", e);
            return;
        }
    };

    let handler = ZechoPanelEventHandler::new();

    // Make key on mouse enter so hover/click work
    let handle = app.handle().clone();
    handler.on_mouse_entered(move |_event| {
        if let Ok(p) = handle.get_webview_panel("pill") {
            p.make_key_window();
        }
        handle.emit("pill-hover", true).ok();
    });

    // Resign key on mouse exit so the previous app regains focus
    let handle = app.handle().clone();
    handler.on_mouse_exited(move |_event| {
        if let Ok(p) = handle.get_webview_panel("pill") {
            p.resign_key_window();
        }
        handle.emit("pill-hover", false).ok();
    });

    panel.set_level(PanelLevel::Floating.value());
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .full_screen_auxiliary()
            .can_join_all_spaces()
            .into(),
    );
    panel.set_hides_on_deactivate(false);
    panel.set_works_when_modal(true);
    panel.set_corner_radius(0.0);
    panel.set_opaque(false);
    panel.set_has_shadow(false);
    panel.set_event_handler(Some(handler.as_ref()));
}

#[cfg(not(target_os = "macos"))]
pub fn make_panel(_app: &tauri::App) {}

// Tracks whether the dock icon should be visible (true = settings window is open).
#[cfg(target_os = "macos")]
static SETTINGS_OPEN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(target_os = "macos")]
pub fn set_dock_visible(visible: bool) {
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};
    SETTINGS_OPEN.store(visible, std::sync::atomic::Ordering::Relaxed);
    unsafe {
        let ns_app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        // 0 = Regular (shows in Dock), 1 = Accessory (no Dock icon)
        let policy: i64 = if visible { 0 } else { 1 };
        let _: () = msg_send![ns_app, setActivationPolicy: policy];
        if visible {
            let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_dock_visible(_visible: bool) {}

/// Swizzles `NSApplication -setActivationPolicy:` so any call that tries to
/// switch to Regular (0) is silently redirected to Accessory (1) unless
/// `SETTINGS_OPEN` is true.  Must be called before `run()` — ideally in
/// `main()` — so Tauri's own initialization calls are intercepted too.
#[cfg(target_os = "macos")]
pub fn swizzle_activation_policy() {
    use objc::runtime::{
        class_addMethod, class_getInstanceMethod, method_exchangeImplementations, Object, Sel,
    };
    use objc::{class, msg_send, sel, sel_impl};
    use std::os::raw::c_char;
    use std::sync::Once;

    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        // Replacement implementation: block Regular policy when settings is closed.
        extern "C" fn swizzled(this: &mut Object, _cmd: Sel, policy: i64) -> bool {
            let effective = if policy == 0
                && !SETTINGS_OPEN.load(std::sync::atomic::Ordering::Relaxed)
            {
                1i64 // force Accessory
            } else {
                policy
            };
            unsafe { msg_send![this, __zechoSetActivationPolicy: effective] }
        }

        let cls = class!(NSApplication);

        // Add a new selector that will hold the original IMP after the swap.
        let fn_ptr: unsafe extern "C" fn() =
            std::mem::transmute(swizzled as unsafe extern "C" fn(&mut Object, Sel, i64) -> bool);
        let added = class_addMethod(
            cls as *const _ as *mut _,
            sel!(__zechoSetActivationPolicy:),
            fn_ptr,
            b"B@:q\0".as_ptr() as *const c_char,
        );

        if !added {
            return;
        }

        let orig = class_getInstanceMethod(cls as *const _ as *mut _, sel!(setActivationPolicy:));
        let swiz =
            class_getInstanceMethod(cls as *const _ as *mut _, sel!(__zechoSetActivationPolicy:));

        if !orig.is_null() && !swiz.is_null() {
            method_exchangeImplementations(orig as *mut _, swiz as *mut _);
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub fn swizzle_activation_policy() {}

/// Kept for compatibility; no longer needed now that we swizzle.
#[cfg(target_os = "macos")]
pub fn install_activation_observer() {}
#[cfg(not(target_os = "macos"))]
pub fn install_activation_observer() {}
