mod ai;
mod clipboard;
mod commands;
mod db;
mod embed;
mod storage;
mod types;

use crate::ai::AiClient;
use crate::clipboard::ClipboardWatcher;
use crate::commands::AppState;
use crate::db::Database;
use crate::embed::Embedder;
use crate::storage::SecureStore;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, _shortcut, _event| {})
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::NSApplication;
                use objc2_foundation::MainThreadMarker;
                if let Some(mtm) = MainThreadMarker::new() {
                    let ns_app = NSApplication::sharedApplication(mtm);
                    ns_app.setActivationPolicy(
                        objc2_app_kit::NSApplicationActivationPolicy::Accessory,
                    );
                }
            }

            let app_handle = app.handle().clone();
            let data_dir = app.path().app_data_dir().expect("no data dir");
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("buffbrain.db");
            let database = Arc::new(Database::new(db_path, 500).expect("db init"));
            let store = Arc::new(SecureStore::new());
            let ai = Arc::new(tokio::sync::Mutex::new(None));

            if let Ok(Some(key)) = store.get_api_key() {
                *ai.blocking_lock() = Some(AiClient::new(key));
            }

            let resource_dir = app.path().resource_dir().expect("no resource dir");
            let model_dir = resource_dir.join("resources");
            let embedder = match Embedder::new(&model_dir) {
                Ok(e) => {
                    eprintln!("[embed] model loaded successfully");
                    Some(Arc::new(e))
                }
                Err(e) => {
                    eprintln!("[embed] failed to load model: {e}");
                    None
                }
            };

            let show_tray = database
                .get_config("show_tray")
                .ok()
                .flatten()
                .map(|s| s == "true")
                .unwrap_or(true);

            let state = AppState {
                db: database.clone(),
                store: store.clone(),
                ai: ai.clone(),
                embedder,
                data_dir: data_dir.clone(),
                tray: Mutex::new(None),
                #[cfg(target_os = "macos")]
                previous_app: Arc::new(Mutex::new(None)),
            };
            app.manage(state);

            // Auto-start on first run (enable by default)
            if database.get_config("auto_start").ok().flatten().is_none() {
                database.set_config("auto_start", "true").ok();
                let _ = app.autolaunch().enable();
            }

            let window = app.get_webview_window("main").expect("no main window");
            commands::position_window_top_center(&window);
            #[cfg(debug_assertions)]
            {
                let dev_window = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = dev_window.show();
                    let _ = dev_window.set_focus();
                    dev_window.open_devtools();
                });
            }
            let _ = window.show();

            // Tray
            let show_item = MenuItem::with_id(app, "show", "Open BuffBrain", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &hide_item, &separator, &quit_item])?;

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))
                .expect("failed to load tray icon");
            eprintln!("[tray] icon loaded: {}x{}", tray_icon.width(), tray_icon.height());
            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .icon_as_template(false)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("BuffBrain - AI Clipboard Manager")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            commands::position_window_top_center(&w);
                            let _ = w.show();
                            let _ = w.set_focus();
                            let _ = app.emit("focus-search", ());
                        }
                    }
                    "hide" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                commands::position_window_top_center(&w);
                                let _ = w.show();
                                let _ = w.set_focus();
                                let _ = app.emit("focus-search", ());
                            }
                        }
                    }
                })
                .build(app)?;
            eprintln!("[tray] tray icon built successfully");
            tray.set_visible(show_tray).ok();
            *app.state::<AppState>().tray.lock().unwrap() = Some(tray.clone());

            // Global shortcut: Cmd+Shift+V (macOS) / Ctrl+Shift+V
            let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
            let handle_for_shortcut = app_handle.clone();
            let global_shortcut = app.global_shortcut();
            global_shortcut.on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    toggle_window(&handle_for_shortcut);
                }
            })?;

            // Window event listeners
            #[cfg(not(debug_assertions))]
            window.on_window_event({
                let handle = app_handle.clone();
                move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        if let Some(w) = handle.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                }
            });

            // Clipboard watcher
            let watcher = ClipboardWatcher::new(app_handle.clone());
            tauri::async_runtime::spawn(async move {
                watcher.run().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::semantic_search,
            commands::insert_clip,
            commands::insert_image_clip,
            commands::toggle_pin,
            commands::delete_clip,
            commands::clear_unpinned,
            commands::count_clips,
            commands::paste_clip,
            commands::hide_window,
            commands::show_window,
            commands::get_config,
            commands::set_config,
            commands::set_api_key,
            commands::has_api_key,
            commands::delete_api_key,
            commands::get_clipboard_text,
            commands::write_clipboard_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            #[cfg(target_os = "macos")]
            {
                // Capture the frontmost app BEFORE we take focus, so we can
                // restore focus to it later when pasting.
                if let Ok(output) = std::process::Command::new("osascript")
                    .args(&["-e", r#"tell application "System Events" to get name of first application process whose frontmost is true"#])
                    .output()
                {
                    if let Ok(name) = String::from_utf8(output.stdout) {
                        let trimmed = name.trim().to_string();
                        if !trimmed.is_empty() && trimmed != "BuffBrain" {
                            if let Some(state) = app.try_state::<AppState>() {
                                if let Ok(mut prev) = state.previous_app.lock() {
                                    *prev = Some(trimmed);
                                }
                            }
                        }
                    }
                }
            }
            commands::position_window_top_center(&window);
            let _ = window.show();
            let _ = window.set_focus();
            let _ = app.emit("focus-search", ());
        }
    }
}
