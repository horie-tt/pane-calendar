// WindowManager: ウィンドウ透過・ピン留め管理

use eframe::egui;

/// ウィンドウ管理ユーティリティ
pub struct WindowManager;

impl WindowManager {
    /// グラスモーフィズム透過効果を適用する（Windows専用）。
    /// 失敗した場合は警告ログを出力して続行する（半透明背景にフォールバック）。
    #[cfg(target_os = "windows")]
    pub fn apply_acrylic(cc: &eframe::CreationContext<'_>) {
        use window_vibrancy::apply_acrylic;
        if let Err(e) = apply_acrylic(cc, Some((0, 0, 0, 80))) {
            log::warn!("Acrylic効果の適用に失敗しました（フォールバック: 半透明背景）: {:?}", e);
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn apply_acrylic(_cc: &eframe::CreationContext<'_>) {
        // Windows以外では何もしない
    }

    /// ピン留め状態をウィンドウに反映する
    pub fn apply_always_on_top(ctx: &egui::Context, always_on_top: bool) {
        let level = if always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
    }
}
