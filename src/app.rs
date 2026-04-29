use crate::{config, sleep_prevention, window_manager};
use eframe::egui;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

const CONFIG_FILE: &str = "PreventSleep.txt";
// スリープ防止タイマー間隔
const SLEEP_PREVENT_INTERVAL: Duration = Duration::from_secs(30);

pub struct App {
    prevent_sleep: bool,
    log: String,
    last_prevent: Instant,
    last_num_display: usize,
    power_rx: Receiver<()>,
    /// モニターON通知後に2秒遅延して再配置するためのタイムスタンプ
    pending_relocate: Option<Instant>,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        prevent_sleep: bool,
        power_rx: Receiver<()>,
    ) -> Self {
        let mut app = Self {
            prevent_sleep,
            log: String::new(),
            last_prevent: Instant::now(),
            last_num_display: 0,
            power_rx,
            pending_relocate: None,
        };

        if app.prevent_sleep {
            sleep_prevention::prevent_sleep();
        }

        // 起動時に位置を設定
        let num = window_manager::enum_monitors().len();
        app.last_num_display = num;
        app.log = window_manager::relocate_windows(
            &config::load_rules(CONFIG_FILE),
            num,
        );

        app
    }

    fn do_location_set(&mut self) {
        let num = window_manager::enum_monitors().len();
        self.last_num_display = num;
        self.log = window_manager::relocate_windows(
            &config::load_rules(CONFIG_FILE),
            num,
        );
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- タイマー処理 ---

        // スリープ防止 (30秒ごと)
        if self.prevent_sleep && self.last_prevent.elapsed() >= SLEEP_PREVENT_INTERVAL {
            sleep_prevention::prevent_sleep();
            sleep_prevention::send_mouse_move();
            self.last_prevent = Instant::now();
        }

        // スクリーン数変化の監視
        let cur_num = window_manager::enum_monitors().len();
        if cur_num != self.last_num_display && cur_num > 0 {
            self.do_location_set();
        }

        // 電源イベント受信チェック (モニターON → 2秒後に再配置)
        if let Ok(()) = self.power_rx.try_recv() {
            self.pending_relocate = Some(Instant::now());
        }
        if let Some(pending) = self.pending_relocate {
            if pending.elapsed() >= Duration::from_secs(2) {
                self.pending_relocate = None;
                self.do_location_set();
            }
        }

        // 次の repaint をスケジュール (1秒ポーリング)
        ctx.request_repaint_after(Duration::from_secs(1));

        // --- UI 描画 ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // スリープ防止チェックボックス
                if ui.checkbox(&mut self.prevent_sleep, "スリープ防止").changed() {
                    if self.prevent_sleep {
                        sleep_prevention::prevent_sleep();
                    } else {
                        sleep_prevention::release_sleep_prevention();
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Set Location").clicked() {
                    self.do_location_set();
                }
                if ui.button("List Windows").clicked() {
                    self.do_list_windows();
                }
                if ui.button("1-Display").clicked() {
                    self.last_num_display = 1;
                    self.log = window_manager::relocate_windows(
                        &config::load_rules(CONFIG_FILE),
                        1,
                    );
                }
                if ui.button("X").clicked() {
                    sleep_prevention::release_sleep_prevention();
                    self.prevent_sleep = false;
                    window_manager::turn_off_monitor();
                }
            });

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.log)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
        });
    }
}
