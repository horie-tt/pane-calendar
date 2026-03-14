// HolidayService: koyomi_rsを用いた祝日判定

use chrono::NaiveDate;
use koyomi_rs::JapaneseHoliday;

pub struct HolidayService;

impl HolidayService {
    /// 指定日が祝日かどうかを返す
    pub fn is_holiday(date: NaiveDate) -> bool {
        JapaneseHoliday::holiday(&date).is_some()
    }

    /// 指定日の祝日名を返す。祝日でなければ None
    pub fn holiday_name(date: NaiveDate) -> Option<&'static str> {
        JapaneseHoliday::holiday(&date).map(|h| h.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 祝日判定テスト ---

    #[test]
    fn new_years_day_is_holiday() {
        // 元日
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(HolidayService::is_holiday(date));
        assert_eq!(HolidayService::holiday_name(date), Some("元日"));
    }

    #[test]
    fn coming_of_age_day_2026_is_holiday() {
        // 2026年成人の日: 1月第2月曜 = 1月12日
        let date = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap();
        assert!(HolidayService::is_holiday(date));
        assert_eq!(HolidayService::holiday_name(date), Some("成人の日"));
    }

    #[test]
    fn substitute_holiday_is_detected() {
        // 振替休日: 2026年3月23日（春分の日3/20が金曜なので振替なし、別の例）
        // 2026年11月3日（文化の日）が火曜なので振替なし
        // 2025年2月24日（天皇誕生日2/23が日曜 → 翌月曜2/24が振替休日）
        let date = NaiveDate::from_ymd_opt(2025, 2, 24).unwrap();
        assert!(HolidayService::is_holiday(date));
        assert_eq!(HolidayService::holiday_name(date), Some("振替休日"));
    }

    #[test]
    fn citizens_holiday_is_detected() {
        // 国民の休日: 2026年9月22日（敬老の日9/21と秋分の日9/23に挟まれた平日）
        let date = NaiveDate::from_ymd_opt(2026, 9, 22).unwrap();
        assert!(HolidayService::is_holiday(date));
        assert_eq!(HolidayService::holiday_name(date), Some("国民の休日"));
    }

    #[test]
    fn regular_weekday_is_not_holiday() {
        // 平日（2026年3月16日 月曜日）
        let date = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
        assert!(!HolidayService::is_holiday(date));
        assert_eq!(HolidayService::holiday_name(date), None);
    }

    #[test]
    fn saturday_is_not_holiday() {
        // 土曜日（2026年3月14日）は祝日ではない
        let date = NaiveDate::from_ymd_opt(2026, 3, 14).unwrap();
        assert!(!HolidayService::is_holiday(date));
    }

    #[test]
    fn sunday_is_not_holiday() {
        // 日曜日（2026年3月15日）は祝日ではない（振替休日でもない）
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        assert!(!HolidayService::is_holiday(date));
    }

    #[test]
    fn holiday_on_sunday_is_not_double_counted() {
        // 祝日が日曜日に重なる場合: その日自体は is_holiday=true
        // 2025年9月15日（敬老の日）は月曜なので別例
        // 2026年1月1日（元日）は木曜、2025年1月1日は水曜
        // 2023年1月1日（元日）は日曜 → 当日も holiday=true
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        assert!(HolidayService::is_holiday(date));
        // 翌日の振替休日も holiday=true
        let substitute = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap();
        assert!(HolidayService::is_holiday(substitute));
    }
}
