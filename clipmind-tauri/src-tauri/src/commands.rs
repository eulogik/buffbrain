use crate::ai::AiClient;
use crate::db::{detect_type, Database};
use crate::storage::SecureStore;
use crate::types::{AppConfig, Clip, ClipType, Theme};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::Mutex as AsyncMutex;

pub struct AppState {
    pub db: Arc<Database>,
    pub store: Arc<SecureStore>,
    pub ai: Arc<AsyncMutex<Option<AiClient>>>,
    pub data_dir: PathBuf,
}

#[tauri::command]
pub async fn get_clips(state: State<'_, AppState>) -> Result<Vec<Clip>, String> {
    state.db.get_clips().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn insert_clip(
    state: State<'_, AppState>,
    content: String,
    source: Option<String>,
) -> Result<Clip, String> {
    let kind = detect_type(&content);
    let kind_str = clip_type_to_str(&kind);
    let clip = state
        .db
        .insert_clip(&content, kind, source.as_deref(), None)
        .map_err(|e| e.to_string())?;
    if let Some(ai) = state.ai.lock().await.as_ref() {
        if let Ok(refined) = ai.categorize(&content).await {
            if clip_type_to_str(&refined) != kind_str {
                let _ = state.db.set_config("_last_type_override", &format!("{}:{}", clip.id, clip_type_to_str(&refined)));
            }
        }
    }
    Ok(clip)
}

#[tauri::command]
pub async fn insert_image_clip(
    state: State<'_, AppState>,
    thumbnail: String,
    source: Option<String>,
) -> Result<Clip, String> {
    let placeholder = format!("[Image {} bytes]", thumbnail.len());
    state
        .db
        .insert_clip(&placeholder, ClipType::Image, source.as_deref(), Some(&thumbnail))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_pin(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    state.db.toggle_pin(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clip(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_clip(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_unpinned(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn count_clips(state: State<'_, AppState>) -> Result<i64, String> {
    state.db.count().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn paste_clip<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    id: i64,
) -> Result<(), String> {
    let clips = state_get(&app).db.get_clips().map_err(|e| e.to_string())?;
    if let Some(clip) = clips.into_iter().find(|c| c.id == id) {
        if matches!(clip.clip_type, ClipType::Image) {
            if let Some(thumb) = &clip.thumbnail {
                if let Some(b64) = thumb.strip_prefix("data:image/png;base64,") {
                    if let Ok(bytes) = base64_decode(b64) {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let rgba = img.to_rgba8();
                            let (w, h) = (img.width(), img.height());
                            let img_struct = tauri::image::Image::new_owned(rgba.into_raw(), w, h);
                            let _ = app.clipboard().write_image(&img_struct);
                        }
                    }
                }
            }
        } else {
            app.clipboard()
                .write_text(clip.content.clone())
                .map_err(|e| e.to_string())?;
        }
    }
    let _ = window.hide();
    // Deactivate BuffBrain so macOS restores focus to the previous app,
    // then send Cmd+V to paste the clipboard contents there.
    #[cfg(target_os = "macos")]
    {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            // Cmd+H hides the entire app → macOS auto-activates the previous app
            // Then Cmd+V pastes into whatever is now frontmost
            let script = r#"tell application "System Events"
                key code 4 using {command down}
                delay 0.15
                keystroke "v" using command down
            end tell"#;
            match std::process::Command::new("osascript")
                .args(&["-e", script])
                .output()
            {
                Ok(output) => {
                    if !output.status.success() {
                        eprintln!("[paste] osascript failed: {:?}",
                            String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => eprintln!("[paste] osascript error: {}", e),
            }
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn show_window(window: WebviewWindow) -> Result<(), String> {
    position_window_top_center(&window);
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let theme = state
        .db
        .get_config("theme")
        .ok()
        .flatten()
        .and_then(|s| match s.as_str() {
            "dark" => Some(Theme::Dark),
            "light" => Some(Theme::Light),
            _ => Some(Theme::System),
        })
        .unwrap_or(Theme::System);
    let ai_enabled = state
        .db
        .get_config("ai_enabled")
        .ok()
        .flatten()
        .map(|s| s == "true")
        .unwrap_or(false);
    Ok(AppConfig {
        theme,
        ai_enabled,
        auto_hide: true,
        max_clips: 500,
    })
}

#[tauri::command]
pub async fn set_config(
    state: State<'_, AppState>,
    theme: Option<Theme>,
    ai_enabled: Option<bool>,
) -> Result<(), String> {
    if let Some(t) = theme {
        let s = match t {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::System => "system",
        };
        state.db.set_config("theme", s).map_err(|e| e.to_string())?;
    }
    if let Some(ai) = ai_enabled {
        state
            .db
            .set_config("ai_enabled", if ai { "true" } else { "false" })
            .map_err(|e| e.to_string())?;
        if ai {
            refresh_ai_client(&state).await;
        } else {
            *state.ai.lock().await = None;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn set_api_key(
    state: State<'_, AppState>,
    key: String,
) -> Result<(), String> {
    state
        .store
        .set_api_key(&key)
        .map_err(|e| e.to_string())?;
    refresh_ai_client(&state).await;
    Ok(())
}

#[tauri::command]
pub async fn has_api_key(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.store.get_api_key().map_err(|e| e.to_string())?.is_some())
}

#[tauri::command]
pub async fn delete_api_key(state: State<'_, AppState>) -> Result<(), String> {
    state.store.delete_api_key().map_err(|e| e.to_string())?;
    *state.ai.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn get_clipboard_text<R: Runtime>(app: AppHandle<R>) -> Result<String, String> {
    app.clipboard().read_text().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_clipboard_text<R: Runtime>(
    app: AppHandle<R>,
    text: String,
) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

pub fn position_window_top_center<R: Runtime>(window: &WebviewWindow<R>) {
    use tauri::{LogicalPosition, LogicalSize};
    if let Some(monitor) = window.current_monitor().ok().flatten() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let win_w = 720.0;
        let win_h = 460.0;
        let x = ((size.width as f64 / scale) - win_w) / 2.0;
        let y = (size.height as f64 / scale) * 0.10;
        let _ = window.set_size(LogicalSize::new(win_w, win_h));
        let _ = window.set_position(LogicalPosition::new(x, y));
    }
}

fn state_get<R: Runtime>(app: &AppHandle<R>) -> Arc<AppState> {
    let state: State<'_, AppState> = app.state();
    Arc::new(AppState {
        db: state.db.clone(),
        store: state.store.clone(),
        ai: state.ai.clone(),
        data_dir: state.data_dir.clone(),
    })
}

async fn refresh_ai_client(state: &State<'_, AppState>) {
    if let Ok(Some(key)) = state.store.get_api_key() {
        *state.ai.lock().await = Some(AiClient::new(key));
    }
}

fn clip_type_to_str(t: &ClipType) -> &'static str {
    match t {
        ClipType::Text => "text",
        ClipType::Code => "code",
        ClipType::Link => "link",
        ClipType::Image => "image",
    }
}

fn base64_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    use base64::{engine::general_purpose, Engine as _};
    Ok(general_purpose::STANDARD.decode(s)?)
}
