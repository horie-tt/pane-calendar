mod domain;
mod infrastructure;
mod ui;

use chrono::{Datelike, Local};
use eframe::egui;

use domain::calendar::ViewMode;
use domain::selection::SelectionManager;
use infrastructure::config::ConfigManager;
use ui::calendar_view::CalendarView;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let config = ConfigManager::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 400.0])
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
    selection: SelectionManager,
}

impl PaneCalendarApp {
    fn new(config: infrastructure::config::Config) -> Self {
        let today = Local::now().date_naive();
        Self {
            view_mode: ViewMode::from(config.view_mode),
            base_year: today.year(),
            base_month: today.month(),
            fiscal_year_start: config.fiscal_year_start,
            selection: SelectionManager::new(),
        }
    }
}

impl eframe::App for PaneCalendarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            CalendarView::show(
                ui,
                self.view_mode,
                self.base_year,
                self.base_month,
                self.fiscal_year_start,
                &self.selection,
            );
        });
    }
}
