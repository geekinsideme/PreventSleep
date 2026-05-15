use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, TranslateMessage, DispatchMessageW, WM_HOTKEY, MSG,
    MessageBoxW, MB_OK, MB_ICONERROR, MESSAGEBOX_STYLE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_SHIFT
};

const HOTKEY_ID_APPLY: i32 = 9000;   // Alt+Shift+Z: 配置適用
const HOTKEY_ID_CASCADE: i32 = 9001; // Alt+Shift+X: 階段配置
const VK_Z: u32 = 0x5A;
const VK_X: u32 = 0x58;

pub fn run_global_hotkeys<F1, F2>(apply_layout: F1, apply_cascading: F2)
where
    F1: Fn() + Send + 'static,
    F2: Fn() + Send + 'static,
{
    std::thread::spawn(move || {
        unsafe {
            let modifiers = MOD_ALT | MOD_SHIFT;
            let mut failed: Vec<&str> = Vec::new();

            if RegisterHotKey(None, HOTKEY_ID_APPLY, modifiers, VK_Z).is_err() {
                failed.push("Alt+Shift+Z (配置適用)");
            }
            if RegisterHotKey(None, HOTKEY_ID_CASCADE, modifiers, VK_X).is_err() {
                failed.push("Alt+Shift+X (階段配置)");
            }

            if !failed.is_empty() {
                let text = format!(
                    "グローバルホットキーの登録に失敗しました:\n{}\0",
                    failed.join("\n")
                );
                let msg: Vec<u16> = text.encode_utf16().collect();
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
                if ret.0 == 0 {
                    break;
                }
                if msg.message == WM_HOTKEY {
                    match msg.wParam.0 as i32 {
                        HOTKEY_ID_APPLY => apply_layout(),
                        HOTKEY_ID_CASCADE => apply_cascading(),
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

