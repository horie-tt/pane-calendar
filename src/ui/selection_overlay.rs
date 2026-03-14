// SelectionOverlay: 選択日数・営業日数の表示

use eframe::egui::Ui;

use crate::domain::business_day::BusinessDayCalc;
use crate::domain::selection::{SelectionManager, SelectionMode};

pub struct SelectionOverlay;

impl SelectionOverlay {
    /// 日数・営業日数を表示しクリアボタンを提供する。
    /// Range モード時のみ表示。Individual モードでは非表示（false を返す）。
    /// クリアボタンが押された場合 true を返す。
    pub fn show_with_clear(ui: &mut Ui, selection: &SelectionManager) -> bool {
        match selection.mode() {
            SelectionMode::Range { start, end } => {
                let total = BusinessDayCalc::total_days(*start, *end);
                let business = BusinessDayCalc::business_days(*start, *end);
                let mut cleared = false;
                ui.horizontal(|ui| {
                    ui.label(
                        eframe::egui::RichText::new(
                            format!("{}日  {}営業日", total, business)
                        ).strong()
                    );
                    if ui.button("✕ クリア").clicked() {
                        cleared = true;
                    }
                });
                cleared
            }
            SelectionMode::Individual => false,
        }
    }
}
