mod accessibility;
mod audio;
pub mod cleanup;
mod history;
mod hotkey;
mod macos_panel;
mod models;
pub mod settings;
mod transcribe;

use audio::AudioRecorder;
use cleanup::TextCleaner;
use history::{HistoryItem, HistoryStore};
use models::ModelStatus;
use settings::Settings;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use transcribe::Transcriber;

struct AppState {
    recorder: Mutex<AudioRecorder>,
    transcriber: Mutex<Transcriber>,
    cleaner: Mutex<TextCleaner>,
    history: Mutex<HistoryStore>,
    settings: Mutex<Settings>,
}

#[tauri::command]
fn start_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    recorder.start()
}

#[tauri::command]
fn get_audio_level(state: tauri::State<'_, AppState>) -> f32 {
    state
        .recorder
        .lock()
        .map(|r| r.rms_level())
        .unwrap_or(0.0)
}

#[tauri::command]
fn stop_recording(state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    let samples = recorder.stop();
    drop(recorder);

    if samples.is_empty() {
        return Err("No audio recorded".to_string());
    }

    let app_handle = app.clone();
    std::thread::spawn(move || {
        let state: tauri::State<'_, AppState> = app_handle.state();
        match process_recording(&state, samples) {
            Ok(text) => {
                app_handle.emit("transcription-complete", &text).ok();
            }
            Err(e) => {
                eprintln!("Processing error: {}", e);
                app_handle.emit("transcription-error", &e).ok();
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn cancel_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    recorder.stop();
    Ok(())
}

fn process_recording(state: &AppState, samples: Vec<f32>) -> Result<String, String> {
    use std::time::Instant;

    let t0 = Instant::now();
    let transcriber = state.transcriber.lock().map_err(|e| e.to_string())?;
    let raw_text = transcriber.transcribe(&samples)?;
    drop(transcriber);
    let transcribe_ms = t0.elapsed().as_millis() as u64;

    if raw_text.is_empty() {
        return Err("No speech detected".to_string());
    }

    let t1 = Instant::now();
    let cleaner = state.cleaner.lock().map_err(|e| e.to_string())?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let text = match cleaner.clean(
        &raw_text,
        &settings.writing_style,
        &settings.cleanup_level,
        settings.custom_prompt.as_deref(),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Cleanup error: {}, using raw text", e);
            raw_text.clone()
        }
    };
    drop(cleaner);
    drop(settings);
    let cleanup_ms = t1.elapsed().as_millis() as u64;

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&text).map_err(|e| e.to_string())?;

    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    history.add(text.clone(), raw_text, transcribe_ms, cleanup_ms);
    drop(history);

    let auto_paste = state.settings.lock().map(|s| s.auto_paste).unwrap_or(false);
    if auto_paste {
        simulate_paste();
    }

    Ok(text)
}

fn simulate_paste() {
    #[cfg(target_os = "macos")]
    {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let _ = std::process::Command::new("osascript")
                .arg("-e")
                .arg("tell application \"System Events\" to keystroke \"v\" using command down")
                .output();
        });
    }
}

#[tauri::command]
fn get_history(state: tauri::State<'_, AppState>) -> Result<Vec<HistoryItem>, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    Ok(history.items())
}

#[tauri::command]
fn copy_history_item(id: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    let item = history.get(&id).ok_or("Item not found")?.clone();
    drop(history);
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&item.text).map_err(|e| e.to_string())?;
    Ok(item.text)
}

#[tauri::command]
fn delete_history_item(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    history.delete(&id);
    Ok(())
}

