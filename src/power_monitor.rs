use std::sync::mpsc::Sender;
use windows::core::GUID;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Power::RegisterPowerSettingNotification;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
    RegisterClassExW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, DEVICE_NOTIFY_WINDOW_HANDLE,
    MSG, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

const WM_POWERBROADCAST: u32 = 0x0218;
const PBT_POWERSETTINGCHANGE: usize = 0x8013;

// GUID_MONITOR_POWER_ON: 02731015-4510-4526-99e6-e5a17ebd1aea
const GUID_MONITOR_POWER_ON: GUID = GUID {
    data1: 0x0273_1015,
    data2: 0x4510,
    data3: 0x4526,
    data4: [0x99, 0xe6, 0xe5, 0xa1, 0x7e, 0xbd, 0x1a, 0xea],
};

#[repr(C)]
struct PowerBroadcastSetting {
    power_setting: GUID,
    data_length: u32,
    data: u8,
}

/// 別スレッドで隠し Win32 ウィンドウを生成し、モニター電源ON イベントを監視する。
/// モニター電源ON 検出時に sender へ () を送信する。
pub fn start_power_monitor(sender: Sender<()>) {
    std::thread::spawn(move || {
        unsafe {
            let class_name = encode_wide("PreventSleepPowerMonitor\0");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(power_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                    .map(|h| HINSTANCE(h.0))
                    .unwrap_or_default(),
                hIcon: Default::default(),
                hCursor: Default::default(),
                hbrBackground: Default::default(),
                lpszMenuName: windows::core::PCWSTR::null(),
                lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
                hIconSm: Default::default(),
            };
            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                windows::core::PCWSTR(class_name.as_ptr()),
                windows::core::PCWSTR(encode_wide("PreventSleepPowerMonitor\0").as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0,
                0,
                None,
                None,
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                    .map(|h| HINSTANCE(h.0))
                    .ok(),
                None,
            );

            if hwnd.is_err() {
                return;
            }
            let hwnd = hwnd.unwrap();

            // sender をウィンドウのユーザーデータに保存
            let boxed_sender = Box::new(sender);
            windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                Box::into_raw(boxed_sender) as isize,
            );

            // 電源設定通知を登録
            let mut guid = GUID_MONITOR_POWER_ON;
            let _ = RegisterPowerSettingNotification(
                windows::Win32::Foundation::HANDLE(hwnd.0),
                &mut guid,
                DEVICE_NOTIFY_WINDOW_HANDLE,
            );

            // メッセージループ
            let mut msg = MSG::default();
            loop {
                let ret = GetMessageW(&mut msg, None, 0, 0);
                if ret.0 == 0 || ret.0 == -1 {
                    break;
                }
                DispatchMessageW(&msg);
            }
        }
    });
}

unsafe extern "system" fn power_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_POWERBROADCAST && wparam.0 == PBT_POWERSETTINGCHANGE {
        let setting = &*(lparam.0 as *const PowerBroadcastSetting);
        if setting.power_setting == GUID_MONITOR_POWER_ON && setting.data != 0 {
            // モニター ON
            let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            );
            if ptr != 0 {
                let sender = &*(ptr as *const std::sync::mpsc::Sender<()>);
                let _ = sender.send(());
            }
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn encode_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().collect()
}
