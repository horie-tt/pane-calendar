mod domain;
mod infrastructure;
mod ui;

use chrono::{Datelike, Local};
use eframe::egui;

use domain::calendar::ViewMode;
use domain::selection::SelectionManager;
use infrastructure::config::{Config, ConfigManager};
use infrastructure::window::WindowManager;
use ui::calendar_view::{CalendarView, DragState};
use ui::selection_overlay::SelectionOverlay;
use ui::toolbar::Toolbar;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let config = ConfigManager::load();

    // ウィンドウサイズ・位置を設定から復元（未保存時はデフォルト値）
    let window_size = config
        .window_size
        .unwrap_or([800.0, 500.0]);
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(window_size)
        .with_title("Pane Calendar")
        .with_transparent(true)
        .with_decorations(false);

    if let Some(pos) = config.window_position {
        viewport = viewport.with_position(egui::Pos2::new(pos[0], pos[1]));
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: config.window_position.is_none(), // 位置未保存時は画面中央
        ..Default::default()
    };

    eframe::run_native(
        "Pane Calendar",
        options,
        Box::new(move |cc| {
            // Windows Acrylic 透過効果を適用（失敗時はフォールバック）
            WindowManager::apply_acrylic(cc);
            Ok(Box::new(PaneCalendarApp::new(config)))
        }),
    )
}

struct PaneCalendarApp {
    view_mode: ViewMode,
    base_year: i32,
    base_month: u32,
    fiscal_year_start: bool,
    always_on_top: bool,
    selection: SelectionManager,
    drag_state: DragState,
}

impl PaneCalendarApp {
    fn new(config: Config) -> Self {
        let today = Local::now().date_naive();
        Self {
            view_mode: ViewMode::from(config.view_mode),
            base_year: today.year(),
            base_month: today.month(),
            fiscal_year_start: config.fiscal_year_start,
            always_on_top: config.always_on_top,
            selection: SelectionManager::new(),
            drag_state: DragState::default(),
        }
    }


}

impl eframe::App for PaneCalendarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ピン留め状態をウィンドウに反映
        WindowManager::apply_always_on_top(ctx, self.always_on_top);

        egui::CentralPanel::default().show(ctx, |ui| {
            // ツールバー
            Toolbar::show(
                ui,
                &mut self.view_mode,
                &mut self.base_year,
                &mut self.base_month,
                &mut self.always_on_top,
            );

            ui.separator();

            // 日数・営業日表示 + クリアボタン
            if SelectionOverlay::show_with_clear(ui, &self.selection) {
                self.selection.clear();
            }

            // カレンダー本体
            CalendarView::show(
                ui,
                self.view_mode,
                self.base_year,
                self.base_month,
                self.fiscal_year_start,
                &mut self.selection,
                &mut self.drag_state,
            );
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 終了時に現在の設定を保存する
        // NOTE: on_exit では Context が取得できないため、
        // ウィンドウ位置・サイズは保存しない（view_mode 等のみ保存）
        let config = Config {
            view_mode: self.view_mode.into(),
            window_position: None, // 位置はupdate内でのみ取得可能
            window_size: None,
            always_on_top: self.always_on_top,
            fiscal_year_start: self.fiscal_year_start,
        };
        if let Err(e) = ConfigManager::save(&config) {
            ConfigManager::write_error_log(&e);
        }
    }
}
