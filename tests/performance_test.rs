// パフォーマンス検証テスト
// リリースビルドで目標時間内に完了することを検証する
// cargo test --release --test performance_test

use chrono::NaiveDate;
use pane_calendar::domain::business_day::BusinessDayCalc;
use pane_calendar::domain::calendar::{CalendarModel, ViewMode};
use pane_calendar::domain::holiday::HolidayService;
use std::time::Instant;

/// デバッグビルドの閾値倍率（最適化なしのため余裕を持たせる）
#[cfg(debug_assertions)]
const DEBUG_FACTOR: u128 = 50;
#[cfg(not(debug_assertions))]
const DEBUG_FACTOR: u128 = 1;

/// 1年分（365日）の祝日判定バッチ処理が 1ms 以内であること（release）
#[test]
fn holiday_batch_365days_within_1ms() {
    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let limit_us = 1_000 * DEBUG_FACTOR;

    let t = Instant::now();
    for days in 0..365 {
        let date = start_date + chrono::Duration::days(days);
        let _ = HolidayService::is_holiday(date);
    }
    let elapsed = t.elapsed();

    assert!(
        elapsed.as_micros() < limit_us,
        "365日分の祝日判定が {}μs かかった（目標: {}μs未満）",
        elapsed.as_micros(),
        limit_us,
    );
}

/// 3ヶ月→1年 表示切替が 16ms 以内であること
#[test]
fn view_mode_switch_within_16ms() {
    let limit_us = 16_000 * DEBUG_FACTOR;

    let t = Instant::now();
    let _ = CalendarModel::months_for_mode(ViewMode::ThreeMonths, 2026, 3, false);
    let _ = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 3, false);
    let elapsed = t.elapsed();

    assert!(
        elapsed.as_micros() < limit_us,
        "表示モード切替が {}μs かかった（目標: {}μs未満）",
        elapsed.as_micros(),
        limit_us,
    );
}

/// 1年表示グリッド生成（12ヶ月）が 10ms 以内であること
#[test]
fn one_year_grid_generation_within_10ms() {
    let limit_us = 10_000 * DEBUG_FACTOR;

    let t = Instant::now();
    let grids = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 1, false);
    let elapsed = t.elapsed();

    assert_eq!(grids.len(), 12);
    assert!(
        elapsed.as_micros() < limit_us,
        "1年グリッド生成が {}μs かかった（目標: {}μs未満）",
        elapsed.as_micros(),
        limit_us,
    );
}

/// 1年分の営業日算出が 1ms 以内であること
#[test]
fn business_day_calc_within_1ms() {
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
    let limit_us = 1_000 * DEBUG_FACTOR;

    let t = Instant::now();
    let _ = BusinessDayCalc::total_days(start, end);
    let _ = BusinessDayCalc::business_days(start, end);
    let elapsed = t.elapsed();

    assert!(
        elapsed.as_micros() < limit_us,
        "1年分の営業日算出が {}μs かかった（目標: {}μs未満）",
        elapsed.as_micros(),
        limit_us,
    );
}
