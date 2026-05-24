use crate::{config, hotkey::{self, HotkeyAction}, sleep_prevention, window_manager};
use eframe::egui;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

const CONFIG_FILE: &str = "PreventSleep.txt";
// スリープ防止タイマー間隔
const SLEEP_PREVENT_INTERVAL: Duration = Duration::from_secs(30);
const LOG_BOX_WIDTH: f32 = 430.0;
const LOG_BOX_HEIGHT: f32 = 95.0;

pub struct App {
    prevent_sleep: bool,
    log: String,
    last_prevent: Instant,
    last_num_display: usize,
    power_rx: Receiver<()>,
    hotkey_rx: Receiver<HotkeyAction>,
    /// モニターON通知後に2秒遅延して再配置するためのタイムスタンプ
    pending_relocate: Option<Instant>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        prevent_sleep: bool,
        power_rx: Receiver<()>,
        hotkey_rx: Receiver<HotkeyAction>,
    ) -> Self {
        setup_japanese_fonts(&cc.egui_ctx);

        // ホットキー発火時に即 repaint するため Context を共有
        hotkey::set_egui_context(cc.egui_ctx.clone());

        let mut app = Self {
            prevent_sleep,
            log: String::new(),
            last_prevent: Instant::now(),
            last_num_display: 0,
            power_rx,
            hotkey_rx,
            pending_relocate: None,
        };

        if app.prevent_sleep {
            sleep_prevention::prevent_sleep();
        }

        // 起動時に位置を設定
        let num = window_manager::enum_monitors().len();
        app.last_num_display = num;
        app.log = format_log_with_config_path(window_manager::relocate_windows(
            &config::load_rules(CONFIG_FILE),
            num,
        ));

        app
    }

    fn do_location_set(&mut self, _ctx: &egui::Context) {
        let num = window_manager::enum_monitors().len();
        self.last_num_display = num;
        self.log = format_log_with_config_path(window_manager::relocate_windows(
            &config::load_rules(CONFIG_FILE),
            num,
        ));
        // 画面座標系の差異による画面外移動を避けるため、自ウィンドウは Win32 側で再配置する
        window_manager::relocate_preventsleep_window_to_origin_bottom_left();
    }

    fn do_location_set_cascading(&mut self, _ctx: &egui::Context) {
        let num = window_manager::enum_monitors().len();
        self.last_num_display = num;
        self.log = format_log_with_config_path(window_manager::relocate_windows_cascading(
            &config::load_rules(CONFIG_FILE),
            num,
        ));
        // 画面座標系の差異による画面外移動を避けるため、自ウィンドウは Win32 側で再配置する
        window_manager::relocate_preventsleep_window_to_origin_bottom_left();
    }

    fn do_list_windows(&mut self) {
        let windows = window_manager::enum_windows_list();
        let mut out = String::new();
        for w in &windows {
            let left = w.rect.left;
            let top = w.rect.top;
            let width = w.rect.right - w.rect.left;
            let height = w.rect.bottom - w.rect.top;
            out.push_str(&format!(
                "\"{}\",\"{}\", {}, {}, {}, {}\r\n",
                regex::escape(&w.title),
                regex::escape(&w.class_name),
                left,
                top,
                width,
                height,
            ));
        }
        self.log = if out.is_empty() {
            "ウィンドウなし".to_string()
        } else {
            out
        };
    }

}

fn format_log_with_config_path(log_body: String) -> String {
    let config_path = config::resolve_rules_path(CONFIG_FILE);
    format!("# {}\r\n{}", config_path.display(), log_body)
}

