// Toolbar: ツールバーUI（表示モード切替・月ナビゲーション・ピン留め）

use eframe::egui::Ui;

use crate::domain::calendar::{CalendarModel, ViewMode};

pub struct Toolbar;

impl Toolbar {
    /// ツールバーを描画し、操作に応じてアプリ状態を更新する
    pub fn show(
        ui: &mut Ui,
        view_mode: &mut ViewMode,
        base_year: &mut i32,
        base_month: &mut u32,
        always_on_top: &mut bool,
    ) {
        ui.horizontal(|ui| {
            // --- 表示モード切替ボタン ---
            ui.label("表示:");
            for (label, mode) in [
                ("1ヶ月", ViewMode::OneMonth),
                ("3ヶ月", ViewMode::ThreeMonths),
                ("1年", ViewMode::OneYear),
            ] {
                let selected = *view_mode == mode;
                if ui.selectable_label(selected, label).clicked() {
                    *view_mode = mode;
                }
            }

            ui.separator();

            // --- 月ナビゲーションボタン ---
            if ui.button("◀").clicked() {
                let (y, m) = CalendarModel::navigate(*base_year, *base_month, -1);
                *base_year = y;
                *base_month = m;
            }
            ui.label(format!("{}年{}月", base_year, base_month));
            if ui.button("▶").clicked() {
                let (y, m) = CalendarModel::navigate(*base_year, *base_month, 1);
                *base_year = y;
                *base_month = m;
            }

            ui.separator();

            // --- ピン留めトグルボタン ---
            let pin_label = if *always_on_top { "📌 固定中" } else { "📌 固定" };
            if ui.selectable_label(*always_on_top, pin_label).clicked() {
                *always_on_top = !*always_on_top;
            }
        });
    }
}
