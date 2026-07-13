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
    #[cfg(target_os = "macos")]
    pub previous_frontmost_app: Arc<Mutex<Option<objc2::rc::Retained<objc2_app_kit::NSRunningApplication>>>>,
}

fn create_tray_menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    Menu::with_items(app, &[&show, &quit])
}

#[cfg(target_os = "macos")]
fn macos_tray_icon() -> tauri::image::Image<'static> {
    fn rounded_rect(
        x: f32,
        y: f32,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
        radius: f32,
    ) -> bool {
        let nearest_x = x.clamp(left + radius, right - radius);
        let nearest_y = y.clamp(top + radius, bottom - radius);
        let dx = x - nearest_x;
        let dy = y - nearest_y;
        dx * dx + dy * dy <= radius * radius
    }

    let size = 36u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let outer = rounded_rect(px, py, 5.0, 4.0, 31.0, 35.0, 5.0);
            let inner = rounded_rect(px, py, 9.0, 8.0, 27.0, 31.0, 2.0);
            let clip = rounded_rect(px, py, 11.0, 1.0, 25.0, 10.0, 3.0);
            let first_line = rounded_rect(px, py, 12.0, 14.0, 24.0, 17.0, 1.5);
            let second_line = rounded_rect(px, py, 12.0, 20.0, 24.0, 23.0, 1.5);
            let third_line = rounded_rect(px, py, 12.0, 26.0, 21.0, 29.0, 1.5);
            if (outer && !inner) || clip || first_line || second_line || third_line {
                rgba[((y * size + x) * 4 + 3) as usize] = 255;
            }
        }
    }

    tauri::image::Image::new_owned(rgba, size, size)
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let menu = create_tray_menu(app)?;

    #[cfg(target_os = "macos")]
    let icon = macos_tray_icon();
    #[cfg(not(target_os = "macos"))]
    let icon = app.default_window_icon().unwrap().clone();

    let mut builder = TrayIconBuilder::new()
        .icon(icon)
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
        });

    #[cfg(target_os = "macos")]
    {
        builder = builder.icon_as_template(true);
    }

    let tray = builder.build(app)?;

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
            if !hwnd.is_invalid() && GetWindowRect(hwnd, &mut rect).is_ok() {
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

#[cfg(target_os = "macos")]
fn remember_frontmost_app(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSRunningApplication, NSWorkspace};

    let Some(frontmost) = NSWorkspace::sharedWorkspace().frontmostApplication() else {
        return;
    };
    if frontmost == NSRunningApplication::currentApplication() {
        return;
    }

    let state = window.state::<AppState>();
    if let Ok(mut previous) = state.previous_frontmost_app.lock() {
        *previous = Some(frontmost);
    };
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn show_window_near_mouse(window: &tauri::WebviewWindow) {
    use enigo::{Enigo, Mouse, Settings};

    #[cfg(target_os = "macos")]
    remember_frontmost_app(window);

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
    #[cfg(debug_assertions)]
    {
        env_logger::init();
        log::info!("Starting SunSaltyBoard application");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            let db = Database::new(&app_handle).expect("Failed to initialize database");
            let settings = Arc::new(Mutex::new(db.load_settings()));

            let sync_manager = Arc::new(SyncManager::new());
            {
                let s = settings.lock().unwrap();
                if s.sync_enabled {
                    sync_manager.configure(s.sync_server.clone());
                }
            }

            // Setup auto-start based on settings
            {
                let settings_guard = settings.lock().unwrap();
                autostart::setup_autostart(settings_guard.auto_start);
            }

            let clipboard_manager = Arc::new(ClipboardManager::new());

            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                clipboard_manager,
                settings: settings.clone(),
                sync_manager,
                current_shortcut: Arc::new(Mutex::new(None)),
                app_handle: Arc::new(Mutex::new(Some(app_handle.clone()))),
                tray_icon: Arc::new(Mutex::new(None)),
                #[cfg(target_os = "macos")]
                previous_frontmost_app: Arc::new(Mutex::new(None)),
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
