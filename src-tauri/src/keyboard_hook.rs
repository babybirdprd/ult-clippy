use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput,
    INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_LWIN, VK_RWIN, VK_V
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, HC_ACTION, WM_KEYDOWN, WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP,
    DispatchMessageW, GetMessageW, TranslateMessage, HHOOK, MSG,
};
use std::sync::LazyLock;

// Thread-safe storage for the AppHandle
static APP_HANDLE: LazyLock<Mutex<Option<AppHandle>>> = LazyLock::new(|| Mutex::new(None));

static mut HHOOK_HANDLE: HHOOK = HHOOK(std::ptr::null_mut());

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code == HC_ACTION {
        let kbd_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);

        // We only care about Key Down events for triggering logic
        let msg = w_param.0 as u32;
        let is_key_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let _is_key_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        if kbd_struct.vkCode == VK_V.0 as u32 {
            // Check if Windows key is down
            let lwin_down = (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0;
            let rwin_down = (GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0;

            if lwin_down || rwin_down {
                if is_key_down {
                    // Suppress Start Menu by injecting a harmless key (Ctrl)
                    send_ctrl_input();

                    // Trigger the window toggle
                    if let Ok(guard) = APP_HANDLE.lock() {
                        if let Some(app) = guard.as_ref() {
                           // We need to run this on the main thread because UI operations usually require it.
                           // However, AppHandle is thread safe.
                           let app_clone = app.clone();
                           // We can't block here.
                           let _ = app_clone.run_on_main_thread(move || {
                               toggle_window(&app_clone);
                           });
                        }
                    }
                }
                // Swallow the event (both up and down) to prevent the system from seeing Win+V
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(HHOOK_HANDLE, code, w_param, l_param)
}

fn send_ctrl_input() {
    unsafe {
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
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
                h_mod,
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