fn setup_japanese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Windows 環境の代表的な日本語フォント候補を順に試す
    let candidates = [
        r"C:\Windows\Fonts\YuGothR.ttc",
        r"C:\Windows\Fonts\meiryo.ttc",
        r"C:\Windows\Fonts\msgothic.ttc",
    ];

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let font_name = "jp_font".to_string();
            fonts
                .font_data
                .insert(font_name.clone(), egui::FontData::from_owned(bytes).into());

            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, font_name.clone());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.insert(0, font_name);
            }

            ctx.set_fonts(fonts);
            return;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 状態監視処理 ---

        // スクリーン数変化の監視
        let cur_num = window_manager::enum_monitors().len();
        if cur_num != self.last_num_display && cur_num > 0 {
            self.do_location_set(ctx);
        }

        // 電源イベント受信チェック (モニターON → 2秒後に再配置)
        if let Ok(()) = self.power_rx.try_recv() {
            self.pending_relocate = Some(Instant::now());
        }
        if let Some(pending) = self.pending_relocate {
            if pending.elapsed() >= Duration::from_secs(2) {
                self.pending_relocate = None;
                self.do_location_set(ctx);
            }
        }

        // グローバルホットキーイベント受信
        while let Ok(action) = self.hotkey_rx.try_recv() {
            match action {
                HotkeyAction::Apply => {
                    self.do_location_set(ctx);
                    self.log = format!(
                        "[ショートカット] Ctrl+Alt+Z で配置適用を実行\r\n{}",
                        self.log
                    );
                }
                HotkeyAction::Cascade => {
                    self.do_location_set_cascading(ctx);
                    self.log = format!(
                        "[ショートカット] Ctrl+Alt+X で階段配置を実行\r\n{}",
                        self.log
                    );
                }
            }
        }

        // --- UI 描画 ---
        egui::CentralPanel::default().show(ctx, |ui| {
            // 固定サイズのログテキストボックス + スクロールバー
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(LOG_BOX_WIDTH);
                egui::ScrollArea::both()
                    .id_salt("log_scroll")
                    .max_width(LOG_BOX_WIDTH)
                    .max_height(LOG_BOX_HEIGHT)
                    .show(ui, |ui| {
                        let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
                            let layout_job = egui::text::LayoutJob::simple(
                                text.to_owned(),
                                egui::TextStyle::Monospace.resolve(ui.style()),
                                ui.visuals().text_color(),
                                f32::INFINITY,
                            );
                            ui.fonts(|f| f.layout_job(layout_job))
                        };
                        ui.add(
                            egui::TextEdit::multiline(&mut self.log)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        );
                    });
            });

            ui.add_space(4.0);

            // テキストボックスの下にチェックボックス
            ui.horizontal(|ui| {
                let previous = self.prevent_sleep;
                ui.checkbox(&mut self.prevent_sleep, "スリープ防止");

                if self.prevent_sleep != previous {
                    if self.prevent_sleep {
                        sleep_prevention::prevent_sleep();
                        self.last_prevent = Instant::now();
                    } else {
                        sleep_prevention::release_sleep_prevention();
                    }
                }
            });

            ui.add_space(2.0);

            // ボタン類はテキストボックスの下
            ui.horizontal(|ui| {
                if ui.button("配置適用").clicked() {
                    self.do_location_set(ctx);
                }
                if ui.button("階段配置").clicked() {
                    self.do_location_set_cascading(ctx);
                }
                if ui.button("１画面配置").clicked() {
                    // last_num_display は実際のモニタ数を維持する。
                    // ここで 1 に書き換えるとモニタ数変化監視ロジックが誤発火し、
                    // 直後に do_location_set（通常の配置適用）が呼ばれてしまう。
                    self.log = format_log_with_config_path(window_manager::relocate_windows_single_screen(
                        &config::load_rules(CONFIG_FILE),
                    ));
                    window_manager::relocate_preventsleep_window_to_origin_bottom_left();
                }
                if ui.button("ウィンドウ一覧").clicked() {
                    self.do_list_windows();
                }
                if ui.button("モニターOFF").clicked() {
                    self.prevent_sleep = false;
                    sleep_prevention::release_sleep_prevention();
                    std::thread::spawn(|| {
                        window_manager::turn_off_monitor();
                    });
                }
            });
        });

        // --- タイマー処理 ---

        // スリープ防止 (30秒ごと / 有効時のみ)
        if self.prevent_sleep && self.last_prevent.elapsed() >= SLEEP_PREVENT_INTERVAL {
            sleep_prevention::send_mouse_move();
            self.last_prevent = Instant::now();
        }

        // 次の repaint をスケジュール (1秒ポーリング)
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}
