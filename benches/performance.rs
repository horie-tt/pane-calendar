// パフォーマンスベンチマーク (criterion)
// cargo bench で実行する

use chrono::NaiveDate;
use criterion::{criterion_group, criterion_main, Criterion};
use pane_calendar::domain::business_day::BusinessDayCalc;
use pane_calendar::domain::calendar::{CalendarModel, ViewMode};
use pane_calendar::domain::holiday::HolidayService;

/// 1年分（365日）の祝日判定バッチ処理（目標: 1ms以内）
fn bench_holiday_batch(c: &mut Criterion) {
    c.bench_function("holiday_batch_365days", |b| {
        b.iter(|| {
            let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            for days in 0..365 {
                let date = start + chrono::Duration::days(days);
                let _ = HolidayService::is_holiday(date);
            }
        });
    });
}

/// 3ヶ月→1年 表示切替（月グリッド生成、目標: 16ms以内）
fn bench_view_mode_switch(c: &mut Criterion) {
    c.bench_function("view_mode_switch_three_to_year", |b| {
        b.iter(|| {
            let _ = CalendarModel::months_for_mode(ViewMode::ThreeMonths, 2026, 3, false);
            let _ = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 3, false);
        });
    });
}

/// 1年表示グリッド生成（12ヶ月分）
fn bench_one_year_grid(c: &mut Criterion) {
    c.bench_function("one_year_grid_generation", |b| {
        b.iter(|| {
            let _ = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 1, false);
        });
    });
}

/// 1ヶ月分の営業日算出
fn bench_business_day_calc(c: &mut Criterion) {
    c.bench_function("business_days_one_month", |b| {
        b.iter(|| {
            let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
            let _ = BusinessDayCalc::total_days(start, end);
            let _ = BusinessDayCalc::business_days(start, end);
        });
    });
}

criterion_group!(
    benches,
    bench_holiday_batch,
    bench_view_mode_switch,
    bench_one_year_grid,
    bench_business_day_calc,
);
criterion_main!(benches);