#[tauri::command]
fn clear_history(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    history.clear();
    Ok(())
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
fn update_settings(new_settings: Settings, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    let path = settings.path.clone();
    *settings = new_settings;
    settings.path = path;
    settings.save()
}

#[tauri::command]
fn list_models() -> Vec<ModelStatus> {
    models::list_models()
}

#[tauri::command]
fn download_model(model_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let info = models::get_model(&model_id).ok_or("Unknown model")?;

    std::thread::spawn({
        let info = info.clone();
        let app = app.clone();
        move || {
            app.emit("model-download-started", &info.id).ok();
            match models::download_model_blocking(&info) {
                Ok(_) => {
                    app.emit("model-download-complete", &info.id).ok();
                }
                Err(e) => {
                    app.emit("model-download-error", &e).ok();
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn load_cleanup_model(model_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let info = models::get_model(&model_id).ok_or("Unknown model")?;
    let path = models::model_path(info);
    if !path.exists() {
        return Err(format!("Model not downloaded: {}", info.name));
    }
    let mut cleaner = state.cleaner.lock().map_err(|e| e.to_string())?;
    cleaner.start_worker(&path)
}

#[tauri::command]
fn load_whisper_model_cmd(model_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let info = models::get_model(&model_id).ok_or("Unknown model")?;
    let path = models::model_path(info);
    if !path.exists() {
        return Err(format!("Model not downloaded: {}", info.name));
    }
    let mut transcriber = state.transcriber.lock().map_err(|e| e.to_string())?;
    transcriber.load_model(&path)
}

#[tauri::command]
fn save_pill_position(x_pct: f64, y_pct: f64, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.pill_x_pct = Some(x_pct);
    settings.pill_y_pct = Some(y_pct);
    settings.save()
}

#[tauri::command]
fn start_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn check_accessibility() -> bool {
    accessibility::is_accessibility_enabled()
}

#[tauri::command]
fn open_accessibility_settings() {
    accessibility::prompt_accessibility();
}

#[tauri::command]
fn start_fn_listener(app: tauri::AppHandle) {
    if accessibility::is_accessibility_enabled() {
        hotkey::start_fn_key_listener(app);
    }
}

#[tauri::command]
fn check_microphone() -> bool {
    accessibility::is_microphone_enabled()
}

#[tauri::command]
fn open_mic_settings() {
    accessibility::open_mic_settings();
}

#[tauri::command]
fn request_microphone() {
    accessibility::request_microphone();
}

#[derive(serde::Serialize, Clone)]
struct SetupStatus {
    whisper_ready: bool,
    cleanup_ready: bool,
    whisper_downloading: bool,
    cleanup_downloading: bool,
}

#[tauri::command]
fn check_setup() -> SetupStatus {
    let whisper = models::default_whisper_model();
    let cleanup_candidates = ["qwen25-1.5b", "qwen25-3b"];
    let cleanup_ready = cleanup_candidates
        .iter()
        .any(|id| models::get_model(id).map(|m| models::is_downloaded(m)).unwrap_or(false));

    SetupStatus {
        whisper_ready: models::is_downloaded(whisper),
        cleanup_ready,
        whisper_downloading: false,
        cleanup_downloading: false,
    }
}

#[tauri::command]
fn setup_download_models(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let whisper = models::default_whisper_model();
        if !models::is_downloaded(whisper) {
            app.emit("setup-progress", "Downloading speech model...").ok();
            match models::download_model_blocking(whisper) {
                Ok(_) => {
                    app.emit("setup-progress", "Speech model ready").ok();
                    {
                        let state = app.state::<AppState>();
                        let mut t = state.transcriber.lock().unwrap();
                        t.load_model(&models::model_path(whisper)).ok();
                    }
                }
                Err(e) => {
                    app.emit("setup-error", &format!("Failed to download speech model: {}", e)).ok();
                    return;
                }
            }
        }

        let cleanup_candidates = ["qwen25-1.5b", "qwen25-3b"];
        let mut cleanup_downloaded = false;
        for id in &cleanup_candidates {
            if let Some(info) = models::get_model(id) {
                if models::is_downloaded(info) {
                    cleanup_downloaded = true;
                    break;
                }
            }
        }
        if !cleanup_downloaded {
            if let Some(info) = models::get_model("qwen25-1.5b") {
                app.emit("setup-progress", "Downloading cleanup model...").ok();
                match models::download_model_blocking(info) {
                    Ok(_) => {
                        app.emit("setup-progress", "Cleanup model ready").ok();
                    }
                    Err(e) => {
                        app.emit("setup-error", &format!("Failed to download cleanup model: {}", e)).ok();
                        return;
                    }
                }
            }
        }

        // Start the cleanup worker
        for id in &cleanup_candidates {
            if let Some(info) = models::get_model(id) {
                let path = models::model_path(info);
                if path.exists() {
                    {
                        let state = app.state::<AppState>();
                        let mut c = state.cleaner.lock().unwrap();
                        c.start_worker(&path).ok();
                    }
                    break;
                }
            }
        }

        app.emit("setup-complete", ()).ok();
    });
}

#[tauri::command]
fn hide_setup(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("setup") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn complete_setup(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.setup_complete = true;
    settings.save()
}

#[tauri::command]
async fn check_for_updates(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_updater::UpdaterExt;
    match app.updater().map_err(|e| e.to_string())?.check().await {
        Ok(Some(update)) => Ok(Some(update.version)),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn create_tray_icon(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let icon = {
        let png_data = include_bytes!("../icons/tray_icon.png");
        let decoder = png::Decoder::new(std::io::Cursor::new(png_data));
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
        buf.truncate(info.buffer_size());
        tauri::image::Image::new_owned(buf, info.width, info.height)
    };

    let show = MenuItem::with_id(app, "show", "Show Zecho", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings_item, &quit])?;

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .tooltip("Zecho")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("pill") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("settings") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn save_pill_position_from_window(window: &tauri::WebviewWindow, state: &AppState) {
    let pos = match window.outer_position() {
        Ok(p) => p,
        _ => return,
    };
    let size = match window.outer_size() {
        Ok(s) => s,
        _ => return,
    };
    let monitor = match window.current_monitor() {
        Ok(Some(m)) => m,
        _ => return,
    };
    let screen = monitor.size();
    if screen.width == 0 || screen.height == 0 {
        return;
    }
    let x_pct = pos.x as f64 / screen.width as f64;
    let y_pct = (pos.y as f64 + size.height as f64) / screen.height as f64;

    if let Ok(mut settings) = state.settings.lock() {
        settings.pill_x_pct = Some(x_pct);
        settings.pill_y_pct = Some(y_pct);
        settings.save().ok();
    }
}

fn position_pill_window(window: &tauri::WebviewWindow, state: &AppState) {
    let monitor = match window.primary_monitor() {
        Ok(Some(m)) => m,
        _ => match window.current_monitor() {
            Ok(Some(m)) => m,
            _ => return,
        },
    };
    let screen = monitor.size();
    let scale = monitor.scale_factor();
    let win_h = window.outer_size().map(|s| s.height as i32).unwrap_or((420.0 * scale) as i32);
    let win_w = window.outer_size().map(|s| s.width as i32).unwrap_or((280.0 * scale) as i32);

    let settings = state.settings.lock().ok();
    let saved = settings.as_ref().and_then(|s| {
        match (s.pill_x_pct, s.pill_y_pct) {
            (Some(xp), Some(yp)) if xp >= 0.0 && xp <= 1.5 && yp >= 0.0 && yp <= 1.5 => {
                Some((xp, yp))
            }
            _ => None,
        }
    });

    let (x, y) = if let Some((x_pct, y_pct)) = saved {
        let x = (x_pct * screen.width as f64) as i32;
        let bottom_y = (y_pct * screen.height as f64) as i32;
        (x, bottom_y - win_h)
    } else {
        let x = (screen.width as i32 - win_w) / 2;
        let y = screen.height as i32 - win_h - (60.0 * scale) as i32;
        (x, y)
    };

    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .ok();
}

fn register_global_shortcut(app: &tauri::App) {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    app.global_shortcut().on_shortcut(shortcut, |app_handle, _shortcut, _event| {
        let _ = app_handle.emit("toggle-recording", ());
    }).ok();
}

fn init_models_async(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let state = app_handle.state::<AppState>();
        let whisper_model = models::default_whisper_model();
        let whisper_path = models::model_path(whisper_model);
        if whisper_path.exists() {
            if let Ok(mut t) = state.transcriber.lock() {
                t.load_model(&whisper_path).ok();
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("zecho");
    std::fs::create_dir_all(&data_dir).ok();
    std::fs::create_dir_all(models::model_dir()).ok();

    let state = AppState {
        recorder: Mutex::new(AudioRecorder::new()),
        transcriber: Mutex::new(Transcriber::new()),
        cleaner: Mutex::new(TextCleaner::new()),
        history: Mutex::new(HistoryStore::load(&data_dir)),
        settings: Mutex::new(Settings::load(&data_dir)),
    };

    // Models loaded async in setup — see init_models_async

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_nspanel::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            cancel_recording,
            get_audio_level,
            start_drag,
            get_history,
            copy_history_item,
            delete_history_item,
            clear_history,
            open_settings,
            get_settings,
            update_settings,
            list_models,
            download_model,
            load_cleanup_model,
            load_whisper_model_cmd,
            check_accessibility,
            open_accessibility_settings,
            start_fn_listener,
            check_microphone,
            open_mic_settings,
            request_microphone,
            check_setup,
            setup_download_models,
            complete_setup,
            hide_setup,
            check_for_updates,
        ])
        .setup(|app| {
            create_tray_icon(app).ok();
            register_global_shortcut(app);
            macos_panel::make_panel(app);
            init_models_async(app.handle().clone());

            // Only start FN key listener if accessibility is already granted
            // Otherwise, FRE will guide the user to enable it
            if accessibility::is_accessibility_enabled() {
                hotkey::start_fn_key_listener(app.handle().clone());
            }

            // Show setup window if anything is missing
            {
                let whisper_ready = models::is_downloaded(models::default_whisper_model());
                let cleanup_ready = ["qwen25-1.5b", "qwen25-3b"].iter()
                    .any(|id| models::get_model(id).map(|m| models::is_downloaded(m)).unwrap_or(false));
                let accessibility = accessibility::is_accessibility_enabled();
                if !whisper_ready || !cleanup_ready || !accessibility {
                    if let Some(window) = app.get_webview_window("setup") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
            }

            // Start cleanup worker if model already downloaded
            let cleanup_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(3));
                let cleanup_candidates = ["qwen25-1.5b", "qwen25-3b"];
                for id in &cleanup_candidates {
                    if let Some(info) = models::get_model(id) {
                        let path = models::model_path(info);
                        if path.exists() {
                            let state = cleanup_handle.state::<AppState>();
                            if let Ok(mut c) = state.cleaner.lock() {
                                c.start_worker(&path).ok();
                            }
                            return;
                        }
                    }
                }
            });

            // Delay positioning so the window is fully created first
            let handle = app.handle().clone();
            let handle2 = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(300));
                let state: tauri::State<'_, AppState> = handle.state();
                if let Some(window) = handle.get_webview_window("pill") {
                    position_pill_window(&window, &state);
                }
            });

            // Listen for window move events to persist position
            if let Some(window) = app.get_webview_window("pill") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Moved(_) = event {
                        let state: tauri::State<'_, AppState> = handle2.state();
                        if let Some(win) = handle2.get_webview_window("pill") {
                            save_pill_position_from_window(&win, &state);
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running zecho");
}
