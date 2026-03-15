// CalendarView: 月グリッド描画・クリック/ドラッグ操作・表示モード切替

use chrono::{Datelike, Local, NaiveDate, Weekday};
use eframe::egui::{self, Color32, PointerButton, Sense, Vec2};

use crate::domain::calendar::{CalendarModel, MonthGrid, ViewMode};
use crate::domain::holiday::HolidayService;
use crate::domain::selection::SelectionManager;
use crate::ui::theme::Theme;

/// 曜日ヘッダーラベル（日曜始まり）
const WEEKDAY_LABELS: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];

/// ドラッグ状態（月を跨ぐドラッグのためApp側で保持）
#[derive(Default)]
pub struct DragState {
    pub dragging: bool,
    pub start_date: Option<NaiveDate>,
}

pub struct CalendarView;

impl CalendarView {
    /// カレンダー全体を描画する。クリック/ドラッグ操作を検出し selection を更新する
    pub fn show(
        ui: &mut eframe::egui::Ui,
        mode: ViewMode,
        base_year: i32,
        base_month: u32,
        fiscal_year_start: bool,
        selection: &mut SelectionManager,
        drag_state: &mut DragState,
    ) {
        let grids = CalendarModel::months_for_mode(mode, base_year, base_month, fiscal_year_start);
        let layout = CalendarModel::layout_for_mode(mode);
        let today = Local::now().date_naive();

        for row in 0..layout.rows {
            ui.horizontal(|ui| {
                for col in 0..layout.columns {
                    let idx = row * layout.columns + col;
                    if let Some(grid) = grids.get(idx) {
                        Self::show_month(ui, grid, today, selection, drag_state);
                    }
                }
            });
        }
    }

    /// 1ヶ月分のグリッドを描画する
    fn show_month(
        ui: &mut eframe::egui::Ui,
        grid: &MonthGrid,
        today: NaiveDate,
        selection: &mut SelectionManager,
        drag_state: &mut DragState,
    ) {
        let cell_size = Vec2::new(36.0, 28.0);
        let total_width = cell_size.x * 7.0;
        let header_height = 24.0;

        ui.vertical(|ui| {
            Self::show_month_header(ui, grid, total_width, header_height);
            Self::show_weekday_header(ui, cell_size);

            for week in &grid.weeks {
                ui.horizontal(|ui| {
                    for day_opt in week.iter() {
                        Self::show_day_cell(ui, *day_opt, grid, today, selection, drag_state, cell_size);
                    }
                });
            }
        });
    }

    /// 月ヘッダーを描画する（例: "2026年3月"）
    fn show_month_header(
        ui: &mut eframe::egui::Ui,
        grid: &MonthGrid,
        width: f32,
        height: f32,
    ) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 2.0, Theme::HEADER_BG);
            let label = format!("{}年{}月", grid.year, grid.month);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(13.0),
                Theme::TEXT_NORMAL,
            );
        }
    }

    /// 曜日ヘッダー行を描画する（日〜土）
    fn show_weekday_header(ui: &mut eframe::egui::Ui, cell_size: Vec2) {
        ui.horizontal(|ui| {
            for (i, label) in WEEKDAY_LABELS.iter().enumerate() {
                let color = match i {
                    0 => Theme::TEXT_SUNDAY,
                    6 => Theme::TEXT_SATURDAY,
                    _ => Theme::TEXT_NORMAL,
                };
                let (rect, _) = ui.allocate_exact_size(cell_size, Sense::hover());
                if ui.is_rect_visible(rect) {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *label,
                        egui::FontId::proportional(11.0),
                        color,
                    );
                }
            }
        });
    }

    /// 日付セルを1つ描画し、クリック/ドラッグを検出して selection を更新する
    fn show_day_cell(
        ui: &mut eframe::egui::Ui,
        day_opt: Option<u32>,
        grid: &MonthGrid,
        today: NaiveDate,
        selection: &mut SelectionManager,
        drag_state: &mut DragState,
        cell_size: Vec2,
    ) {
        // 空セル（その月に属さない日）
        let Some(day) = day_opt else {
            let (rect, _) = ui.allocate_exact_size(cell_size, Sense::hover());
            ui.painter().rect_filled(rect, 0.0, Color32::TRANSPARENT);
            return;
        };

        let date = NaiveDate::from_ymd_opt(grid.year, grid.month, day)
            .expect("invalid date in grid");

        // クリック・ドラッグ両対応のSense
        let (rect, response) = ui.allocate_exact_size(cell_size, Sense::click_and_drag());

        // --- インタラクション処理 ---
        if response.drag_started_by(PointerButton::Primary) {
            // ドラッグ開始: 既存選択をクリアして開始日を記録
            drag_state.dragging = true;
            drag_state.start_date = Some(date);
            selection.set_drag_range(date, date);
        } else if drag_state.dragging && response.dragged_by(PointerButton::Primary) {
            // ドラッグ中: プレビュー範囲を更新
            if let Some(start) = drag_state.start_date {
                selection.update_drag_preview(date);
                selection.set_drag_range(start, date);
            }
        } else if drag_state.dragging && response.drag_stopped() {
            // ドラッグ終了: 範囲を確定
            if let Some(start) = drag_state.start_date {
                selection.set_drag_range(start, date);
            }
            drag_state.dragging = false;
            drag_state.start_date = None;
        } else if response.clicked_by(PointerButton::Primary) && !drag_state.dragging {
            // クリック: トグル選択
            selection.toggle_date(date);
        }

        // --- 描画 ---
        if !ui.is_rect_visible(rect) {
            return;
        }

        let is_today = date == today;
        let is_selected = selection.is_selected(date);
        let is_holiday = HolidayService::is_holiday(date);
        let weekday = date.weekday();

        let bg_color = if is_selected {
            Theme::BG_SELECTED
        } else if is_today {
            Theme::BG_TODAY
        } else if is_holiday || weekday == Weekday::Sun {
            Theme::BG_SUNDAY
        } else if weekday == Weekday::Sat {
            Theme::BG_SATURDAY
        } else {
            Theme::BG_NORMAL
        };

        let text_color = if is_today {
            Theme::TEXT_TODAY
        } else if is_holiday || weekday == Weekday::Sun {
            Theme::TEXT_SUNDAY
        } else if weekday == Weekday::Sat {
            Theme::TEXT_SATURDAY
        } else {
            Theme::TEXT_NORMAL
        };

        // ホバー時の軽いハイライト
        let bg_color = if response.hovered() && !is_selected {
            egui::Color32::from_rgba_premultiplied(
                bg_color.r().saturating_add(20),
                bg_color.g().saturating_add(20),
                bg_color.b().saturating_add(20),
                bg_color.a(),
            )
        } else {
            bg_color
        };

        ui.painter().rect_filled(rect, 2.0, bg_color);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            day.to_string(),
            egui::FontId::proportional(12.0),
            text_color,
        );
    }
}
