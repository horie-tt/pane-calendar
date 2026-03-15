// 統合テスト: 設定 / 選択+営業日算出 / 表示モード切替

use chrono::NaiveDate;
use pane_calendar::domain::business_day::BusinessDayCalc;
use pane_calendar::domain::calendar::{CalendarModel, ViewMode};
use pane_calendar::domain::selection::SelectionManager;
use pane_calendar::infrastructure::config::{Config, ConfigManager, ViewModeConfig};

// -------------------------------------------------------------------
// 1. 設定ファイルの書き込み→読み込みラウンドトリップ
// -------------------------------------------------------------------

#[test]
fn config_roundtrip_preserves_all_fields() {
    let dir = std::env::temp_dir();
    let path = dir.join("pane_calendar_integration_roundtrip.toml");

    let original = Config {
        view_mode: ViewModeConfig::OneYear,
        window_position: Some([300.0, 150.0]),
        window_size: Some([1024.0, 768.0]),
        always_on_top: true,
        fiscal_year_start: true,
    };

    ConfigManager::save_to(&original, &path).expect("設定の保存に失敗");
    let loaded = ConfigManager::load_from(&path);

    assert_eq!(loaded, original, "ラウンドトリップ後に値が一致すること");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn config_roundtrip_default_values() {
    let dir = std::env::temp_dir();
    let path = dir.join("pane_calendar_integration_default.toml");

    let original = Config::default();
    ConfigManager::save_to(&original, &path).expect("設定の保存に失敗");
    let loaded = ConfigManager::load_from(&path);

    assert_eq!(loaded, original);
    let _ = std::fs::remove_file(&path);
}

// -------------------------------------------------------------------
// 2. 選択範囲 → 営業日算出の連携
// -------------------------------------------------------------------

#[test]
fn selection_to_business_day_calculation() {
    let mut sel = SelectionManager::new();

    // 2026年1月5日（月）〜 2026年1月9日（金）: 5日 / 5営業日
    let start = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 1, 9).unwrap();

    sel.set_drag_range(start, end);

    // 選択状態の確認
    assert!(sel.is_selected(start));
    assert!(sel.is_selected(end));

    // 営業日算出との連携
    let total = BusinessDayCalc::total_days(start, end);
    let business = BusinessDayCalc::business_days(start, end);

    assert_eq!(total, 5, "月〜金の5日間");
    assert_eq!(business, 5, "全日が営業日");
}

#[test]
fn selection_range_with_weekend_and_holiday() {
    let mut sel = SelectionManager::new();

    // 2026年1月1日（木・元日）〜 2026年1月4日（日）: 4日 / 1営業日
    // 1/1: 元日（祝日・木曜） → 非営業日
    // 1/2: 金曜（平日） → 営業日
    // 1/3: 土曜 → 非営業日
    // 1/4: 日曜 → 非営業日
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();

    sel.set_drag_range(start, end);

    let total = BusinessDayCalc::total_days(start, end);
    let business = BusinessDayCalc::business_days(start, end);

    assert_eq!(total, 4);
    assert_eq!(business, 1, "元日(非営業)+金曜(営業)+土日(非営業) = 1営業日");
}

#[test]
fn selection_clear_resets_state() {
    let mut sel = SelectionManager::new();

    let start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
    sel.set_drag_range(start, end);

    assert!(sel.is_selected(start));
    sel.clear();
    assert!(!sel.is_selected(start), "クリア後は選択が解除される");
}

// -------------------------------------------------------------------
// 3. 表示モード切替とカレンダーモデルの連携
// -------------------------------------------------------------------

#[test]
fn view_mode_one_month_returns_single_grid() {
    let grids = CalendarModel::months_for_mode(ViewMode::OneMonth, 2026, 3, false);
    assert_eq!(grids.len(), 1);
    assert_eq!(grids[0].year, 2026);
    assert_eq!(grids[0].month, 3);
}

#[test]
fn view_mode_three_months_returns_correct_grids() {
    // 3ヶ月表示: 前月・今月・翌月
    let grids = CalendarModel::months_for_mode(ViewMode::ThreeMonths, 2026, 3, false);
    assert_eq!(grids.len(), 3);
    assert_eq!((grids[0].year, grids[0].month), (2026, 2));
    assert_eq!((grids[1].year, grids[1].month), (2026, 3));
    assert_eq!((grids[2].year, grids[2].month), (2026, 4));
}

#[test]
fn view_mode_one_year_returns_12_grids() {
    let grids = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 3, false);
    assert_eq!(grids.len(), 12);
    for (i, grid) in grids.iter().enumerate() {
        assert_eq!(grid.month as usize, i + 1, "1月〜12月の順");
    }
}

#[test]
fn view_mode_one_year_fiscal_start_from_april() {
    // 2026年3月基準・年度始まり有効 → 3月は4月より前なので前年度 (2025年4月〜2026年3月)
    let grids = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 3, true);
    assert_eq!(grids.len(), 12);
    assert_eq!(grids[0].month, 4, "年度始まりは4月");
    assert_eq!(grids[0].year, 2025);
    assert_eq!(grids[11].month, 3, "年度末は3月");
    assert_eq!(grids[11].year, 2026);
}

#[test]
fn layout_changes_with_view_mode() {
    let one = CalendarModel::layout_for_mode(ViewMode::OneMonth);
    let three = CalendarModel::layout_for_mode(ViewMode::ThreeMonths);
    let year = CalendarModel::layout_for_mode(ViewMode::OneYear);

    assert_eq!((one.columns, one.rows), (1, 1));
    assert_eq!((three.columns, three.rows), (3, 1));
    assert_eq!((year.columns, year.rows), (3, 4));
}

#[test]
fn view_mode_switch_year_end_three_months() {
    // 12月基準の3ヶ月表示: 翌年1月・2月が含まれること
    let grids = CalendarModel::months_for_mode(ViewMode::ThreeMonths, 2025, 12, false);
    assert_eq!(grids.len(), 3);
    assert_eq!((grids[0].year, grids[0].month), (2025, 11));
    assert_eq!((grids[1].year, grids[1].month), (2025, 12));
    assert_eq!((grids[2].year, grids[2].month), (2026, 1));
}
