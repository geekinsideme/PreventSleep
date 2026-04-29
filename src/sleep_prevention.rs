use windows::Win32::System::Power::SetThreadExecutionState;
use windows::Win32::System::Power::{ES_CONTINUOUS, ES_DISPLAY_REQUIRED};
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_MOUSE, MOUSEEVENTF_MOVE};

/// スリープ・画面OFFを抑止する (Continuous + DisplayRequired)
pub fn prevent_sleep() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED);
    }
}

/// スリープ防止を解除する (Continuous のみ)
pub fn release_sleep_prevention() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
}

/// マウスを微小移動して画面のアクティビティを通知する (相対座標 0,0 移動)
pub fn send_mouse_move() {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            mi: windows::Win32::UI::Input::KeyboardAndMouse::MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}
