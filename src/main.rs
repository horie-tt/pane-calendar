mod domain;
mod infrastructure;
mod ui;

use chrono::{Datelike, Local};
use eframe::egui;

use domain::calendar::ViewMode;
use domain::selection::SelectionManager;
use infrastructure::config::ConfigManager;
use ui::calendar_view::{CalendarView, DragState};
use ui::selection_overlay::SelectionOverlay;
use ui::toolbar::Toolbar;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let config = ConfigManager::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 500.0])
            .with_title("Pane Calendar"),
        ..Default::default()
    };

    eframe::run_native(
        "Pane Calendar",
        options,
        Box::new(move |_cc| Ok(Box::new(PaneCalendarApp::new(config)))),
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
    fn new(config: infrastructure::config::Config) -> Self {
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
        let level = if self.always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));

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
}
