#![windows_subsystem = "windows"]

mod app;
mod config;
mod power_monitor;
mod sleep_prevention;
mod window_manager;
mod hotkey;

use std::sync::mpsc;
use window_manager::turn_off_monitor;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "set" => {
                let rules = config::load_rules("PreventSleep.txt");
                let num = window_manager::enum_monitors().len();
                window_manager::relocate_windows(&rules, num);
                window_manager::relocate_preventsleep_window_to_origin_bottom_left();
                return;
            }
            "monitoroff" => {
                turn_off_monitor();
                return;
            }
            _ => {}
        }
    }

    // "noprevent" オプション
    let prevent_sleep = args.get(1).map(|a| a.as_str()) != Some("noprevent");

    // 電源監視スレッド起動
    let (tx, rx) = mpsc::channel::<()>();
    power_monitor::start_power_monitor(tx);

    // グローバルホットキー監視スレッド起動
    // Ctrl+Alt+Z: 配置適用 / Ctrl+Alt+X: 階段配置
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<hotkey::HotkeyAction>();
    hotkey::run_global_hotkeys(hotkey_tx);

    // egui ウィンドウ設定
    // 左下に配置するための初期位置を計算
    let win_width = 460.0_f32;
    let win_height = 190.0_f32;
    let (init_x, init_y) = window_manager::preventsleep_window_origin_bottom_left_position();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PreventSleep v2.4.1")
            .with_inner_size([win_width, win_height])
            .with_min_inner_size([win_width, win_height])
            .with_max_inner_size([win_width, win_height])
            .with_resizable(false)
            .with_position([init_x, init_y])
            .with_icon(load_icon()),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "PreventSleep",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, prevent_sleep, rx, hotkey_rx)))),
    );
}

fn load_icon() -> egui::IconData {
    let bytes = include_bytes!("../assets/app.ico");
    let cursor = std::io::Cursor::new(bytes.as_slice());

    if let Ok(icon_dir) = ico::IconDir::read(cursor) {
        if let Some(entry) = icon_dir
            .entries()
            .iter()
            .max_by_key(|e| e.width() * e.height())
        {
            if let Ok(image) = entry.decode() {
                return egui::IconData {
                    rgba: image.rgba_data().to_vec(),
                    width: image.width(),
                    height: image.height(),
                };
            }
        }
    }

    egui::IconData {
        rgba: vec![0u8; 4],
        width: 1,
        height: 1,
    }
}
