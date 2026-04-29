use crate::config::Rule;
use regex::Regex;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::core::BOOL;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop,
    EnumWindows, GetClassNameW, GetWindowPlacement, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, GetWindowThreadProcessId, IsWindow, IsWindowVisible, SetWindowPos,
    HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST, SWP_NOZORDER, SWP_SHOWWINDOW, SW_MAXIMIZE,
    SW_MINIMIZE, WINDOWPLACEMENT,
};
use windows::Win32::System::Threading::GetCurrentProcessId;

#[derive(Debug, Clone)]
pub struct MonitorRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl MonitorRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

pub fn enum_monitors() -> Vec<MonitorRect> {
    let mut monitors: Vec<MonitorRect> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }
    monitors
}

unsafe extern "system" fn monitor_enum_proc(
    hmon: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<MonitorRect>);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(hmon, &mut info).as_bool() {
        let wa = info.rcWork;
        monitors.push(MonitorRect {
            left: wa.left,
            top: wa.top,
            right: wa.right,
            bottom: wa.bottom,
        });
    }
    BOOL(1)
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class_name: String,
    pub rect: MonitorRect,
}

pub fn enum_windows_list() -> Vec<WindowInfo> {
    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_all_windows_proc),
            LPARAM(&mut windows as *mut _ as isize),
        );
    }
    windows
}

unsafe extern "system" fn enum_all_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    // PreventSleep 自身のウィンドウは処理対象外
    let mut pid: u32 = 0;
    let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == GetCurrentProcessId() {
        return BOOL(1);
    }

    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    let _ = GetWindowPlacement(hwnd, &mut placement);
    if placement.showCmd == SW_MINIMIZE.0 as u32 || placement.showCmd == SW_MAXIMIZE.0 as u32 {
        return BOOL(1);
    }

    let text_len = GetWindowTextLengthW(hwnd);
    if text_len == 0 {
        return BOOL(1);
    }
    let mut title_buf = vec![0u16; (text_len + 1) as usize];
    GetWindowTextW(hwnd, &mut title_buf);
    let title = OsString::from_wide(&title_buf[..text_len as usize])
        .to_string_lossy()
        .to_string();

    if title.trim().is_empty() {
        return BOOL(1);
    }

    let mut class_buf = vec![0u16; 256];
    let class_len = GetClassNameW(hwnd, &mut class_buf);
    let class_name = OsString::from_wide(&class_buf[..class_len as usize])
        .to_string_lossy()
        .to_string();

    if (title == "Program Manager" && class_name == "Progman")
        || class_name == "Shell_TrayWnd"
        || class_name == "Shell_SecondaryTrayWnd"
        || class_name == "Windows.UI.Core.CoreWindow"
    {
        return BOOL(1);
    }

    let mut rect = RECT::default();
    let _ = GetWindowRect(hwnd, &mut rect);

    windows.push(WindowInfo {
        hwnd: hwnd.0 as isize,
        title,
        class_name,
        rect: MonitorRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        },
    });

    BOOL(1)
}

