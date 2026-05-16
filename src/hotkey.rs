use std::sync::{mpsc, OnceLock};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, VK_X, VK_Z,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage,
    MB_ICONERROR, MB_OK, MESSAGEBOX_STYLE, MSG, WM_HOTKEY,
};

#[derive(Clone, Copy)]
pub enum HotkeyAction {
    Apply,
    Cascade,
}

static ACTION_TX: OnceLock<mpsc::Sender<HotkeyAction>> = OnceLock::new();
const HOTKEY_ID_APPLY: i32 = 9000;
const HOTKEY_ID_CASCADE: i32 = 9001;

pub fn run_global_hotkeys(action_tx: mpsc::Sender<HotkeyAction>) {
    std::thread::spawn(move || {
        let _ = ACTION_TX.set(action_tx);

        unsafe {
            let modifiers = MOD_CONTROL | MOD_ALT;

            let apply_ok = RegisterHotKey(None, HOTKEY_ID_APPLY, modifiers, VK_Z.0 as u32).is_ok();
            let cascade_ok = RegisterHotKey(None, HOTKEY_ID_CASCADE, modifiers, VK_X.0 as u32).is_ok();

            if !apply_ok || !cascade_ok {
                let msg: Vec<u16> =
                    "グローバルホットキー(Ctrl+Alt+Z/X)の登録に失敗しました。\0".encode_utf16().collect();
                let title: Vec<u16> = "PreventSleep\0".encode_utf16().collect();
                MessageBoxW(
                    None,
                    windows::core::PCWSTR(msg.as_ptr()),
                    windows::core::PCWSTR(title.as_ptr()),
                    MB_OK | MESSAGEBOX_STYLE(MB_ICONERROR.0),
                );
                let _ = UnregisterHotKey(None, HOTKEY_ID_APPLY);
                let _ = UnregisterHotKey(None, HOTKEY_ID_CASCADE);
                return;
            }

            let mut msg = MSG::default();
            loop {
                let ret = GetMessageW(&mut msg, None, 0, 0);
                if ret.0 <= 0 {
                    break;
                }

                if msg.message == WM_HOTKEY {
                    match msg.wParam.0 as i32 {
                        HOTKEY_ID_APPLY => {
                            if let Some(tx) = ACTION_TX.get() {
                                let _ = tx.send(HotkeyAction::Apply);
                            }
                        }
                        HOTKEY_ID_CASCADE => {
                            if let Some(tx) = ACTION_TX.get() {
                                let _ = tx.send(HotkeyAction::Cascade);
                            }
                        }
                        _ => {}
                    }
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnregisterHotKey(None, HOTKEY_ID_APPLY);
            let _ = UnregisterHotKey(None, HOTKEY_ID_CASCADE);
        }
    });
}

