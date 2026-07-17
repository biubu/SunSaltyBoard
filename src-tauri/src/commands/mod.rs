use crate::database::{ClipboardItem, Group, Hotkey, Tag};
use crate::settings::Settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use tauri::{command, AppHandle, Manager, State};

/// Shared HTTP client with connection pooling.
pub(crate) fn http_client() -> &'static reqwest::Client {
    static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client")
    })
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

    #[cfg(target_os = "linux")]
    {
        use enigo::Keyboard;
        use std::thread;
        use std::time::Duration;

        thread::sleep(Duration::from_millis(100));

        let mut enigo = enigo::Enigo::new(&enigo::Settings::default())
            .map_err(|e| format!("enigo init: {}", e))?;
        enigo.key(enigo::Key::Control, enigo::Direction::Press)
            .map_err(|e| format!("failed key_down modifier: {}", e))?;
        enigo.key(enigo::Key::Unicode('v'), enigo::Direction::Click)
            .map_err(|e| format!("failed key_down v: {}", e))?;
        enigo.key(enigo::Key::Control, enigo::Direction::Release)
            .map_err(|e| format!("failed key_up modifier: {}", e))?;

        thread::sleep(Duration::from_millis(200));
    }

    #[cfg(target_os = "macos")]
    {
        use enigo::Keyboard;
        use std::thread;

        let mut enigo = enigo::Enigo::new(&enigo::Settings::default())
            .map_err(|e| format!("enigo init: {}", e))?;
        enigo.key(enigo::Key::Meta, enigo::Direction::Press)
            .map_err(|e| format!("failed key_down modifier: {}", e))?;
        thread::sleep(Duration::from_millis(30));
        let paste_result = enigo.raw(9, enigo::Direction::Click)
            .map_err(|e| format!("failed key_down v: {}", e));
        thread::sleep(Duration::from_millis(30));
        let release_result = enigo.key(enigo::Key::Meta, enigo::Direction::Release)
            .map_err(|e| format!("failed key_up modifier: {}", e));
        paste_result?;
        release_result?;
        thread::sleep(Duration::from_millis(350));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_paste_target(
    state: &AppState,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSRunningApplication>, String> {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGPreflightPostEventAccess() -> bool;
        fn CGRequestPostEventAccess() -> bool;
    }

    let has_access = unsafe {
        CGPreflightPostEventAccess() || CGRequestPostEventAccess()
    };
    if !has_access {
        return Err("需要辅助功能权限：请在“系统设置 → 隐私与安全性 → 辅助功能”中允许 SunSaltyBoard，然后重试".to_string());
    }

    let target = state
        .previous_frontmost_app
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("未找到要粘贴到的活动窗口，请切回目标应用后重新打开剪贴板")?;
    if target.isTerminated() {
        return Err("目标应用已退出，请重新打开剪贴板后再试".to_string());
    }
    Ok(target)
}

#[cfg(target_os = "macos")]
fn activate_macos_app(app: &objc2_app_kit::NSRunningApplication) -> Result<(), String> {
    use objc2_app_kit::NSApplicationActivationOptions;
    use std::thread;

    if app.isTerminated() {
        return Err("目标应用已退出，请重新打开剪贴板后再试".to_string());
    }

    if !app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows) {
        thread::sleep(Duration::from_millis(80));
        let _ = app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
    }

    for _ in 0..40 {
        if app.isActive() {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }

    thread::sleep(Duration::from_millis(120));
    Ok(())
}

fn paste_clipboard_item(
    state: &AppState,
    app: &AppHandle,
    item: ClipboardItem,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let target = macos_paste_target(state)?;

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let original = clipboard.get_text().ok();
    let payload = if item.content_type == "text" {
        item.content
    } else {
        item.preview
    };

    clipboard.set_text(&payload).map_err(|e| e.to_string())?;
    state.clipboard_manager.record_self_write("text", &payload);

    let result = (|| {
        if let Some(window) = app.get_webview_window("main") {
            window.hide().map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        activate_macos_app(&target)?;
        simulate_ctrl_v()
    })();

    if let Some(orig) = original {
        state.clipboard_manager.record_self_write("text", &orig);
        let _ = clipboard.set_text(&orig);
    }

    if result.is_err() {
        log::warn!("Paste simulation failed: {}. Window stays hidden; user can reopen via shortcut.", result.as_ref().unwrap_err());
    }

    result
}

#[command]
pub fn paste_item(state: State<'_, AppState>, app: AppHandle, item: ClipboardItem) -> Result<(), String> {
    log::info!("Pasting item: {} of type {}", item.id, item.content_type);
    paste_clipboard_item(state.inner(), &app, item)
}

#[command]
pub fn paste_to_active(state: State<'_, AppState>, app: AppHandle, item: ClipboardItem) -> Result<(), String> {
    log::info!("Paste to active: {} of type {}", item.id, item.content_type);
    paste_clipboard_item(state.inner(), &app, item)
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
    Ok(db.load_settings())
}

#[command]
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    // Snapshot old settings BEFORE writing
    let old = state.settings.lock().map_err(|e| e.to_string())?.clone();

    // 1. Persist to DB (batch in single transaction)
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let pairs = vec![
            ("max_history_size".to_string(), settings.max_history_size.to_string()),
            ("auto_start".to_string(), settings.auto_start.to_string()),
            ("minimize_to_tray".to_string(), settings.minimize_to_tray.to_string()),
            ("global_shortcut".to_string(), settings.global_shortcut.clone()),
            ("sync_enabled".to_string(), settings.sync_enabled.to_string()),
            ("sync_server".to_string(), settings.sync_server.clone().unwrap_or_default()),
            ("theme".to_string(), settings.theme.clone()),
            ("clipboard_poll_interval_ms".to_string(), settings.clipboard_poll_interval_ms.to_string()),
            ("clipboard_monitor_enabled".to_string(), settings.clipboard_monitor_enabled.to_string()),
            ("clipboard_monitor_mode".to_string(), settings.clipboard_monitor_mode.clone()),
            ("font_size".to_string(), settings.font_size.to_string()),
        ];
        let mut pairs = pairs;
        if let Some(ref url) = settings.update_server_url {
            pairs.push(("update_server_url".to_string(), url.clone()));
        }
        db.set_settings_batch(&pairs).map_err(|e| e.to_string())?;
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
pub fn get_app_version(app: tauri::AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

fn parse_github_repo(url: &str) -> Option<(String, String)> {
    let url = url.trim().trim_end_matches('/');
    let marker = "github.com/";
    let idx = url.find(marker)?;
    let after = &url[idx + marker.len()..];
    let after = after.split("/releases").next().unwrap_or(after);
    let parts: Vec<&str> = after.split('/').take(2).collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

fn pick_release_asset(assets: &[GitHubAsset]) -> Option<String> {
    const RANKED: &[(&str, u32)] = &[
        (".exe", 100),
        (".msi", 95),
        (".dmg", 90),
        (".deb", 80),
        (".appimage", 70),
    ];

    assets
        .iter()
        .filter_map(|a| {
            let lower = a.name.to_lowercase();
            RANKED
                .iter()
                .find(|(ext, _)| lower.ends_with(ext))
                .map(|(_, score)| (*score, a.browser_download_url.clone()))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, url)| url)
}

#[command]
pub async fn check_update(state: State<'_, AppState>) -> Result<UpdateInfo, String> {
    let url = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.update_server_url.clone()
    };

    let server_url = match url {
        Some(u) if !u.is_empty() => u,
        _ => {
            return Ok(UpdateInfo {
                latest_version: None,
                download_url: None,
                release_notes: None,
                error: Some("未配置更新服务器地址".to_string()),
            });
        }
    };

    let current_version = env!("CARGO_PKG_VERSION");

    if let Some((owner, repo)) = parse_github_repo(&server_url) {
        return Ok(check_github_release(&owner, &repo).await);
    }

    let endpoint = format!(
        "{}/update.json?current={}",
        server_url.trim_end_matches('/'),
        current_version
    );

    match http_client()
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

async fn check_github_release(owner: &str, repo: &str) -> UpdateInfo {
    let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);

    match http_client()
        .get(&api_url)
        .header("User-Agent", "SunSaltyBoard")
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return UpdateInfo {
                    latest_version: None,
                    download_url: None,
                    release_notes: None,
                    error: Some(format!(
                        "GitHub API 限流 ({}),稍后重试或换用自定义 update.json",
                        status.as_u16()
                    )),
                };
            }
            if !status.is_success() {
                return UpdateInfo {
                    latest_version: None,
                    download_url: None,
                    release_notes: None,
                    error: Some(format!("GitHub API 返回状态 {}", status.as_u16())),
                };
            }
            match resp.json::<GitHubRelease>().await {
                Ok(release) => {
                    let latest = release.tag_name.trim_start_matches('v').to_string();
                    let download_url = pick_release_asset(&release.assets).or(release.html_url);
                    UpdateInfo {
                        latest_version: Some(latest),
                        download_url,
                        release_notes: release.body,
                        error: None,
                    }
                }
                Err(e) => UpdateInfo {
                    latest_version: None,
                    download_url: None,
                    release_notes: None,
                    error: Some(format!("解析 GitHub 响应失败: {}", e)),
                },
            }
        }
        Err(e) => UpdateInfo {
            latest_version: None,
            download_url: None,
            release_notes: None,
            error: Some(format!("请求 GitHub 失败: {}", e)),
        },
    }
}
