use crate::config::{CoordSpec, Rule, SizeSpec};
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
    HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_MAXIMIZE,
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

fn split_tall_portrait_monitor(m: &MonitorRect) -> Option<[MonitorRect; 2]> {
    let width = m.width();
    let height = m.height();

    // 縦長かつ高さが十分に大きい場合は、上下2分割の仮想モニターとして扱う。
    if height > width && height > 1500 {
        let mid = m.top + (height / 2);
        return Some([
            MonitorRect {
                left: m.left,
                top: m.top,
                right: m.right,
                bottom: mid,
            },
            MonitorRect {
                left: m.left,
                top: mid,
                right: m.right,
                bottom: m.bottom,
            },
        ]);
    }

    None
}

fn expand_monitors_with_virtual_split(monitors: Vec<MonitorRect>) -> Vec<MonitorRect> {
    let mut expanded: Vec<MonitorRect> = Vec::with_capacity(monitors.len());
    for m in monitors {
        if let Some([top_half, bottom_half]) = split_tall_portrait_monitor(&m) {
            expanded.push(top_half);
            expanded.push(bottom_half);
        } else {
            expanded.push(m);
        }
    }
    expanded
}

const WINDOW_MARGIN_LEFT: i32 = 3;
const WINDOW_MARGIN_RIGHT: i32 = 3;
const WINDOW_MARGIN_TOP: i32 = 5;
const WINDOW_MARGIN_BOTTOM: i32 = 0;
const CASCADE_OFFSET: i32 = 35;

fn effective_monitor_area(m: &MonitorRect) -> MonitorRect {
    let mut left = m.left + WINDOW_MARGIN_LEFT;
    let mut top = m.top + WINDOW_MARGIN_TOP;
    let mut right = m.right - WINDOW_MARGIN_RIGHT;
    let mut bottom = m.bottom - WINDOW_MARGIN_BOTTOM;

    if right <= left {
        let center_x = m.left + (m.width() / 2);
        left = center_x;
        right = center_x + 1;
    }
    if bottom <= top {
        let center_y = m.top + (m.height() / 2);
        top = center_y;
        bottom = center_y + 1;
    }

    MonitorRect {
        left,
        top,
        right,
        bottom,
    }
}

fn monitor_key(m: &MonitorRect) -> (i32, i32, i32, i32) {
    (m.left, m.top, m.right, m.bottom)
}

fn ordered_monitors_for_screen_numbering(
    monitors: &[MonitorRect],
    origin: &MonitorRect,
) -> Vec<MonitorRect> {
    let origin_key = monitor_key(origin);
    let mut right_side: Vec<MonitorRect> = Vec::new();
    let mut left_side: Vec<MonitorRect> = Vec::new();
    let mut same_x_non_origin: Vec<MonitorRect> = Vec::new();

    for m in monitors {
        if monitor_key(m) == origin_key {
            continue;
        }

        if m.left > 0 {
            right_side.push(m.clone());
        } else if m.left < 0 {
            left_side.push(m.clone());
        } else {
            same_x_non_origin.push(m.clone());
        }
    }

    let dist2 = |m: &MonitorRect| -> i64 {
        let x = m.left as i64;
        let y = m.top as i64;
        x * x + y * y
    };

    let tie_key = |m: &MonitorRect| -> (i32, i32, i32, i32) {
        (m.top.abs(), m.left.abs(), m.top, m.left)
    };

    right_side.sort_by_key(|m| (dist2(m), tie_key(m)));
    left_side.sort_by_key(|m| (dist2(m), tie_key(m)));
    same_x_non_origin.sort_by_key(|m| (dist2(m), tie_key(m)));

    let mut ordered: Vec<MonitorRect> = Vec::with_capacity(monitors.len());
    ordered.push(origin.clone());
    ordered.extend(right_side);
    ordered.extend(left_side);
    ordered.extend(same_x_non_origin);

    ordered
}

