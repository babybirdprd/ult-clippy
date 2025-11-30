use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use std::thread;
use std::time::Duration;

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

fn toggle_window(app: &AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    if window.is_visible().unwrap() {
        window.hide().unwrap();
    } else {
        window.show().unwrap();
        window.set_focus().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(tauri::ipc::Scope::default(), None))
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, shortcut, event| {
             if event.state == ShortcutState::Pressed {
                if shortcut.matches(Modifiers::SUPER, Code::KeyV) {
                    toggle_window(app);
                }
            }
        }).build())
        .invoke_handler(tauri::generate_handler![paste_selection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
