// SelectionManager: 日付選択状態管理

use chrono::NaiveDate;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    Individual,
    Range {
        start: NaiveDate,
        end: NaiveDate,
    },
}

#[derive(Debug, Clone)]
pub struct SelectionManager {
    selected_dates: HashSet<NaiveDate>,
    mode: SelectionMode,
    drag_preview_end: Option<NaiveDate>,
}

impl SelectionManager {
    pub fn new() -> Self {
        Self {
            selected_dates: HashSet::new(),
            mode: SelectionMode::Individual,
            drag_preview_end: None,
        }
    }

    /// 日付セルのクリック選択をトグルする。
    /// Rangeモード中であれば範囲をクリアして Individual に切り替えてからトグルする。
    pub fn toggle_date(&mut self, date: NaiveDate) {
        // 範囲選択中のクリックは範囲をクリアして Individual に戻す
        if matches!(self.mode, SelectionMode::Range { .. }) {
            self.selected_dates.clear();
            self.drag_preview_end = None;
            self.mode = SelectionMode::Individual;
        }
        if self.selected_dates.contains(&date) {
            self.selected_dates.remove(&date);
        } else {
            self.selected_dates.insert(date);
        }
    }

    /// ドラッグ範囲選択を確定する。既存の全選択をクリアして Range モードへ移行する。
    /// start > end の場合は内部で正規化する。
    pub fn set_drag_range(&mut self, start: NaiveDate, end: NaiveDate) {
        let (s, e) = if start <= end { (start, end) } else { (end, start) };
        self.selected_dates.clear();
        self.drag_preview_end = None;
        // s..=e の全日付を selected_dates に追加
        let mut current = s;
        while current <= e {
            self.selected_dates.insert(current);
            current = current.succ_opt().expect("date overflow");
        }
        self.mode = SelectionMode::Range { start: s, end: e };
    }

    /// ドラッグ中の仮範囲終端を更新する（描画用プレビュー）
    pub fn update_drag_preview(&mut self, current: NaiveDate) {
        self.drag_preview_end = Some(current);
    }

    /// 全選択をクリアして Idle（Individual）状態に戻す
    pub fn clear(&mut self) {
        self.selected_dates.clear();
        self.drag_preview_end = None;
        self.mode = SelectionMode::Individual;
    }

    /// 指定日付が選択されているかを返す
    pub fn is_selected(&self, date: NaiveDate) -> bool {
        self.selected_dates.contains(&date)
    }

    /// 現在の選択モードへの参照を返す
    pub fn mode(&self) -> &SelectionMode {
        &self.mode
    }

    /// ドラッグ中のプレビュー終端を返す
    pub fn drag_preview_end(&self) -> Option<NaiveDate> {
        self.drag_preview_end
    }

    /// 現在の選択日付数を返す
    pub fn selected_count(&self) -> usize {
        self.selected_dates.len()
    }
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // --- クリック選択テスト ---

    #[test]
    fn click_adds_date_to_selection() {
        let mut mgr = SelectionManager::new();
        mgr.toggle_date(date(2026, 3, 16));
        assert!(mgr.is_selected(date(2026, 3, 16)));
        assert_eq!(mgr.selected_count(), 1);
    }

    #[test]
    fn click_again_removes_date_from_selection() {
        let mut mgr = SelectionManager::new();
        mgr.toggle_date(date(2026, 3, 16));
        mgr.toggle_date(date(2026, 3, 16));
        assert!(!mgr.is_selected(date(2026, 3, 16)));
        assert_eq!(mgr.selected_count(), 0);
    }

    #[test]
    fn multiple_clicks_select_non_contiguous_dates() {
        let mut mgr = SelectionManager::new();
        mgr.toggle_date(date(2026, 3, 2));
        mgr.toggle_date(date(2026, 3, 5));
        mgr.toggle_date(date(2026, 3, 10));
        assert!(mgr.is_selected(date(2026, 3, 2)));
        assert!(mgr.is_selected(date(2026, 3, 5)));
        assert!(mgr.is_selected(date(2026, 3, 10)));
        assert_eq!(mgr.selected_count(), 3);
    }

    #[test]
    fn initial_mode_is_individual() {
        let mgr = SelectionManager::new();
        assert_eq!(mgr.mode(), &SelectionMode::Individual);
    }

    // --- ドラッグ選択テスト ---

    #[test]
    fn drag_sets_range_mode() {
        let mut mgr = SelectionManager::new();
        mgr.set_drag_range(date(2026, 3, 16), date(2026, 3, 20));
        assert_eq!(
            mgr.mode(),
            &SelectionMode::Range {
                start: date(2026, 3, 16),
                end: date(2026, 3, 20),
            }
        );
    }