fn is_window_matched(win: &WindowInfo, rule: &Rule, num_display: usize) -> bool {
    if !rule.displays.contains(&num_display.to_string()) {
        return false;
    }
    if !rule.title_regex.is_empty() {
        match Regex::new(&rule.title_regex) {
            Ok(re) => {
                if !re.is_match(&win.title) {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }
    if !rule.class_regex.is_empty() {
        match Regex::new(&rule.class_regex) {
            Ok(re) => {
                if !re.is_match(&win.class_name) {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }
    true
}

fn find_monitor_for_pos(left: i32, top: i32, monitors: &[MonitorRect]) -> Option<&MonitorRect> {
    monitors
        .iter()
        .find(|m| m.left <= left && left < m.right && m.top <= top && top < m.bottom)
}

pub fn relocate_windows(rules: &[Rule], num_display: usize) -> String {
    relocate_windows_impl(rules, num_display, false)
}

pub fn relocate_windows_cascading(rules: &[Rule], num_display: usize) -> String {
    relocate_windows_impl(rules, num_display, true)
}

fn relocate_windows_impl(rules: &[Rule], num_display: usize, cascade_unspecified: bool) -> String {
    let monitors = enum_monitors();
    let windows = enum_windows_list();

    let mut log = String::new();
    for (i, m) in monitors.iter().enumerate() {
        log.push_str(&format!(
            "# {}, {}, {}, {}, {}\r\n",
            i + 1,
            m.left,
            m.top,
            m.width(),
            m.height()
        ));
    }
    log.push_str("\r\n");

    let primary = monitors.first().cloned().unwrap_or(MonitorRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    });

    const TITLE_HEIGHT: i32 = 25;
    const CASCADE_OFFSET: i32 = 35;
    let mut shift_x = 0i32;
    let mut shift_y = 0i32;
    let mut cascade_index = 0i32;

    for win in &windows {
        let old_left = win.rect.left;
        let old_top = win.rect.top;
        let old_w = win.rect.right - win.rect.left;
        let old_h = win.rect.bottom - win.rect.top;

        let mut left = old_left;
        let mut top = old_top;
        let mut width = old_w;
        let mut height = old_h;
        let mut is_specified = false;

        for rule in rules {
            if is_window_matched(win, rule, num_display) {
                left = rule.x;
                top = rule.y;
                width = rule.w;
                height = rule.h;
                is_specified = true;
                break;
            }
        }

        let should_cascade = cascade_unspecified && !is_specified;

        if !is_specified {
            if should_cascade {
                left = cascade_index * CASCADE_OFFSET;
                top = cascade_index * CASCADE_OFFSET;
                width = win.rect.right - win.rect.left;
                height = win.rect.bottom - win.rect.top;
                cascade_index += 1;
            } else {
                left = win.rect.left;
                top = win.rect.top;
                width = win.rect.right - win.rect.left;
                height = win.rect.bottom - win.rect.top;
            }
        }

        width = width.max(1);
        height = height.max(1);

        let target = if let Some(m) = find_monitor_for_pos(left, top, &monitors) {
            m.clone()
        } else {
            let m = primary.clone();
            if !cascade_unspecified {
                left = m.left + shift_x;
                top = m.top + m.height() - TITLE_HEIGHT - shift_y;
                shift_x += TITLE_HEIGHT;
                shift_y += TITLE_HEIGHT;
                if shift_x > m.width() / 2 {
                    shift_x = 0;
                    shift_y = 0;
                }
            }
            m
        };

        if left + width > target.right {
            left = target.right - width;
        }
        if top + height > target.bottom {
            top = target.bottom - height;
        }
        if left < target.left {
            width -= target.left - left;
            left = target.left;
        }
        if top < target.top {
            height -= target.top - top;
            top = target.top;
        }
        width = width.max(1);
        height = height.max(1);

        unsafe {
            let hwnd = HWND(win.hwnd as *mut _);
            if IsWindow(Some(hwnd)).as_bool() {
                if should_cascade {
                    // Cascading対象は移動のたびに最前面へ。
                    // TOPMOST -> NOTOPMOST の順で強制的に前面へ出しつつ、常時TOPMOST化はしない。
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_TOPMOST),
                        left,
                        top,
                        width,
                        height,
                        SWP_SHOWWINDOW,
                    );
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_TOP),
                        left,
                        top,
                        width,
                        height,
                        SWP_SHOWWINDOW,
                    );
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_NOTOPMOST),
                        left,
                        top,
                        width,
                        height,
                        SWP_SHOWWINDOW,
                    );
                    let _ = BringWindowToTop(hwnd);
                } else {
                    // 通常移動は位置・サイズのみ更新し、Zオーダーは維持する。
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_NOTOPMOST),
                        left,
                        top,
                        width,
                        height,
                        SWP_SHOWWINDOW | SWP_NOZORDER,
                    );
                }
            }
        }

        log.push_str(&format!(
            "\"{}\",\"{}\", ({}, {}, {}, {}) -> ({}, {}, {}, {})\r\n",
            regex::escape(&win.title),
            regex::escape(&win.class_name),
            old_left,
            old_top,
            old_w,
            old_h,
            left,
            top,
            width,
            height,
        ));
    }

    if log.trim().is_empty() {
        "No target windows found.".to_string()
    } else {
        log
    }
}

pub fn turn_off_monitor() {
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, HWND_BROADCAST, WM_SYSCOMMAND};
    const SC_MONITORPOWER: usize = 0xF170;
    const MONITOR_SHUTOFF: isize = 2;
    unsafe {
        SendMessageW(
            HWND_BROADCAST,
            WM_SYSCOMMAND,
            Some(windows::Win32::Foundation::WPARAM(SC_MONITORPOWER)),
            Some(windows::Win32::Foundation::LPARAM(MONITOR_SHUTOFF)),
        );
    }
}

