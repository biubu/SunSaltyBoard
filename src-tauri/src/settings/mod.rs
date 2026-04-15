use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use rusqlite::{params, Connection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub max_history_size: i32,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub global_shortcut: String,
    pub sync_enabled: bool,
    pub sync_server: Option<String>,
    pub theme: String,
    pub sensitive_filter: bool,
    pub encrypt_sensitive: bool,
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
            sensitive_filter: false,
            encrypt_sensitive: false,
        }
    }
}

impl Settings {
    pub fn load(app: &AppHandle) -> Self {
        let app_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let db_path = app_dir.join("clipstash.db");

        if let Ok(conn) = Connection::open(&db_path) {
            let get_setting = |key: &str, default: &str| -> String {
                conn.query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    params![key],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| default.to_string())
            };

            return Self {
                max_history_size: get_setting("max_history_size", "500").parse().unwrap_or(500),
                auto_start: get_setting("auto_start", "false") == "true",
                minimize_to_tray: get_setting("minimize_to_tray", "true") == "true",
                global_shortcut: get_setting("global_shortcut", "Ctrl+Shift+V"),
                sync_enabled: get_setting("sync_enabled", "false") == "true",
                sync_server: {
                    let val = get_setting("sync_server", "");
                    if val.is_empty() { None } else { Some(val) }
                },
                theme: get_setting("theme", "dark"),
                sensitive_filter: get_setting("sensitive_filter", "false") == "true",
                encrypt_sensitive: get_setting("encrypt_sensitive", "false") == "true",
            };
        }

        Self::default()
    }
}
