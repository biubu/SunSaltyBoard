use crate::database::{ClipboardItem, Group, Hotkey, Plugin, Tag};
use crate::AppState;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager, State};

// Decrypt sensitive content (imported from clipboard module logic)
fn decrypt_sensitive_content(encrypted: &str) -> String {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::Aes256Gcm;
    
    if let Ok(data) = BASE64.decode(encrypted) {
        if data.len() > 12 {
            let nonce_bytes: &[u8; 12] = match data[..12].try_into() {
                Ok(arr) => arr,
                Err(_) => return encrypted.to_string(),
            };
            let ciphertext = &data[12..];
            
            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(b"SunSaltyBoardSecretKey1234567890!");
            let cipher = Aes256Gcm::new(key);
            
            if let Ok(plaintext) = cipher.decrypt(&nonce_bytes.into(), ciphertext) {
                return String::from_utf8_lossy(&plaintext).to_string();
            }
        }
    }
    encrypted.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub items: Vec<ClipboardItem>,
    pub total: usize,
}

#[command]
pub fn get_clipboard_history(
    state: State<'_, AppState>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<ClipboardItem>, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_clipboard_history(limit, offset).map_err(|e| e.to_string())
}

#[command]
pub fn search_clipboard(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<ClipboardItem>, String> {
    let limit = limit.unwrap_or(50);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_clipboard(&query, limit).map_err(|e| e.to_string())
}

#[command]
pub fn paste_item(item: ClipboardItem) -> Result<(), String> {
    log::info!("Pasting item: {} of type {}", item.id, item.content_type);
    // Clipboard paste functionality - to be implemented with proper Windows API
    Ok(())
}

#[command]
pub fn paste_to_active(app: AppHandle, item: ClipboardItem) -> Result<(), String> {
    log::info!("Paste to active: {} of type {}", item.id, item.content_type);

    // 1. Write content to clipboard using arboard
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    // Handle different content types
    match item.content_type.as_str() {
        "text" => {
            // Check if content is encrypted and decrypt if needed
            let content = if item.content.starts_with("AAAAAAAAAAAA") || item.content.len() > 100 && item.content.chars().take(20).all(|c| c.is_alphanumeric() || c == '+' || c == '/') {
                // Try to decrypt (heuristic: base64 encoded encrypted content)
                decrypt_sensitive_content(&item.content)
            } else {
                item.content.clone()
            };
            clipboard.set_text(&content).map_err(|e| e.to_string())?;
        }
        "image" => {
            // Decode base64 image data
            if let Ok(image_data) = BASE64.decode(&item.content) {
                // Parse metadata for dimensions
                let (width, height) = if let Some(ref meta) = item.metadata {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(meta) {
                        let w = json["width"].as_u64().unwrap_or(100) as usize;
                        let h = json["height"].as_u64().unwrap_or(100) as usize;
                        (w, h)
                    } else {
                        (100, 100)
                    }
                } else {
                    (100, 100)
                };
                
                // Create image data (assuming RGBA format)
                let img = arboard::ImageData {
                    width,
                    height,
                    bytes: image_data.into(),
                };
                clipboard.set_image(&img).map_err(|e| e.to_string())?;
            } else {
                // Fallback to text preview
                clipboard.set_text(&item.preview).map_err(|e| e.to_string())?;
            }
        }
        _ => {
            // For other types, fall back to text representation
            clipboard.set_text(&item.preview).map_err(|e| e.to_string())?;
        }
    }

    // 2. Hide the window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // 3. Simulate Ctrl+V keypress using windows-rs
    #[cfg(windows)]
    {
        use std::thread;
        use std::time::Duration;

        // Small delay to ensure window is hidden before pasting
        thread::sleep(Duration::from_millis(100));

        unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, VK_CONTROL, VK_LCONTROL, VK_V, KEYEVENTF_KEYUP, KEYBD_EVENT_FLAGS};

            // Press Ctrl
            keybd_event(VK_LCONTROL.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);

            // Press V
            keybd_event(VK_V.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);

            // Release V
            keybd_event(VK_V.0 as u8, 0, KEYEVENTF_KEYUP, 0);

            // Release Ctrl
            keybd_event(VK_LCONTROL.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }

    Ok(())
}

#[command]
pub fn delete_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_item(&id).map_err(|e| e.to_string())
}

// Groups
#[command]
pub fn get_groups(state: State<'_, AppState>) -> Result<Vec<Group>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_groups().map_err(|e| e.to_string())
}

#[command]
pub fn create_group(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<Group, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_group(&name, &color).map_err(|e| e.to_string())
}

#[command]
pub fn delete_group(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_group(&id).map_err(|e| e.to_string())
}

#[command]
pub fn move_item_to_group(
    state: State<'_, AppState>,
    item_id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_item_group(&item_id, group_id.as_deref())
        .map_err(|e| e.to_string())
}