fn monitor_by_screen_index(
    monitors: &[MonitorRect],
    origin: &MonitorRect,
    index_1based: usize,
) -> MonitorRect {
    let ordered = ordered_monitors_for_screen_numbering(monitors, origin);

    let requested = index_1based.max(1) - 1;
    let idx = requested.min(ordered.len().saturating_sub(1));
    ordered
        .get(idx)
        .cloned()
        .unwrap_or_else(|| origin.clone())
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
    expand_monitors_with_virtual_split(monitors)
}

pub fn monitor_with_origin_top_left(monitors: &[MonitorRect]) -> Option<MonitorRect> {
    monitors
        .iter()
        .find(|m| m.left == 0 && m.top == 0)
        .cloned()
        .or_else(|| {
            monitors
                .iter()
                .find(|m| m.left <= 0 && 0 < m.right && m.top <= 0 && 0 < m.bottom)
                .cloned()
        })
        .or_else(|| {
            monitors
                .iter()
                .min_by_key(|m| m.left.abs() + m.top.abs())
                .cloned()
        })
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

pub fn preventsleep_window_origin_bottom_left_position() -> (f32, f32) {
    let monitors = enum_monitors();
    let origin_monitor = monitor_with_origin_top_left(&monitors).unwrap_or(MonitorRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    });

    let window_height = 190.0_f32;
    const NON_CLIENT_HEIGHT: f32 = 32.0;
    let x = origin_monitor.left as f32;
    let y = ((origin_monitor.bottom as f32) - (window_height + NON_CLIENT_HEIGHT))
        .max(origin_monitor.top as f32);

    (x, y)
}

