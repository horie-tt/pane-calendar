mod domain;
mod ui;
mod infrastructure;

use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 400.0])
            .with_title("Pane Calendar"),
        ..Default::default()
    };

    eframe::run_native(
        "Pane Calendar",
        options,
        Box::new(|_cc| Ok(Box::new(PaneCalendarApp::default()))),
    )
}

#[derive(Default)]
struct PaneCalendarApp;

impl eframe::App for PaneCalendarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pane Calendar");
            ui.label("カレンダーアプリ起動確認");
        });
    }
}