    #[test]
    fn drag_selects_all_dates_in_range() {
        let mut mgr = SelectionManager::new();
        mgr.set_drag_range(date(2026, 3, 16), date(2026, 3, 18));
        assert!(mgr.is_selected(date(2026, 3, 16)));
        assert!(mgr.is_selected(date(2026, 3, 17)));
        assert!(mgr.is_selected(date(2026, 3, 18)));
        assert_eq!(mgr.selected_count(), 3);
    }

    #[test]
    fn drag_clears_existing_click_selection() {
        let mut mgr = SelectionManager::new();
        // 先にクリック選択
        mgr.toggle_date(date(2026, 3, 1));
        mgr.toggle_date(date(2026, 3, 5));
        assert_eq!(mgr.selected_count(), 2);
        // ドラッグ開始で既存選択がクリアされる
        mgr.set_drag_range(date(2026, 3, 16), date(2026, 3, 18));
        assert!(!mgr.is_selected(date(2026, 3, 1)));
        assert!(!mgr.is_selected(date(2026, 3, 5)));
        assert_eq!(mgr.selected_count(), 3);
    }

    #[test]
    fn drag_reversed_input_is_normalized() {
        let mut mgr = SelectionManager::new();
        // end < start でも正規化される
        mgr.set_drag_range(date(2026, 3, 20), date(2026, 3, 16));
        assert_eq!(
            mgr.mode(),
            &SelectionMode::Range {
                start: date(2026, 3, 16),
                end: date(2026, 3, 20),
            }
        );
        assert!(mgr.is_selected(date(2026, 3, 16)));
        assert!(mgr.is_selected(date(2026, 3, 20)));
    }

    #[test]
    fn drag_cross_month_works() {
        let mut mgr = SelectionManager::new();
        mgr.set_drag_range(date(2026, 3, 30), date(2026, 4, 2));
        assert!(mgr.is_selected(date(2026, 3, 30)));
        assert!(mgr.is_selected(date(2026, 3, 31)));
        assert!(mgr.is_selected(date(2026, 4, 1)));
        assert!(mgr.is_selected(date(2026, 4, 2)));
        assert_eq!(mgr.selected_count(), 4);
    }

    // --- 状態遷移テスト ---

    #[test]
    fn click_after_range_clears_range_and_switches_to_individual() {
        let mut mgr = SelectionManager::new();
        mgr.set_drag_range(date(2026, 3, 16), date(2026, 3, 20));
        assert!(matches!(mgr.mode(), SelectionMode::Range { .. }));
        // 範囲確定後にクリック → 範囲クリア・Individual 切替・クリック先を選択
        mgr.toggle_date(date(2026, 3, 25));
        assert_eq!(mgr.mode(), &SelectionMode::Individual);
        assert!(!mgr.is_selected(date(2026, 3, 16)));
        assert!(mgr.is_selected(date(2026, 3, 25)));
        assert_eq!(mgr.selected_count(), 1);
    }

    #[test]
    fn drag_start_during_range_clears_existing_range() {
        let mut mgr = SelectionManager::new();
        mgr.set_drag_range(date(2026, 3, 1), date(2026, 3, 5));
        // 新しいドラッグで前の範囲はクリアされる
        mgr.set_drag_range(date(2026, 3, 16), date(2026, 3, 18));
        assert!(!mgr.is_selected(date(2026, 3, 1)));
        assert!(mgr.is_selected(date(2026, 3, 16)));
        assert_eq!(mgr.selected_count(), 3);
    }

    // --- クリア・プレビューテスト ---

    #[test]
    fn clear_removes_all_selection() {
        let mut mgr = SelectionManager::new();
        mgr.toggle_date(date(2026, 3, 1));
        mgr.toggle_date(date(2026, 3, 5));
        mgr.clear();
        assert_eq!(mgr.selected_count(), 0);
        assert_eq!(mgr.mode(), &SelectionMode::Individual);
    }

    #[test]
    fn clear_after_range_resets_to_individual() {
        let mut mgr = SelectionManager::new();
        mgr.set_drag_range(date(2026, 3, 16), date(2026, 3, 20));
        mgr.clear();
        assert_eq!(mgr.mode(), &SelectionMode::Individual);
        assert_eq!(mgr.selected_count(), 0);
    }

    #[test]
    fn drag_preview_update() {
        let mut mgr = SelectionManager::new();
        assert_eq!(mgr.drag_preview_end(), None);
        mgr.update_drag_preview(date(2026, 3, 18));
        assert_eq!(mgr.drag_preview_end(), Some(date(2026, 3, 18)));
        // clear でプレビューもリセット
        mgr.clear();
        assert_eq!(mgr.drag_preview_end(), None);
    }
}