pub fn relocate_preventsleep_window_to_origin_bottom_left() {
    let monitors = enum_monitors();
    let target_monitor = monitor_with_origin_top_left(&monitors).unwrap_or(MonitorRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    });

    #[derive(Copy, Clone)]
    struct RelocateTarget {
        left: i32,
        top: i32,
        bottom: i32,
    }

    unsafe extern "system" fn enum_preventsleep_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        // PreventSleep 自プロセスのウィンドウのみ対象にする。
        // タイトル先頭一致だけだと VS Code 等を誤判定する可能性がある。
        let mut pid: u32 = 0;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid != GetCurrentProcessId() {
            return BOOL(1);
        }

        let text_len = GetWindowTextLengthW(hwnd);
        if text_len <= 0 {
            return BOOL(1);
        }

        let mut title_buf = vec![0u16; (text_len + 1) as usize];
        GetWindowTextW(hwnd, &mut title_buf);
        let title = OsString::from_wide(&title_buf[..text_len as usize])
            .to_string_lossy()
            .to_string();

        // GUI モードの PreventSleep メインウィンドウだけを対象にする。
        // 以前の starts_with("PreventSleep") だと
        // "PreventSleep.txt - ... - Visual Studio Code" のようなタイトルを誤判定しうる。
        if !(title == "PreventSleep v2.5.0" || title.starts_with("PreventSleep v")) {
            return BOOL(1);
        }

        let target = *(lparam.0 as *const RelocateTarget);
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        let window_height = (rect.bottom - rect.top).max(1);
        let target_y = (target.bottom - window_height).max(target.top);

        let _ = SetWindowPos(
            hwnd,
            Some(HWND_NOTOPMOST),
            target.left,
            target_y,
            0,
            0,
            SWP_SHOWWINDOW | SWP_NOZORDER | SWP_NOSIZE,
        );

        BOOL(1)
    }

    let target = RelocateTarget {
        left: target_monitor.left,
        top: target_monitor.top,
        bottom: target_monitor.bottom,
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_preventsleep_windows_proc),
            LPARAM(&target as *const _ as isize),
        );
    }
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
    if !is_display_count_enabled(&rule.displays, num_display) {
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

fn is_display_count_enabled(displays: &str, num_display: usize) -> bool {
    let trimmed = displays.trim();
    if trimmed.is_empty() {
        return true;
    }

    // @N は「配置対象モニタ指定」であり、接続画面数条件には使わない
    if trimmed.contains('@') {
        return true;
    }

    if num_display < 10 {
        if let Some(d) = char::from_digit(num_display as u32, 10) {
            return trimmed.chars().any(|c| c == d);
        }
        return false;
    }

    let target = num_display.to_string();
    trimmed
        .split(|c: char| !c.is_ascii_digit())
        .any(|token| !token.is_empty() && token == target)
}

fn parse_monitor_index_from_displays(displays: &str) -> Option<usize> {
    let at = displays.find('@')?;
    let digits: String = displays[at + 1..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();

    if digits.is_empty() {
        return None;
    }

    digits.parse::<usize>().ok().map(|n| n.max(1))
}

fn find_monitor_for_pos(left: i32, top: i32, monitors: &[MonitorRect]) -> Option<&MonitorRect> {
    monitors
        .iter()
        .find(|m| m.left <= left && left < m.right && m.top <= top && top < m.bottom)
}

fn clamp_to_target_area(
    mut left: i32,
    mut top: i32,
    mut width: i32,
    mut height: i32,
    target: &MonitorRect,
) -> (i32, i32, i32, i32) {
    let target_w = (target.right - target.left).max(1);
    let target_h = (target.bottom - target.top).max(1);

    width = width.max(1).min(target_w);
    height = height.max(1).min(target_h);

    // まず位置を有効領域内に寄せる（この段階ではサイズは削らない）
    if left < target.left {
        left = target.left;
    }
    if top < target.top {
        top = target.top;
    }

    // 右端/下端をはみ出す場合のみ、位置を戻して収める
    if left + width > target.right {
        left = target.right - width;
    }
    if top + height > target.bottom {
        top = target.bottom - height;
    }

    // 念のため最終防衛
    left = left.max(target.left);
    top = top.max(target.top);
    (left, top, width, height)
}

fn resolve_size_spec(spec: &SizeSpec, available: i32) -> i32 {
    let available = available.max(1);
    match spec {
        SizeSpec::Pixels(px) => *px,
        SizeSpec::Fill => available,
        SizeSpec::Percent(p) => ((available as f32) * *p).round() as i32,
    }
    .max(1)
}

fn resolve_coord_spec(
    spec: &CoordSpec,
    target_effective: &MonitorRect,
    is_x_axis: bool,
    pixel_is_relative_to_target: bool,
) -> i32 {
    match spec {
        CoordSpec::Pixels(px) => {
            if pixel_is_relative_to_target {
                if is_x_axis {
                    target_effective.left + *px
                } else {
                    target_effective.top + *px
                }
            } else {
                *px
            }
        }
        CoordSpec::Percent(p) => {
            if is_x_axis {
                target_effective.left + ((target_effective.width() as f32) * *p).round() as i32
            } else {
                target_effective.top + ((target_effective.height() as f32) * *p).round() as i32
            }
        }
    }
}

pub fn relocate_windows(rules: &[Rule], num_display: usize) -> String {
    relocate_windows_impl(rules, num_display, false, None)
}

pub fn relocate_windows_cascading(rules: &[Rule], num_display: usize) -> String {
    relocate_windows_impl(rules, num_display, true, None)
}

/// 原点モニタのみが存在すると仮定して配置を行う（num_display は 1 固定）
pub fn relocate_windows_single_screen(rules: &[Rule]) -> String {
    let all_monitors = enum_monitors();
    let origin = monitor_with_origin_top_left(&all_monitors).unwrap_or(MonitorRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    });
    relocate_windows_impl(rules, 1, false, Some(vec![origin]))
}

fn relocate_windows_impl(
    rules: &[Rule],
    num_display: usize,
    cascade_unspecified: bool,
    monitors_override: Option<Vec<MonitorRect>>,
) -> String {
    let mut monitors = monitors_override.unwrap_or_else(enum_monitors);
    if monitors.is_empty() {
        monitors.push(MonitorRect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        });
    }

    let origin = monitor_with_origin_top_left(&monitors).unwrap_or(MonitorRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    });

    let mut effective_by_monitor: std::collections::HashMap<(i32, i32, i32, i32), MonitorRect> =
        std::collections::HashMap::new();
    for m in &monitors {
        effective_by_monitor.insert(monitor_key(m), effective_monitor_area(m));
    }

    let origin_effective = effective_by_monitor
        .get(&monitor_key(&origin))
        .cloned()
        .unwrap_or_else(|| effective_monitor_area(&origin));

    let windows = enum_windows_list();
    let ordered_monitors = ordered_monitors_for_screen_numbering(&monitors, &origin);

    let mut log = String::new();
    for (i, m) in ordered_monitors.iter().enumerate() {
        log.push_str(&format!(
            "# {}, {}, {}, {}, {}\r\n",
            i + 1,
            m.left,
            m.top,
            m.width(),
            m.height()
        ));
    }

    let mut cascade_cursor_by_monitor: std::collections::HashMap<(i32, i32, i32, i32), (i32, i32)> =
        std::collections::HashMap::new();
    for m in &monitors {
        if let Some(effective) = effective_by_monitor.get(&monitor_key(m)) {
            cascade_cursor_by_monitor
                .entry(monitor_key(m))
                .or_insert((effective.left, effective.top));
        }
    }

    for win in &windows {
        let old_left = win.rect.left;
        let old_top = win.rect.top;
        let old_w = win.rect.right - win.rect.left;
        let old_h = win.rect.bottom - win.rect.top;

        // 設定ファイル上の目標位置（ルール未指定時は -1）
        let mut target_left = -1;
        let mut target_top = -1;
        let mut target_w = -1;
        let mut target_h = -1;

        let mut left = old_left;
        let mut top = old_top;
        let mut width = old_w;
        let mut height = old_h;
        let mut is_specified = false;
        let mut forced_target_monitor: Option<MonitorRect> = None;

        for rule in rules {
            if is_window_matched(win, rule, num_display) {
                let screen_idx = parse_monitor_index_from_displays(&rule.displays);
                forced_target_monitor = screen_idx
                    .map(|idx| monitor_by_screen_index(&monitors, &origin, idx));

                let fallback_monitor_for_percent =
                    find_monitor_for_pos(old_left, old_top, &monitors)
                        .cloned()
                        .unwrap_or_else(|| origin.clone());

                let coord_reference_monitor = match (&forced_target_monitor, &rule.x, &rule.y) {
                    (Some(selected), _, _) => selected.clone(),
                    (None, CoordSpec::Pixels(xp), CoordSpec::Pixels(yp)) => {
                        find_monitor_for_pos(*xp, *yp, &monitors)
                            .cloned()
                            .unwrap_or_else(|| fallback_monitor_for_percent.clone())
                    }
                    _ => fallback_monitor_for_percent.clone(),
                };

                let coord_reference_effective = effective_by_monitor
                    .get(&monitor_key(&coord_reference_monitor))
                    .cloned()
                    .unwrap_or_else(|| effective_monitor_area(&coord_reference_monitor));

                let pixel_is_relative_to_target = forced_target_monitor.is_some();
                left = resolve_coord_spec(
                    &rule.x,
                    &coord_reference_effective,
                    true,
                    pixel_is_relative_to_target,
                );
                top = resolve_coord_spec(
                    &rule.y,
                    &coord_reference_effective,
                    false,
                    pixel_is_relative_to_target,
                );

                let target_for_size = forced_target_monitor
                    .clone()
                    .or_else(|| find_monitor_for_pos(left, top, &monitors).cloned())
                    .unwrap_or_else(|| coord_reference_monitor.clone());
                let target_for_size_effective = effective_by_monitor
                    .get(&monitor_key(&target_for_size))
                    .cloned()
                    .unwrap_or_else(|| effective_monitor_area(&target_for_size));

                let available_w = (target_for_size_effective.right - left).max(1);
                let available_h = (target_for_size_effective.bottom - top).max(1);
                width = resolve_size_spec(&rule.w, available_w);
                height = resolve_size_spec(&rule.h, available_h);

                target_left = left;
                target_top = top;
                target_w = width;
                target_h = height;
                is_specified = true;
                // 最後にマッチしたルールを適用するため break しない
            }
        }

        let mut should_cascade = false;

        if !is_specified {
            width = win.rect.right - win.rect.left;
            height = win.rect.bottom - win.rect.top;

            if cascade_unspecified {
                should_cascade = true;

                let assigned_monitor = find_monitor_for_pos(win.rect.left, win.rect.top, &monitors)
                    .cloned()
                    .unwrap_or_else(|| origin.clone());
                let assigned_key = monitor_key(&assigned_monitor);
                let (cursor_x, cursor_y) = cascade_cursor_by_monitor
                    .get(&assigned_key)
                    .copied()
                    .unwrap_or((origin_effective.left, origin_effective.top));

                left = cursor_x;
                top = cursor_y;

                cascade_cursor_by_monitor.insert(
                    assigned_key,
                    (cursor_x + CASCADE_OFFSET, cursor_y + CASCADE_OFFSET),
                );
            } else {
                left = win.rect.left;
                top = win.rect.top;
            }
        }

        width = width.max(1);
        height = height.max(1);

        let target_monitor = forced_target_monitor
            .clone()
            .or_else(|| find_monitor_for_pos(left, top, &monitors).cloned())
            .unwrap_or_else(|| origin.clone());
        let target = effective_by_monitor
            .get(&monitor_key(&target_monitor))
            .cloned()
            .unwrap_or_else(|| effective_monitor_area(&target_monitor));

        (left, top, width, height) = clamp_to_target_area(left, top, width, height, &target);

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

                // 初回移動で実際の外形（DPIスケーリング/非クライアント領域など）が
                // 変わるケースを吸収するため、実寸を再取得して同一処理内で再補正する。
                let mut moved_rect = RECT::default();
                let _ = GetWindowRect(hwnd, &mut moved_rect);
                let moved_w = (moved_rect.right - moved_rect.left).max(1);
                let moved_h = (moved_rect.bottom - moved_rect.top).max(1);

                let moved_monitor = find_monitor_for_pos(moved_rect.left, moved_rect.top, &monitors)
                    .cloned()
                    .unwrap_or_else(|| origin.clone());
                let moved_target = effective_by_monitor
                    .get(&monitor_key(&moved_monitor))
                    .cloned()
                    .unwrap_or_else(|| effective_monitor_area(&moved_monitor));

                let (fixed_left, fixed_top, fixed_w, fixed_h) = clamp_to_target_area(
                    moved_rect.left,
                    moved_rect.top,
                    moved_w,
                    moved_h,
                    &moved_target,
                );

                if fixed_left != moved_rect.left
                    || fixed_top != moved_rect.top
                    || fixed_w != moved_w
                    || fixed_h != moved_h
                {
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_NOTOPMOST),
                        fixed_left,
                        fixed_top,
                        fixed_w,
                        fixed_h,
                        SWP_SHOWWINDOW | SWP_NOZORDER,
                    );

                    left = fixed_left;
                    top = fixed_top;
                    width = fixed_w;
                    height = fixed_h;
                }
            }
        }

        log.push_str(&format!(
            "\"{}\",\"{}\", ({}, {}, {}, {}) -> ({}, {}, {}, {}) -> ({}, {}, {}, {})\r\n",
            regex::escape(&win.title),
            regex::escape(&win.class_name),
            old_left,
            old_top,
            old_w,
            old_h,
            target_left,
            target_top,
            target_w,
            target_h,
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

