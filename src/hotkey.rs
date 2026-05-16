use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, OnceLock};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, MessageBoxW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT,
    MB_ICONERROR, MB_OK, MESSAGEBOX_STYLE, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Clone, Copy)]
pub enum HotkeyAction {
    Apply,
    Cascade,
}

static ACTION_TX: OnceLock<mpsc::Sender<HotkeyAction>> = OnceLock::new();
// App::new() de touroku. Hotkey hakkaji ni soku repaint o yokyuu surutame ni tsukau
static EGUI_CTX: OnceLock<egui::Context> = OnceLock::new();

static Z_LATCH: AtomicBool = AtomicBool::new(false);
static X_LATCH: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);

// Virtual-key codes
const VK_CONTROL: u32 = 0x11;
const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_MENU: u32 = 0x12;
const VK_LMENU: u32 = 0xA4;
const VK_RMENU: u32 = 0xA5;
const VK_Z: u32 = 0x5A;
const VK_X: u32 = 0x58;

#[inline]
fn is_ctrl(vk: u32) -> bool {
    vk == VK_CONTROL || vk == VK_LCONTROL || vk == VK_RCONTROL
}

#[inline]
fn is_alt(vk: u32) -> bool {
    vk == VK_MENU || vk == VK_LMENU || vk == VK_RMENU
}

fn fire_action(action: HotkeyAction) {
    if let Some(tx) = ACTION_TX.get() {
        let _ = tx.send(action);
    }
    // Soku repaint shite UI textbox o chien naku koushin
    if let Some(ctx) = EGUI_CTX.get() {
        ctx.request_repaint();
    }
}

unsafe extern "system" fn keyboard_hook_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode == HC_ACTION as i32 {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let msg = wparam.0 as u32;
        let vk = kb.vkCode;

        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_up   = msg == WM_KEYUP   || msg == WM_SYSKEYUP;

        if is_ctrl(vk) {
            if is_down { CTRL_DOWN.store(true,  Ordering::Relaxed); }
            if is_up   { CTRL_DOWN.store(false, Ordering::Relaxed); }
        }
        if is_alt(vk) {
            if is_down { ALT_DOWN.store(true,  Ordering::Relaxed); }
            if is_up   { ALT_DOWN.store(false, Ordering::Relaxed); }
        }

        let ctrl = CTRL_DOWN.load(Ordering::Relaxed);
        let alt  = ALT_DOWN.load(Ordering::Relaxed);

        // Ctrl+Alt+Z
        if vk == VK_Z {
            if is_up && Z_LATCH.swap(false, Ordering::Relaxed) {
                return LRESULT(1);
            }
            if is_down && ctrl && alt && !Z_LATCH.swap(true, Ordering::Relaxed) {
                fire_action(HotkeyAction::Apply);
                return LRESULT(1);
            }
        }

        // Ctrl+Alt+X
        if vk == VK_X {
            if is_up && X_LATCH.swap(false, Ordering::Relaxed) {
                return LRESULT(1);
            }
            if is_down && ctrl && alt && !X_LATCH.swap(true, Ordering::Relaxed) {
                fire_action(HotkeyAction::Cascade);
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(None, ncode, wparam, lparam)
}

pub fn set_egui_context(ctx: egui::Context) {
    let _ = EGUI_CTX.set(ctx);
}

pub fn run_global_hotkeys(action_tx: mpsc::Sender<HotkeyAction>) {
    std::thread::spawn(move || {
        let _ = ACTION_TX.set(action_tx);

        unsafe {
            let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0) {
                Ok(h) => h,
                Err(_) => {
                    let msg: Vec<u16> =
                        "グローバルキーボードフックの登録に失敗しました。\0"
                            .encode_utf16()
                            .collect();
                    let title: Vec<u16> = "PreventSleep\0".encode_utf16().collect();
                    MessageBoxW(
                        None,
                        windows::core::PCWSTR(msg.as_ptr()),
                        windows::core::PCWSTR(title.as_ptr()),
                        MB_OK | MESSAGEBOX_STYLE(MB_ICONERROR.0),
                    );
                    return;
                }
            };

            let mut msg = MSG::default();
            loop {
                let ret = GetMessageW(&mut msg, None, 0, 0);
                if ret.0 <= 0 {
                    break;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        }
    });
}
