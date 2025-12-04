use std::sync::Mutex;
use std::thread;
use std::fs::OpenOptions;
use std::io::Write;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{HMODULE, HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LWIN, VK_RWIN, VK_V, GetAsyncKeyState};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, HC_ACTION, WM_KEYDOWN, WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP,
    DispatchMessageW, GetMessageW, TranslateMessage, HHOOK, MSG, KBDLLHOOKSTRUCT_FLAGS,
};
use std::sync::LazyLock;

// LLKHF_INJECTED flag constant
const LLKHF_INJECTED: u32 = 0x00000010;

// Thread-safe storage for the AppHandle
static APP_HANDLE: LazyLock<Mutex<Option<AppHandle>>> = LazyLock::new(|| Mutex::new(None));

// Track Win key state ourselves (more reliable than GetAsyncKeyState in hooks)
static WIN_KEY_DOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

static mut HHOOK_HANDLE: HHOOK = HHOOK(std::ptr::null_mut());

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = kbd_struct.vkCode;
        let flags = kbd_struct.flags.0;
        
        // Ignore injected events (from other programs or OS)
        if (flags & LLKHF_INJECTED) != 0 {
            return CallNextHookEx(Some(HHOOK_HANDLE), code, w_param, l_param);
        }
        
        let msg = w_param.0 as u32;
        let is_key_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_key_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        // Track Win key state
        let is_win_key = vk_code == VK_LWIN.0 as u32 || vk_code == VK_RWIN.0 as u32;
        if is_win_key {
            if let Ok(mut win_down) = WIN_KEY_DOWN.lock() {
                if is_key_down {
                    *win_down = true;
                    log_debug(&format!("Win key DOWN (vk: {})", vk_code));
                } else if is_key_up {
                    *win_down = false;
                    log_debug(&format!("Win key UP (vk: {})", vk_code));
                }
            }
        }

        // Handle V key
        if vk_code == VK_V.0 as u32 {
            if let Ok(mut win_down) = WIN_KEY_DOWN.lock() {
                // CRITICAL: Verify Win key is ACTUALLY down using GetAsyncKeyState
                // This handles cases where we missed the UP event (focus changes, taskbar clicks, etc.)
                let lwin_actually_down = unsafe { (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0 };
                let rwin_actually_down = unsafe { (GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0 };
                let win_actually_down = lwin_actually_down || rwin_actually_down;
                
                // Reset our tracked state if it doesn't match reality
                if *win_down && !win_actually_down {
                    log_debug("RESET: Win key state was stuck, resetting to false");
                    *win_down = false;
                }
                
                if *win_down && win_actually_down {
                    // Win+V detected!
                    log_debug(&format!("Win+V detected! (key event: {})", 
                        if is_key_down { "DOWN" } else { "UP" }));
                    
                    // Trigger window toggle ONLY on key down
                    if is_key_down {
                        if let Ok(guard) = APP_HANDLE.lock() {
                            if let Some(app) = guard.as_ref() {
                                let app_for_closure = app.clone();
                                let _ = app.run_on_main_thread(move || {
                                    toggle_window(&app_for_closure);
                                });
                            }
                        }
                    }
                    
                    // CRITICAL: Block this V key event
                    log_debug("Blocking V key event");
                    return LRESULT(1);
                } else {
                    // V pressed without Win - ensure state is reset
                    *win_down = false;
                }
            }
        }

        // CRITICAL: Also block Win key events when Win+V combo is active
        if is_win_key {
            if let Ok(win_down) = WIN_KEY_DOWN.lock() {
                if *win_down && is_key_down {
                    // Win key is being held - check if we should block it
                    // We block the Win UP event later when V is released
                    log_debug("Win key held - allowing for now");
                }
            }
        }
    }

    CallNextHookEx(Some(HHOOK_HANDLE), code, w_param, l_param)
}

// Logic duplicated from lib.rs or accessed if we move it.
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn start(app: AppHandle) {
    // Store the AppHandle
    if let Ok(mut guard) = APP_HANDLE.lock() {
        *guard = Some(app);
    }

    // Spawn the hook thread
    thread::spawn(|| {
        unsafe {
            let h_mod = GetModuleHandleW(None).unwrap_or(HMODULE(std::ptr::null_mut()));

            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                Some(HINSTANCE(h_mod.0)),
                0,
            );

            if let Ok(h) = hook {
                HHOOK_HANDLE = h;

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                UnhookWindowsHookEx(HHOOK_HANDLE);
            } else {
                eprintln!("Failed to install keyboard hook");
            }
        }
    });
}

fn log_debug(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("hook_debug.log") 
    {
        let _ = writeln!(file, "{}", msg);
    }
}
