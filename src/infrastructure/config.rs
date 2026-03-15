// ConfigManager: 設定ファイルの読み書き

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::calendar::ViewMode;

// ViewMode に Serialize/Deserialize を追加するためのラッパー
fn default_view_mode() -> ViewModeConfig {
    ViewModeConfig::ThreeMonths
}

/// TOML シリアライズ用の ViewMode ミラー
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewModeConfig {
    OneMonth,
    ThreeMonths,
    OneYear,
}

impl From<ViewModeConfig> for ViewMode {
    fn from(v: ViewModeConfig) -> Self {
        match v {
            ViewModeConfig::OneMonth => ViewMode::OneMonth,
            ViewModeConfig::ThreeMonths => ViewMode::ThreeMonths,
            ViewModeConfig::OneYear => ViewMode::OneYear,
        }
    }
}

impl From<ViewMode> for ViewModeConfig {
    fn from(v: ViewMode) -> Self {
        match v {
            ViewMode::OneMonth => ViewModeConfig::OneMonth,
            ViewMode::ThreeMonths => ViewModeConfig::ThreeMonths,
            ViewMode::OneYear => ViewModeConfig::OneYear,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_view_mode")]
    pub view_mode: ViewModeConfig,
    #[serde(default)]
    pub window_position: Option<[f32; 2]>,
    #[serde(default)]
    pub window_size: Option<[f32; 2]>,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub fiscal_year_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            view_mode: ViewModeConfig::ThreeMonths,
            window_position: None,
            window_size: None,
            always_on_top: false,
            fiscal_year_start: false,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Serialize(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {e}"),
            ConfigError::Serialize(s) => write!(f, "Serialize error: {s}"),
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    /// exe と同フォルダの config.toml パスを返す
    pub fn config_path() -> PathBuf {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let dir = exe.parent().unwrap_or_else(|| std::path::Path::new("."));
        dir.join("config.toml")
    }

    /// 設定を読み込む（ファイル不在・パース失敗時はデフォルト値）
    pub fn load() -> Config {
        Self::load_from(&Self::config_path())
    }

    /// 指定パスから設定を読み込む（テスト用）
    pub fn load_from(path: &std::path::Path) -> Config {
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Config::default(),
        };
        match toml::from_str::<Config>(&content) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("config.toml parse error: {e}, using defaults");
                Config::default()
            }
        }
    }

    /// 設定を保存する。失敗時は exe フォルダにエラーログを出力
    pub fn save(config: &Config) -> Result<(), ConfigError> {
        Self::save_to(config, &Self::config_path())
    }

    /// 指定パスに設定を保存する（テスト用）
    pub fn save_to(config: &Config, path: &std::path::Path) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::Serialize(e.to_string()))?;
        std::fs::write(path, content).map_err(ConfigError::Io)
    }

    /// 保存失敗時のエラーログを exe フォルダに出力する
    pub fn write_error_log(error: &ConfigError) {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let dir = exe.parent().unwrap_or_else(|| std::path::Path::new("."));
        let log_path = dir.join("pane-calendar-error.log");
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message = format!("[{timestamp}] Failed to save config: {error}\n");
        if let Err(e) = std::fs::write(&log_path, message) {
            log::error!("Failed to write error log: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- デフォルト値テスト ---

    #[test]
    fn default_config_has_three_months_view() {
        let c = Config::default();
        assert_eq!(c.view_mode, ViewModeConfig::ThreeMonths);
    }

    #[test]
    fn default_config_has_no_window_position() {
        let c = Config::default();
        assert_eq!(c.window_position, None);
    }

    #[test]
    fn default_config_has_no_window_size() {
        let c = Config::default();
        assert_eq!(c.window_size, None);
    }

    #[test]
    fn default_config_always_on_top_is_false() {
        let c = Config::default();
        assert!(!c.always_on_top);
    }

    #[test]
    fn default_config_fiscal_year_start_is_false() {
        let c = Config::default();
        assert!(!c.fiscal_year_start);
    }

    // --- ファイル不在・パースエラー時のフォールバックテスト ---

    #[test]
    fn load_from_nonexistent_file_returns_default() {
        let path = std::path::Path::new("/tmp/pane_calendar_test_nonexistent_12345.toml");
        let c = ConfigManager::load_from(path);
        assert_eq!(c, Config::default());
    }

    #[test]
    fn load_from_invalid_toml_returns_default() {
        let dir = std::env::temp_dir();
        let path = dir.join("pane_calendar_test_invalid.toml");
        std::fs::write(&path, "this is not valid toml :::").unwrap();
        let c = ConfigManager::load_from(&path);
        assert_eq!(c, Config::default());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_from_partial_toml_fills_missing_with_defaults() {
        // view_mode のみ指定、他フィールドは欠損 → デフォルト補完
        let dir = std::env::temp_dir();
        let path = dir.join("pane_calendar_test_partial.toml");
        std::fs::write(&path, "view_mode = \"one_year\"\n").unwrap();
        let c = ConfigManager::load_from(&path);
        assert_eq!(c.view_mode, ViewModeConfig::OneYear);
        assert_eq!(c.window_position, None); // デフォルト補完
        assert!(!c.always_on_top);           // デフォルト補完
        let _ = std::fs::remove_file(&path);
    }

    // --- ラウンドトリップテスト ---

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("pane_calendar_test_roundtrip.toml");

        let original = Config {
            view_mode: ViewModeConfig::OneYear,
            window_position: Some([100.0, 200.0]),
            window_size: Some([800.0, 600.0]),
            always_on_top: true,
            fiscal_year_start: true,
        };

        ConfigManager::save_to(&original, &path).expect("save failed");
        let loaded = ConfigManager::load_from(&path);

        assert_eq!(loaded, original);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_and_load_default_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("pane_calendar_test_default_roundtrip.toml");

        let original = Config::default();
        ConfigManager::save_to(&original, &path).expect("save failed");
        let loaded = ConfigManager::load_from(&path);

        assert_eq!(loaded, original);
        let _ = std::fs::remove_file(&path);
    }

    // --- ViewModeConfig 変換テスト ---

    #[test]
    fn view_mode_config_converts_to_domain() {
        assert_eq!(ViewMode::from(ViewModeConfig::OneMonth), ViewMode::OneMonth);
        assert_eq!(ViewMode::from(ViewModeConfig::ThreeMonths), ViewMode::ThreeMonths);
        assert_eq!(ViewMode::from(ViewModeConfig::OneYear), ViewMode::OneYear);
    }

    #[test]
    fn domain_view_mode_converts_to_config() {
        assert_eq!(ViewModeConfig::from(ViewMode::OneMonth), ViewModeConfig::OneMonth);
        assert_eq!(ViewModeConfig::from(ViewMode::ThreeMonths), ViewModeConfig::ThreeMonths);
        assert_eq!(ViewModeConfig::from(ViewMode::OneYear), ViewModeConfig::OneYear);
    }
}
