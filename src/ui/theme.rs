// Theme: 色テーマ定数（仮値、レビュー時に調整）

use eframe::egui::Color32;

pub struct Theme;

impl Theme {
    // --- 背景色（アルファ値付き、グラスモーフィズム重ね合わせ考慮）---
    pub const BG_NORMAL: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 180);
    pub const BG_SATURDAY: Color32 = Color32::from_rgba_premultiplied(200, 220, 255, 200);
    pub const BG_SUNDAY: Color32 = Color32::from_rgba_premultiplied(255, 200, 200, 200);
    pub const BG_HOLIDAY: Color32 = Color32::from_rgba_premultiplied(255, 200, 200, 200);
    pub const BG_TODAY: Color32 = Color32::from_rgba_premultiplied(100, 180, 255, 220);
    pub const BG_SELECTED: Color32 = Color32::from_rgba_premultiplied(180, 230, 180, 220);

    // --- テキスト色 ---
    pub const TEXT_NORMAL: Color32 = Color32::from_rgb(40, 40, 40);
    pub const TEXT_SATURDAY: Color32 = Color32::from_rgb(50, 80, 180);
    pub const TEXT_SUNDAY: Color32 = Color32::from_rgb(200, 50, 50);
    pub const TEXT_HOLIDAY: Color32 = Color32::from_rgb(200, 50, 50);
    pub const TEXT_TODAY: Color32 = Color32::from_rgb(255, 255, 255);
    pub const TEXT_OTHER_MONTH: Color32 = Color32::from_rgb(180, 180, 180);

    // --- ウィンドウ・ヘッダー背景 ---
    pub const WINDOW_BG: Color32 = Color32::from_rgba_premultiplied(240, 240, 245, 160);
    pub const HEADER_BG: Color32 = Color32::from_rgba_premultiplied(220, 225, 235, 200);
}
