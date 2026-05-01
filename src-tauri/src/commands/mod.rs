use crate::database::{ClipboardItem, Group, Hotkey, Plugin, Tag};
use crate::settings::Settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager, State};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
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
    Ok(())
}

#[command]
pub fn paste_to_active(app: AppHandle, item: ClipboardItem) -> Result<(), String> {
    log::info!("Paste to active: {} of type {}", item.id, item.content_type);

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    if item.content_type == "text" {
        clipboard.set_text(&item.content).map_err(|e| e.to_string())?;
    } else {
        clipboard.set_text(&item.preview).map_err(|e| e.to_string())?;
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    #[cfg(windows)]
    {
        use std::thread;
        use std::time::Duration;

        thread::sleep(Duration::from_millis(100));

        unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, VK_LCONTROL, VK_V, KEYEVENTF_KEYUP, KEYBD_EVENT_FLAGS};

            keybd_event(VK_LCONTROL.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_V.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_V.0 as u8, 0, KEYEVENTF_KEYUP, 0);
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

#[command]
pub fn toggle_favorite(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_favorite(&id).map_err(|e| e.to_string())
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
    Ok(vec![])
}

#[command]
pub fn toggle_plugin(id: String, enabled: bool) -> Result<(), String> {
    log::info!("Toggle plugin {}: {}", id, enabled);
    Ok(())
}

// Settings
#[command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut settings = Settings::default();

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

    // Also update in-memory settings
    let mut state_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *state_settings = settings;

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
    let sync_manager = &state.sync_manager;
    let status = sync_manager.get_status();
    Ok(SyncStatus {
        connected: status.connected,
        last_sync: status.last_sync,
        status: status.status,
    })
}

#[command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let sync_manager = &state.sync_manager;
    let status = sync_manager.get_status();
    Ok(SyncStatus {
        connected: status.connected,
        last_sync: status.last_sync,
        status: status.status,
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
