mod autostart;
mod clipboard;
mod commands;
mod database;
mod settings;
mod sync;

use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_global_shortcut::Shortcut;

pub use clipboard::ClipboardManager;
pub use database::Database;
pub use settings::Settings;
pub use sync::SyncManager;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub clipboard_manager: Arc<ClipboardManager>,
    pub settings: Arc<Mutex<Settings>>,
    pub sync_manager: Arc<SyncManager>,
    pub current_shortcut: Arc<Mutex<Option<Shortcut>>>,
    pub app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
    pub tray_icon: Arc<Mutex<Option<TrayIcon>>>,
}

fn create_tray_menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    Menu::with_items(app, &[&show, &quit])
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let menu = create_tray_menu(app)?;

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("SunSaltyBoard - 剪贴板管理器")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    show_window_near_mouse(&window);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    show_window_near_mouse(&window);
                }
            }
        })
        .build(app)?;

    // Store tray icon to prevent early drop (avoids duplicate icons on Windows)
    let state = app.state::<AppState>();
    *state.tray_icon.lock().unwrap() = Some(tray);

    Ok(())
}

#[cfg(windows)]
fn show_window_near_mouse(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
    use windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN;
    use windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN;

    unsafe {
        let screen_width = GetSystemMetrics(SM_CXSCREEN) as i32;
        let screen_height = GetSystemMetrics(SM_CYSCREEN) as i32;

        let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 300, height: 600 });
        let window_width = window_size.width as i32;
        let window_height = window_size.height as i32;

        // Try to position near the foreground (active) window first
        let (mut x, mut y) = {
            let hwnd = GetForegroundWindow();
            let mut rect = RECT::default();
            if hwnd.0 != 0 && GetWindowRect(hwnd, &mut rect).is_ok() {
                let cx = (rect.left + rect.right) / 2;
                let cy = (rect.top + rect.bottom) / 2;
                (cx - (window_width / 2), cy - 50)
            } else {
                // Fallback to cursor position
                let mut point = POINT::default();
                let _ = GetCursorPos(&mut point);
                (point.x - (window_width / 2), point.y - 50)
            }
        };

        let margin = 10;
        if x < margin { x = margin; }
        if x + window_width > screen_width - margin { x = screen_width - window_width - margin; }
        if y < margin { y = margin; }
        if y + window_height > screen_height - margin { y = screen_height - window_height - margin; }

        let _ = window.set_position(tauri::Position::Physical(
            tauri::PhysicalPosition { x, y }
        ));
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn show_window_near_mouse(window: &tauri::WebviewWindow) {
    use enigo::{Enigo, Mouse, Settings};

    let enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(_) => {
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
            return;
        }
    };

    if let Ok((mx, my)) = enigo.location() {
        let (mx, my) = (mx as i32, my as i32);
        let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 300, height: 600 });
        let window_width = window_size.width as i32;
        let window_height = window_size.height as i32;

        // Determine screen size from the monitor the cursor is on
        let screen_size = window.available_monitors()
            .ok()
            .and_then(|monitors| {
                monitors.into_iter().find(|m| {
                    let pos = m.position();
                    let size = m.size();
                    mx >= pos.x && mx <= pos.x + size.width as i32
                        && my >= pos.y && my <= pos.y + size.height as i32
                })
            })
            .map(|m| {
                let s = m.size();
                (s.width as i32, s.height as i32)
            })
            .unwrap_or((1920, 1080));

        let mut x = mx - (window_width / 2);
        let mut y = my - 50;

        let margin = 10;
        let (screen_width, screen_height) = screen_size;
        if x < margin { x = margin; }
        if x + window_width > screen_width - margin { x = screen_width - window_width - margin; }
        if y < margin { y = margin; }
        if y + window_height > screen_height - margin { y = screen_height - window_height - margin; }

        let _ = window.set_position(tauri::Position::Physical(
            tauri::PhysicalPosition { x, y }
        ));
    } else {
        let _ = window.center();
    }

    let _ = window.show();
    let _ = window.set_focus();
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn show_window_near_mouse(window: &tauri::WebviewWindow) {
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting SunSaltyBoard application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            let db = Database::new(&app_handle).expect("Failed to initialize database");
            let settings = Arc::new(Mutex::new(db.load_settings()));

            // Setup auto-start based on settings
            {
                let settings_guard = settings.lock().unwrap();
                autostart::setup_autostart(settings_guard.auto_start);
            }

            let clipboard_manager = Arc::new(ClipboardManager::new());

            let sync_manager = Arc::new(SyncManager::new());

            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                clipboard_manager,
                settings: settings.clone(),
                sync_manager,
                current_shortcut: Arc::new(Mutex::new(None)),
                app_handle: Arc::new(Mutex::new(Some(app_handle.clone()))),
                tray_icon: Arc::new(Mutex::new(None)),
            });

            setup_tray(app)?;

            let state = app.state::<AppState>();
            state.clipboard_manager.start(app_handle.clone());
            state.sync_manager.start(app_handle.clone());

            // Register global shortcut from settings
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

            let shortcut_str = {
                let settings_guard = settings.lock().unwrap();
                settings_guard.global_shortcut.clone()
            };
            let shortcut: Shortcut = shortcut_str.parse().unwrap_or_else(|_| "Ctrl+Shift+V".parse().unwrap());
            let app_handle_clone = app_handle.clone();

            if let Err(e) = app.global_shortcut().on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    if let Some(window) = app_handle_clone.get_webview_window("main") {
                        show_window_near_mouse(&window);
                    }
                }
            }) {
                log::error!("Failed to register global shortcut: {}", e);
            } else {
                let state = app.state::<AppState>();
                let mut current_shortcut = state.current_shortcut.lock().unwrap();
                *current_shortcut = Some(shortcut);
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clipboard_history,
            commands::search_clipboard,
            commands::paste_item,
            commands::paste_to_active,
            commands::delete_item,
            commands::toggle_favorite,
            commands::get_groups,
            commands::create_group,
            commands::delete_group,
            commands::move_item_to_group,
            commands::get_tags,
            commands::create_tag,
            commands::delete_tag,
            commands::add_tag_to_item,
            commands::remove_tag_from_item,
            commands::get_hotkeys,
            commands::update_hotkey,
            commands::register_hotkey,
            commands::get_settings,
            commands::update_settings,
            commands::trigger_sync,
            commands::get_sync_status,
            commands::show_window,
            commands::hide_window,
            commands::get_mouse_position,
            commands::get_app_version,
            commands::check_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
