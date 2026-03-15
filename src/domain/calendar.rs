// CalendarModel: 表示モードと月グリッド生成

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    OneMonth,
    ThreeMonths,
    OneYear,
}

/// 各月のカレンダーグリッド（日曜始まり）
#[derive(Debug, Clone, PartialEq)]
pub struct MonthGrid {
    pub year: i32,
    pub month: u32,
    /// 各週の日付配列。index 0=日, 1=月, ..., 6=土。その月に属さないセルは None
    pub weeks: Vec<[Option<u32>; 7]>,
}

/// 表示レイアウト（列数×行数）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLayout {
    pub columns: usize,
    pub rows: usize,
}

pub struct CalendarModel;

impl CalendarModel {
    /// 表示モードと基準年月から表示対象月のグリッド一覧を返す
    pub fn months_for_mode(
        mode: ViewMode,
        base_year: i32,
        base_month: u32,
        fiscal_year_start: bool,
    ) -> Vec<MonthGrid> {
        let months = Self::month_list(mode, base_year, base_month, fiscal_year_start);
        months
            .into_iter()
            .map(|(y, m)| Self::build_month_grid(y, m))
            .collect()
    }

    /// 表示モードに対応するグリッドレイアウトを返す
    pub fn layout_for_mode(mode: ViewMode) -> GridLayout {
        match mode {
            ViewMode::OneMonth => GridLayout { columns: 1, rows: 1 },
            ViewMode::ThreeMonths => GridLayout { columns: 3, rows: 1 },
            ViewMode::OneYear => GridLayout { columns: 3, rows: 4 },
        }
    }

    /// 表示対象の (year, month) リストを返す
    fn month_list(
        mode: ViewMode,
        base_year: i32,
        base_month: u32,
        fiscal_year_start: bool,
    ) -> Vec<(i32, u32)> {
        match mode {
            ViewMode::OneMonth => vec![(base_year, base_month)],
            ViewMode::ThreeMonths => {
                let prev = Self::add_months(base_year, base_month, -1);
                let next = Self::add_months(base_year, base_month, 1);
                vec![prev, (base_year, base_month), next]
            }
            ViewMode::OneYear => {
                let start_month = if fiscal_year_start { 4u32 } else { 1u32 };
                let start_year = if fiscal_year_start && base_month < 4 {
                    base_year - 1
                } else {
                    base_year
                };
                (0..12)
                    .map(|i| Self::add_months(start_year, start_month, i))
                    .collect()
            }
        }
    }

    /// 月ナビゲーション（delta: 正=次月方向, 負=前月方向）
    pub fn navigate(year: i32, month: u32, delta: i32) -> (i32, u32) {
        Self::add_months(year, month, delta)
    }

    /// 月の加算（正負どちらも可）
    fn add_months(year: i32, month: u32, delta: i32) -> (i32, u32) {
        let total = (year * 12 + (month as i32 - 1)) + delta;
        let new_year = total.div_euclid(12);
        let new_month = (total.rem_euclid(12) + 1) as u32;
        (new_year, new_month)
    }

    /// 指定年月の MonthGrid を生成する（日曜始まり）
    fn build_month_grid(year: i32, month: u32) -> MonthGrid {
        use chrono::{Datelike, NaiveDate, Weekday};

        let first_day = NaiveDate::from_ymd_opt(year, month, 1).expect("invalid date");
        let days_in_month = Self::days_in_month(year, month);

        // 日曜始まりでの最初の日の列インデックス（日=0, 月=1, ..., 土=6）
        let first_col = match first_day.weekday() {
            Weekday::Sun => 0,
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
        };

        let mut weeks: Vec<[Option<u32>; 7]> = Vec::new();
        let mut week = [None; 7];
        let mut col = first_col;

        for day in 1..=days_in_month {
            week[col] = Some(day);
            if col == 6 {
                weeks.push(week);
                week = [None; 7];
                col = 0;
            } else {
                col += 1;
            }
        }
        // 最終週が未完の場合は追加
        if col != 0 {
            weeks.push(week);
        }

        MonthGrid { year, month, weeks }
    }

    /// 指定年月の日数を返す
    fn days_in_month(year: i32, month: u32) -> u32 {
        let next = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
        };
        let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("invalid date");
        let next = next.expect("invalid date");
        (next - first).num_days() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ViewMode別・月リスト生成テスト ---

    #[test]
    fn one_month_returns_single_month() {
        let grids = CalendarModel::months_for_mode(ViewMode::OneMonth, 2026, 3, false);
        assert_eq!(grids.len(), 1);
        assert_eq!(grids[0].year, 2026);
        assert_eq!(grids[0].month, 3);
    }

    #[test]
    fn three_months_returns_prev_current_next() {
        let grids = CalendarModel::months_for_mode(ViewMode::ThreeMonths, 2026, 3, false);
        assert_eq!(grids.len(), 3);
        assert_eq!((grids[0].year, grids[0].month), (2026, 2)); // 前月
        assert_eq!((grids[1].year, grids[1].month), (2026, 3)); // 今月（中央）
        assert_eq!((grids[2].year, grids[2].month), (2026, 4)); // 翌月
    }

    #[test]
    fn three_months_wraps_year_end() {
        // 12月基準: 前月=11月, 今月=12月, 翌月=翌年1月
        let grids = CalendarModel::months_for_mode(ViewMode::ThreeMonths, 2025, 12, false);
        assert_eq!(grids.len(), 3);
        assert_eq!((grids[0].year, grids[0].month), (2025, 11));
        assert_eq!((grids[1].year, grids[1].month), (2025, 12));
        assert_eq!((grids[2].year, grids[2].month), (2026, 1));
    }

