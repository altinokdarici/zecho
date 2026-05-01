mod audio;
mod cleanup;
mod history;
mod settings;
mod transcribe;

use audio::AudioRecorder;
use cleanup::TextCleaner;
use history::{HistoryItem, HistoryStore};
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
fn stop_recording(state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<String, String> {
    let text = finish_recording(&state)?;
    app.emit("transcription-complete", &text).ok();
    Ok(text)
}

#[tauri::command]
fn cancel_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    recorder.stop();
    Ok(())
}

fn finish_recording(state: &AppState) -> Result<String, String> {
    let recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    let samples = recorder.stop();
    drop(recorder);

    if samples.is_empty() {
        return Err("No audio recorded".to_string());
    }

    let transcriber = state.transcriber.lock().map_err(|e| e.to_string())?;
    let raw_text = transcriber.transcribe(&samples)?;
    drop(transcriber);

    if raw_text.is_empty() {
        return Err("No speech detected".to_string());
    }

    let cleaner = state.cleaner.lock().map_err(|e| e.to_string())?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let text = cleaner.clean(
        &raw_text,
        &settings.writing_style,
        &settings.cleanup_level,
        settings.custom_prompt.as_deref(),
    )?;
    drop(cleaner);
    drop(settings);

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&text).map_err(|e| e.to_string())?;

    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    history.add(text.clone(), raw_text);

    Ok(text)
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
fn get_model_status(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let transcriber = state.transcriber.lock().map_err(|e| e.to_string())?;
    Ok(transcriber.is_loaded())
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

    let size = 22u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let cx = size as f32 / 2.0;
            let cy = size as f32 / 2.0;
            let dist = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            if dist < cx - 2.0 {
                rgba[idx] = 255;
                rgba[idx + 1] = 255;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 220;
            }
        }
    }

    let show = MenuItem::with_id(app, "show", "Show Zecho", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings_item, &quit])?;

    TrayIconBuilder::new()
        .icon(tauri::image::Image::new_owned(rgba, size, size))
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

fn position_pill(app: &tauri::App) {
    if let Some(window) = app.get_webview_window("pill") {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen = monitor.size();
            let scale = monitor.scale_factor();
            let win_w = (280.0 * scale) as i32;
            let win_h = (56.0 * scale) as i32;
            let x = (screen.width as i32 - win_w) / 2;
            let y = screen.height as i32 - win_h - (60.0 * scale) as i32;
            window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
                .ok();
        }
    }
}

fn load_whisper_model(transcriber: &Mutex<Transcriber>) {
    let model_path = Transcriber::default_model_path();
    if model_path.exists() {
        if let Ok(mut t) = transcriber.lock() {
            if let Err(e) = t.load_model(&model_path) {
                eprintln!("Failed to load whisper model: {}", e);
            } else {
                println!("Whisper model loaded from {}", model_path.display());
            }
        }
    } else {
        eprintln!(
            "Whisper model not found at {}. Run scripts/download-models.sh or the app will download it on first launch.",
            model_path.display()
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("zecho");
    std::fs::create_dir_all(&data_dir).ok();
    std::fs::create_dir_all(Transcriber::model_dir()).ok();

    let transcriber = Mutex::new(Transcriber::new());

    // Load whisper model in background to not block startup
    load_whisper_model(&transcriber);

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            recorder: Mutex::new(AudioRecorder::new()),
            transcriber,
            cleaner: Mutex::new(TextCleaner::new()),
            history: Mutex::new(HistoryStore::load(&data_dir)),
            settings: Mutex::new(Settings::load(&data_dir)),
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            cancel_recording,
            get_history,
            copy_history_item,
            delete_history_item,
            clear_history,
            open_settings,
            get_settings,
            update_settings,
            get_model_status,
            check_for_updates,
        ])
        .setup(|app| {
            create_tray_icon(app).ok();
            position_pill(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running zecho");
}
