use tauri::{AppHandle, Manager};
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use std::thread;
use std::time::Duration;

#[cfg(windows)]
mod keyboard_hook;

#[tauri::command]
fn paste_selection(app: AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    window.hide().unwrap();

    // Small delay to ensure focus returns to the previous app
    thread::sleep(Duration::from_millis(150));

    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    // Simulate Ctrl + V
    // Note: This matches Linux/Windows standard. Mac uses Meta+V (Command).
    // PRD specifically asked for Windows (Ctrl+V).

    // Using enigo 0.3.0 / 0.6.1 API
    let _ = enigo.key(Key::Control, Direction::Press);
    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|_app| {
            #[cfg(windows)]
            keyboard_hook::start(_app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![paste_selection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
