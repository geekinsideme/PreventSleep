use std::sync::{mpsc, OnceLock};
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

#[derive(Clone, Copy)]
pub enum HotkeyAction {
    Apply,
    Cascade,
}

static ACTION_TX: OnceLock<mpsc::Sender<HotkeyAction>> = OnceLock::new();
// App::new() で登録。ホットキー発火時に即 repaint を要求するために使う
static EGUI_CTX: OnceLock<egui::Context> = OnceLock::new();

// Virtual-key codes
const VK_CONTROL: i32 = 0x11;
const VK_MENU: i32 = 0x12; // Alt
const VK_Z: i32 = 0x5A;
const VK_X: i32 = 0x58;

/// 指定キーが現在押されているかどうか（システム全体、前面/背面問わず）
fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16) & 0x8000 != 0 }
}

fn fire_action(action: HotkeyAction) {
    if let Some(tx) = ACTION_TX.get() {
        let _ = tx.send(action);
    }
    // 即 repaint して UI テキストボックスを遅延なく更新
    if let Some(ctx) = EGUI_CTX.get() {
        ctx.request_repaint();
    }
}

/// egui::Context を登録して、ホットキー発火時に即 repaint できるようにする。
/// App::new() 内で呼ぶこと。
pub fn set_egui_context(ctx: egui::Context) {
    let _ = EGUI_CTX.set(ctx);
}

/// グローバルホットキー監視スレッドを起動する。
/// GetAsyncKeyState ポーリングにより前面・背面を問わず確実に検出。
/// 管理者権限不要。
pub fn run_global_hotkeys(action_tx: mpsc::Sender<HotkeyAction>) {
    std::thread::spawn(move || {
        let _ = ACTION_TX.set(action_tx);

        let mut z_was_down = false;
        let mut x_was_down = false;

        loop {
            // 30ms ポーリング: キー押下を取りこぼさない間隔
            std::thread::sleep(Duration::from_millis(30));

            let ctrl = key_down(VK_CONTROL);
            let alt  = key_down(VK_MENU);
            let z    = key_down(VK_Z);
            let x    = key_down(VK_X);

            if ctrl && alt {
                // Ctrl+Alt+Z: 配置適用（立ち上がりエッジのみ発火）
                if z && !z_was_down {
                    fire_action(HotkeyAction::Apply);
                }
                // Ctrl+Alt+X: 階段配置（立ち上がりエッジのみ発火）
                if x && !x_was_down {
                    fire_action(HotkeyAction::Cascade);
                }
            }

            z_was_down = z;
            x_was_down = x;
        }
    });
}