    #[test]
    fn three_months_wraps_year_start() {
        // 1月基準: 前月=前年12月, 今月=1月, 翌月=2月
        let grids = CalendarModel::months_for_mode(ViewMode::ThreeMonths, 2026, 1, false);
        assert_eq!(grids.len(), 3);
        assert_eq!((grids[0].year, grids[0].month), (2025, 12));
        assert_eq!((grids[1].year, grids[1].month), (2026, 1));
        assert_eq!((grids[2].year, grids[2].month), (2026, 2));
    }

    #[test]
    fn one_year_returns_12_months_jan_to_dec() {
        let grids = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 6, false);
        assert_eq!(grids.len(), 12);
        assert_eq!((grids[0].year, grids[0].month), (2026, 1));
        assert_eq!((grids[11].year, grids[11].month), (2026, 12));
    }

    #[test]
    fn one_year_fiscal_returns_apr_to_mar() {
        // 年度始まり有効、基準月=6月 → 4月〜翌3月
        let grids = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 6, true);
        assert_eq!(grids.len(), 12);
        assert_eq!((grids[0].year, grids[0].month), (2026, 4));
        assert_eq!((grids[11].year, grids[11].month), (2027, 3));
    }

    #[test]
    fn one_year_fiscal_before_april_uses_prev_year() {
        // 年度始まり有効、基準月=3月 → 前年4月〜今年3月
        let grids = CalendarModel::months_for_mode(ViewMode::OneYear, 2026, 3, true);
        assert_eq!(grids.len(), 12);
        assert_eq!((grids[0].year, grids[0].month), (2025, 4));
        assert_eq!((grids[11].year, grids[11].month), (2026, 3));
    }

    // --- レイアウトテスト ---

    #[test]
    fn layout_one_month_is_1x1() {
        let layout = CalendarModel::layout_for_mode(ViewMode::OneMonth);
        assert_eq!(layout.columns, 1);
        assert_eq!(layout.rows, 1);
    }

    #[test]
    fn layout_three_months_is_3x1() {
        let layout = CalendarModel::layout_for_mode(ViewMode::ThreeMonths);
        assert_eq!(layout.columns, 3);
        assert_eq!(layout.rows, 1);
    }

    #[test]
    fn layout_one_year_is_3x4() {
        let layout = CalendarModel::layout_for_mode(ViewMode::OneYear);
        assert_eq!(layout.columns, 3);
        assert_eq!(layout.rows, 4);
    }

    // --- 月グリッド生成テスト ---

    #[test]
    fn month_grid_2026_march_starts_on_sunday() {
        // 2026年3月1日は日曜日
        let grids = CalendarModel::months_for_mode(ViewMode::OneMonth, 2026, 3, false);
        let grid = &grids[0];
        // 第1週のindex 0（日曜）が1日
        assert_eq!(grid.weeks[0][0], Some(1));
        // 第1週のindex 6（土曜）が7日
        assert_eq!(grid.weeks[0][6], Some(7));
    }

    #[test]
    fn month_grid_has_correct_days_in_month() {
        // 2026年2月は28日まで
        let grids = CalendarModel::months_for_mode(ViewMode::OneMonth, 2026, 2, false);
        let grid = &grids[0];
        let all_days: Vec<u32> = grid
            .weeks
            .iter()
            .flat_map(|w| w.iter())
            .filter_map(|d| *d)
            .collect();
        assert_eq!(all_days.len(), 28);
        assert_eq!(*all_days.first().unwrap(), 1);
        assert_eq!(*all_days.last().unwrap(), 28);
    }

    // --- 月ナビゲーションテスト ---

    #[test]
    fn navigate_prev_month_from_jan_wraps_to_dec() {
        // 2026年1月から前月移動 → 2025年12月
        assert_eq!(CalendarModel::navigate(2026, 1, -1), (2025, 12));
    }

    #[test]
    fn navigate_next_month_from_dec_wraps_to_jan() {
        // 2025年12月から次月移動 → 2026年1月
        assert_eq!(CalendarModel::navigate(2025, 12, 1), (2026, 1));
    }

    #[test]
    fn navigate_multiple_steps_forward() {
        // 2026年1月から3ヶ月後 → 2026年4月
        assert_eq!(CalendarModel::navigate(2026, 1, 3), (2026, 4));
    }

    #[test]
    fn navigate_multiple_steps_backward() {
        // 2026年3月から5ヶ月前 → 2025年10月
        assert_eq!(CalendarModel::navigate(2026, 3, -5), (2025, 10));
    }

    #[test]
    fn navigate_zero_delta_returns_same() {
        assert_eq!(CalendarModel::navigate(2026, 6, 0), (2026, 6));
    }

    #[test]
    fn month_grid_weeks_are_sunday_first() {
        // 2026年1月1日は木曜日 → 第1週: [None, None, None, None, Some(1), Some(2), Some(3)]
        let grids = CalendarModel::months_for_mode(ViewMode::OneMonth, 2026, 1, false);
        let grid = &grids[0];
        assert_eq!(grid.weeks[0][0], None);  // 日
        assert_eq!(grid.weeks[0][1], None);  // 月
        assert_eq!(grid.weeks[0][2], None);  // 火
        assert_eq!(grid.weeks[0][3], None);  // 水
        assert_eq!(grid.weeks[0][4], Some(1)); // 木
        assert_eq!(grid.weeks[0][5], Some(2)); // 金
        assert_eq!(grid.weeks[0][6], Some(3)); // 土
    }
}
