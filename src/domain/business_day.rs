// BusinessDayCalc: 営業日算出

use chrono::NaiveDate;

use super::holiday::HolidayService;

pub struct BusinessDayCalc;

impl BusinessDayCalc {
    /// 開始日〜終了日の総日数（両端含む）を返す
    /// start > end の場合は内部で正規化する
    pub fn total_days(start: NaiveDate, end: NaiveDate) -> u32 {
        let (s, e) = Self::normalize(start, end);
        ((e - s).num_days() + 1) as u32
    }

    /// 開始日〜終了日の営業日数（土日・祝日を除く、両端含む）を返す
    /// start > end の場合は内部で正規化する
    pub fn business_days(start: NaiveDate, end: NaiveDate) -> u32 {
        use chrono::Weekday;
        use chrono::Datelike;

        let (s, e) = Self::normalize(start, end);
        let mut count = 0u32;
        let mut current = s;
        while current <= e {
            let wd = current.weekday();
            let is_weekend = matches!(wd, Weekday::Sat | Weekday::Sun);
            if !is_weekend && !HolidayService::is_holiday(current) {
                count += 1;
            }
            current = current.succ_opt().expect("date overflow");
        }
        count
    }

    /// start > end の場合に入れ替えて正規化する
    fn normalize(start: NaiveDate, end: NaiveDate) -> (NaiveDate, NaiveDate) {
        if start <= end { (start, end) } else { (end, start) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // --- total_days テスト ---

    #[test]
    fn total_days_single_day() {
        // 開始日=終了日 → 1日
        assert_eq!(BusinessDayCalc::total_days(date(2026, 3, 16), date(2026, 3, 16)), 1);
    }

    #[test]
    fn total_days_one_week() {
        // 1週間 (3/16〜3/22) → 7日
        assert_eq!(BusinessDayCalc::total_days(date(2026, 3, 16), date(2026, 3, 22)), 7);
    }

    #[test]
    fn total_days_reversed_input() {
        // start > end でも正規化される
        assert_eq!(BusinessDayCalc::total_days(date(2026, 3, 22), date(2026, 3, 16)), 7);
    }

    #[test]
    fn total_days_cross_month() {
        // 月末〜翌月初: 3/30〜4/2 → 4日
        assert_eq!(BusinessDayCalc::total_days(date(2026, 3, 30), date(2026, 4, 2)), 4);
    }

    // --- business_days テスト ---

    #[test]
    fn business_days_single_weekday() {
        // 2026年3月16日（月）単日 → 1営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 3, 16), date(2026, 3, 16)), 1);
    }

    #[test]
    fn business_days_single_saturday() {
        // 2026年3月14日（土）単日 → 0営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 3, 14), date(2026, 3, 14)), 0);
    }

    #[test]
    fn business_days_single_sunday() {
        // 2026年3月15日（日）単日 → 0営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 3, 15), date(2026, 3, 15)), 0);
    }

    #[test]
    fn business_days_holiday_is_excluded() {
        // 2026年1月1日（元日）単日 → 0営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 1, 1), date(2026, 1, 1)), 0);
    }

    #[test]
    fn business_days_full_week() {
        // 2026年3月16日（月）〜3月20日（金）: 春分の日=3/20 → 4営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 3, 16), date(2026, 3, 20)), 4);
    }

    #[test]
    fn business_days_week_with_weekend() {
        // 2026年3月16日（月）〜3月22日（日）: 土日除く5日、3/20祝日除く → 4営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 3, 16), date(2026, 3, 22)), 4);
    }

    #[test]
    fn business_days_golden_week_2026() {
        // GW 2026年4/29〜5/6: 昭和の日,振替,憲法記念日,みどりの日,こどもの日,振替
        // 4/29(水,祝), 4/30(木), 5/1(金), 5/2(土), 5/3(日,祝), 5/4(月,祝), 5/5(火,祝), 5/6(水,振替)
        // 営業日は4/30, 5/1の2日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 4, 29), date(2026, 5, 6)), 2);
    }

    #[test]
    fn business_days_cross_month() {
        // 月末〜翌月初: 2026年3/30（月）〜4/3（金）
        // 3/30(月), 3/31(火), 4/1(水), 4/2(木), 4/3(金) → 5営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 3, 30), date(2026, 4, 3)), 5);
    }

    #[test]
    fn business_days_reversed_input() {
        // start > end でも正規化される
        assert_eq!(
            BusinessDayCalc::business_days(date(2026, 3, 22), date(2026, 3, 16)),
            BusinessDayCalc::business_days(date(2026, 3, 16), date(2026, 3, 22))
        );
    }

    #[test]
    fn holiday_on_weekend_not_double_counted() {
        // 祝日が土曜に重なる場合: 2026年5/2(土) ← 祝日ではないが
        // 2026年1/1(木)〜1/12(月,成人の日)
        // 1/1(祝), 1/2(金), 1/3(土), 1/4(日), 1/5(月), 1/6(火), 1/7(水), 1/8(木), 1/9(金),
        // 1/10(土), 1/11(日), 1/12(月,祝)
        // 営業日: 1/2, 1/5, 1/6, 1/7, 1/8, 1/9 → 6営業日
        assert_eq!(BusinessDayCalc::business_days(date(2026, 1, 1), date(2026, 1, 12)), 6);
    }
}
