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