// Tags
#[command]
pub fn get_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tags().map_err(|e| e.to_string())
}

#[command]
pub fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_tag(&name, &color).map_err(|e| e.to_string())
}

#[command]
pub fn delete_tag(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_tag(&id).map_err(|e| e.to_string())
}

#[command]
pub fn add_tag_to_item(
    state: State<'_, AppState>,
    item_id: String,
    tag_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_tag_to_item(&item_id, &tag_id).map_err(|e| e.to_string())
}

#[command]
pub fn remove_tag_from_item(
    state: State<'_, AppState>,
    item_id: String,
    tag_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.remove_tag_from_item(&item_id, &tag_id)
        .map_err(|e| e.to_string())
}

// Hotkeys
#[command]
pub fn get_hotkeys(state: State<'_, AppState>) -> Result<Vec<Hotkey>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_hotkeys().map_err(|e| e.to_string())
}

#[command]
pub fn update_hotkey(
    state: State<'_, AppState>,
    action: String,
    key_combination: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_hotkey(&action, &key_combination)
        .map_err(|e| e.to_string())
}

// Plugins
#[command]
pub fn get_plugins() -> Result<Vec<Plugin>, String> {
    // TODO: Implement plugin loading
    Ok(vec![])
}

#[command]
pub fn toggle_plugin(id: String, enabled: bool) -> Result<(), String> {
    log::info!("Toggle plugin {}: {}", id, enabled);
    // TODO: Implement plugin toggle
    Ok(())
}

// Settings
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

#[command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    // Try to load from database, fall back to defaults
    let mut settings = Settings::default();
    let db = state.db.lock().map_err(|e| e.to_string())?;

    if let Ok(Some(max_history)) = db.get_setting("max_history_size") {
        if let Ok(v) = max_history.parse() {
            settings.max_history_size = v;
        }
    }
    if let Ok(Some(auto_start)) = db.get_setting("auto_start") {
        settings.auto_start = auto_start == "true";
    }
    if let Ok(Some(minimize_to_tray)) = db.get_setting("minimize_to_tray") {
        settings.minimize_to_tray = minimize_to_tray == "true";
    }
    if let Ok(Some(global_shortcut)) = db.get_setting("global_shortcut") {
        settings.global_shortcut = global_shortcut;
    }
    if let Ok(Some(sync_enabled)) = db.get_setting("sync_enabled") {
        settings.sync_enabled = sync_enabled == "true";
    }
    if let Ok(Some(sync_server)) = db.get_setting("sync_server") {
        settings.sync_server = Some(sync_server);
    }
    if let Ok(Some(theme)) = db.get_setting("theme") {
        settings.theme = theme;
    }
    if let Ok(Some(sensitive_filter)) = db.get_setting("sensitive_filter") {
        settings.sensitive_filter = sensitive_filter == "true";
    }
    if let Ok(Some(encrypt_sensitive)) = db.get_setting("encrypt_sensitive") {
        settings.encrypt_sensitive = encrypt_sensitive == "true";
    }

    Ok(settings)
}

#[command]
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting("max_history_size", &settings.max_history_size.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("auto_start", &settings.auto_start.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("minimize_to_tray", &settings.minimize_to_tray.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("global_shortcut", &settings.global_shortcut)
        .map_err(|e| e.to_string())?;
    db.set_setting("sync_enabled", &settings.sync_enabled.to_string())
        .map_err(|e| e.to_string())?;
    if let Some(ref server) = settings.sync_server {
        db.set_setting("sync_server", server)
            .map_err(|e| e.to_string())?;
    }
    db.set_setting("theme", &settings.theme).map_err(|e| e.to_string())?;
    db.set_setting("sensitive_filter", &settings.sensitive_filter.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("encrypt_sensitive", &settings.encrypt_sensitive.to_string())
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Sync
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatus {
    pub connected: bool,
    pub last_sync: Option<String>,
    pub status: String,
}

#[command]
pub fn trigger_sync(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    log::info!("Triggering sync");
    Ok(SyncStatus {
        connected: false,
        last_sync: None,
        status: "idle".to_string(),
    })
}

#[command]
pub fn get_sync_status() -> Result<SyncStatus, String> {
    Ok(SyncStatus {
        connected: false,
        last_sync: None,
        status: "idle".to_string(),
    })
}

// Window management
#[command]
pub fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
pub fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

#[command]
pub fn get_mouse_position() -> Result<MousePosition, String> {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        use windows::Win32::Foundation::POINT;

        unsafe {
            let mut point = POINT::default();
            GetCursorPos(&mut point).map_err(|e| e.to_string())?;
            Ok(MousePosition { x: point.x, y: point.y })
        }
    }
    #[cfg(not(windows))]
    {
        Ok(MousePosition { x: 0, y: 0 })
    }
}
