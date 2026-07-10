use crate::database::{ClipboardItem, Group, Hotkey, Tag};
use crate::settings::Settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{command, AppHandle, Manager, State};

/// Shared HTTP client with connection pooling.
static HTTP_CLIENT: once_cell::sync::Lazy<reqwest::Client> = once_cell::sync::Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build reqwest client")
});

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

fn simulate_ctrl_v() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::thread;
        use std::time::Duration;
        unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::{
                SendInput, INPUT, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
                VK_CONTROL, VK_V,
            };

            fn vk_input(vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
                let mut input = INPUT::default();
                input.r#type = INPUT_KEYBOARD;
                input.Anonymous.ki.wVk = vk;
                input.Anonymous.ki.wScan = 0;
                input.Anonymous.ki.dwFlags = flags;
                input.Anonymous.ki.time = 0;
                input.Anonymous.ki.dwExtraInfo = 0;
                input
            }

            let down_ctrl = vk_input(VK_CONTROL, KEYBD_EVENT_FLAGS(0));
            let down_v = vk_input(VK_V, KEYBD_EVENT_FLAGS(0));
            let up_v = vk_input(VK_V, KEYEVENTF_KEYUP);
            let up_ctrl = vk_input(VK_CONTROL, KEYEVENTF_KEYUP);

            let inputs = [down_ctrl, down_v, up_v, up_ctrl];
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent as usize != inputs.len() {
                log::warn!("SendInput only delivered {} of {} events", sent, inputs.len());
            }
        }
        thread::sleep(Duration::from_millis(200));
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use enigo::Keyboard;
        use std::thread;
        use std::time::Duration;

        thread::sleep(Duration::from_millis(100));

        let mut enigo = enigo::Enigo::new(&enigo::Settings::default())
            .map_err(|e| format!("enigo init: {}", e))?;
        enigo.key(enigo::Key::Control, enigo::Direction::Press)
            .map_err(|e| format!("failed key_down control: {}", e))?;
        enigo.key(enigo::Key::Unicode('v'), enigo::Direction::Click)
            .map_err(|e| format!("failed key_down v: {}", e))?;
        enigo.key(enigo::Key::Control, enigo::Direction::Release)
            .map_err(|e| format!("failed key_up control: {}", e))?;

        thread::sleep(Duration::from_millis(200));
    }

    Ok(())
}

#[command]
pub fn paste_item(state: State<'_, AppState>, app: AppHandle, item: ClipboardItem) -> Result<(), String> {
    log::info!("Pasting item: {} of type {}", item.id, item.content_type);

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let original = clipboard.get_text().ok();

    let payload: String = if item.content_type == "text" {
        item.content.clone()
    } else {
        item.preview.clone()
    };

    clipboard.set_text(&payload).map_err(|e| e.to_string())?;
    state.clipboard_manager.record_self_write("text", &payload);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    simulate_ctrl_v()?;

    if let Some(orig) = original {
        let _ = clipboard.set_text(&orig);
    }

    Ok(())
}

#[command]
pub fn paste_to_active(state: State<'_, AppState>, app: AppHandle, item: ClipboardItem) -> Result<(), String> {
    log::info!("Paste to active: {} of type {}", item.id, item.content_type);

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let original = clipboard.get_text().ok();

    let payload: String = if item.content_type == "text" {
        item.content.clone()
    } else {
        item.preview.clone()
    };

    clipboard.set_text(&payload).map_err(|e| e.to_string())?;
    state.clipboard_manager.record_self_write("text", &payload);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    simulate_ctrl_v()?;

    if let Some(orig) = original {
        let _ = clipboard.set_text(&orig);
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

#[command]
pub fn register_hotkey(
    state: State<'_, AppState>,
    key_combination: String,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    let app_handle = {
        let guard = state.app_handle.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("App handle not available")?.clone()
    };

    let shortcut: Shortcut = key_combination.parse()
        .map_err(|e| format!("Invalid shortcut format: {}", e))?;

    {
        let mut current_shortcut = state.current_shortcut.lock().map_err(|e| e.to_string())?;
        if let Some(old) = current_shortcut.as_ref() {
            if old == &shortcut {
                return Ok(());
            }
            if let Err(e) = app_handle.global_shortcut().unregister(old.clone()) {
                log::warn!("Failed to unregister previous shortcut: {}", e);
            }
        }
        *current_shortcut = None;
    }

    let app_handle_clone = app_handle.clone();
    if let Err(e) = app_handle.global_shortcut().on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                crate::show_window_near_mouse(&window);
            }
        }
    }) {
        return Err(format!("Failed to register shortcut: {}", e));
    }

    let mut current_shortcut = state.current_shortcut.lock().map_err(|e| e.to_string())?;
    *current_shortcut = Some(shortcut);

    log::info!("Hotkey registered: {}", key_combination);
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
    if let Ok(Some(update_server)) = db.get_setting("update_server_url") {
        settings.update_server_url = Some(update_server);
    }

    Ok(settings)
}

