use tauri::{AppHandle, Manager};
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use std::thread;
use std::time::Duration;

#[cfg(windows)]
mod keyboard_hook;

#[cfg(windows)]
fn disable_win_v_hotkey() -> Result<(), Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
    let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)?;
    
    // Set DisabledHotkeys to "V" to disable Win+V
    key.set_value("DisabledHotkeys", &"V")?;
    
    println!("Successfully disabled Win+V hotkey. Please restart Explorer or reboot for changes to take effect.");
    Ok(())
}

#[tauri::command]
fn paste_selection(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // Small delay to ensure focus returns to the previous app
    thread::sleep(Duration::from_millis(150));

    // Attempt to initialize Enigo
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        // Simulate Ctrl + V
        // Note: This matches Linux/Windows standard. Mac uses Meta+V (Command).
        // PRD specifically asked for Windows (Ctrl+V).
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        // Initialize autostart plugin with LaunchAgent (MacosLauncher)
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_clipboard::init())
        .setup(|_app| {
            #[cfg(windows)]
            {
                // Disable Win+V system hotkey
                if let Err(e) = disable_win_v_hotkey() {
                    eprintln!("Failed to disable Win+V hotkey: {}. You may need to disable it manually.", e);
                }
                
                // Start keyboard hook
                keyboard_hook::start(_app.handle().clone());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![paste_selection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
