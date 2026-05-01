#[cfg(target_os = "macos")]
mod macos {
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use core_graphics::event::{
        CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
        CGEventTapPlacement, CGEventType,
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tauri::{AppHandle, Emitter};

    static FN_IS_DOWN: AtomicBool = AtomicBool::new(false);

    pub fn start_fn_key_listener(app_handle: AppHandle) {
        std::thread::spawn(move || {
            let tap = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![CGEventType::FlagsChanged],
                move |_proxy, _event_type, event| {
                    {
                        let flags = event.get_flags();
                        let fn_pressed = flags.contains(CGEventFlags::CGEventFlagSecondaryFn);
                        let was_down = FN_IS_DOWN.swap(fn_pressed, Ordering::SeqCst);

                        if fn_pressed && !was_down {
                            let _ = app_handle.emit("fn-key-down", ());
                        } else if !fn_pressed && was_down {
                            let _ = app_handle.emit("fn-key-up", ());
                        }
                    }
                    None
                },
            );

            match tap {
                Ok(tap) => {
                    unsafe {
                        let loop_source = tap.mach_port.create_runloop_source(0).unwrap();
                        let run_loop = CFRunLoop::get_current();
                        run_loop.add_source(&loop_source, kCFRunLoopCommonModes);
                        tap.enable();
                        CFRunLoop::run_current();
                    }
                }
                Err(()) => {
                    eprintln!(
                        "Failed to create CGEventTap. Grant Accessibility permissions: \
                         System Settings > Privacy & Security > Accessibility"
                    );
                }
            }
        });
    }
}

#[cfg(target_os = "macos")]
pub use macos::start_fn_key_listener;

#[cfg(not(target_os = "macos"))]
pub fn start_fn_key_listener(_app_handle: tauri::AppHandle) {
    eprintln!("FN key listener is only supported on macOS");
}
