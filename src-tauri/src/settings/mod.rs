use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub max_history_size: i32,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub global_shortcut: String,
    pub sync_enabled: bool,
    pub sync_server: Option<String>,
    pub theme: String,
    pub update_server_url: Option<String>,
    pub clipboard_monitor_enabled: bool,
    pub clipboard_poll_interval_ms: i32,
    pub clipboard_monitor_mode: String,
    pub font_size: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_history_size: 500,
            auto_start: false,
            minimize_to_tray: true,
            global_shortcut: "Ctrl+Shift+V".to_string(),
            sync_enabled: false,
            sync_server: None,
            theme: "dark".to_string(),
            update_server_url: None,
            clipboard_monitor_enabled: true,
            clipboard_poll_interval_ms: 2000,
            clipboard_monitor_mode: "adaptive".to_string(),
            font_size: 3,
        }
    }
}