#[command]
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    // Snapshot old settings BEFORE writing
    let old = state.settings.lock().map_err(|e| e.to_string())?.clone();

    // 1. Persist to DB
    {
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
        if let Some(ref url) = settings.update_server_url {
            db.set_setting("update_server_url", url)
                .map_err(|e| e.to_string())?;
        }
    }

    // 2. Apply side-effects

    // auto_start -> toggle OS autostart
    if old.auto_start != settings.auto_start {
        crate::autostart::setup_autostart(settings.auto_start);
    }

    // global_shortcut -> re-register OS hotkey
    if old.global_shortcut != settings.global_shortcut {
        if let Err(e) = register_hotkey(state.clone(), settings.global_shortcut.clone()) {
            log::error!("Failed to apply new shortcut: {}", e);
        }
    }

    // sync -> reconfigure sync manager
    if old.sync_enabled != settings.sync_enabled || old.sync_server != settings.sync_server {
        if settings.sync_enabled {
            state.sync_manager.configure(settings.sync_server.clone());
        } else {
            state.sync_manager.configure(None);
        }
    }

    // 3. Update in-memory copy
    let mut state_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *state_settings = settings;

    Ok(())
}

// Sync
#[command]
pub async fn trigger_sync(state: State<'_, AppState>) -> Result<crate::sync::SyncStatus, String> {
    state.sync_manager.trigger_sync(&state).await
}

#[command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<crate::sync::SyncStatus, String> {
    let sync_manager = &state.sync_manager;
    let status = sync_manager.get_status();
    Ok(crate::sync::SyncStatus {
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
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

        unsafe {
            let mut point = POINT::default();
            GetCursorPos(&mut point).map_err(|e| e.to_string())?;
            Ok(MousePosition { x: point.x, y: point.y })
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use enigo::Mouse;

        let enigo = enigo::Enigo::new(&enigo::Settings::default())
            .map_err(|e| format!("enigo init: {}", e))?;
        let (x, y) = enigo.location()
            .map_err(|e| format!("get location: {}", e))?;
        Ok(MousePosition { x: x as i32, y: y as i32 })
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        Ok(MousePosition { x: 0, y: 0 })
    }
}

// App version & updates
#[command]
pub fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub error: Option<String>,
}

#[command]
pub async fn check_update(state: State<'_, AppState>) -> Result<UpdateInfo, String> {
    let url = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.update_server_url.clone()
    };

    let server_url = match url {
        Some(u) if !u.is_empty() => u,
        _ => return Ok(UpdateInfo {
            latest_version: None,
            download_url: None,
            release_notes: None,
            error: Some("未配置更新服务器地址".to_string()),
        }),
    };

    let current_version = env!("CARGO_PKG_VERSION");

    let endpoint = format!("{}/update.json?current={}", server_url.trim_end_matches('/'), current_version);

    match HTTP_CLIENT
        .get(&endpoint)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                return Ok(UpdateInfo {
                    latest_version: None,
                    download_url: None,
                    release_notes: None,
                    error: Some(format!("服务器返回状态 {}", status.as_u16())),
                });
            }
            match resp.json::<UpdateInfo>().await {
                Ok(info) => Ok(info),
                Err(_) => Ok(UpdateInfo {
                    latest_version: None,
                    download_url: None,
                    release_notes: None,
                    error: Some("无法解析更新响应".to_string()),
                }),
            }
        }
        Err(e) => Ok(UpdateInfo {
            latest_version: None,
            download_url: None,
            release_notes: None,
            error: Some(format!("检查更新失败: {}", e)),
        }),
    }
}
