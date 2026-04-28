mod clipboard;
mod commands;
mod database;
mod plugins;
mod search;
mod settings;
mod sync;

use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub use clipboard::ClipboardManager;
pub use database::Database;
pub use settings::Settings;
pub use sync::SyncManager;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub clipboard_manager: Arc<ClipboardManager>,
    pub settings: Arc<Mutex<Settings>>,
    pub sync_manager: Arc<SyncManager>,
}

fn create_tray_menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    Menu::with_items(app, &[&show, &quit])
}

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let menu = create_tray_menu(app)?;

    let _tray = TrayIconBuilder::new()
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

    Ok(())
}

#[cfg(windows)]
fn show_window_near_mouse(window: &tauri::WebviewWindow) {
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
    use windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN;
    use windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN;
    use windows::Win32::Foundation::POINT;

    unsafe {
        let mut point = POINT::default();
        GetCursorPos(&mut point);

        let screen_width = GetSystemMetrics(SM_CXSCREEN) as i32;
        let screen_height = GetSystemMetrics(SM_CYSCREEN) as i32;

        let window_width = 480;
        let window_height = 420;

        let mut x = point.x - (window_width / 2);
        let mut y = point.y - 50;

        let margin = 10;
        if x < margin {
            x = margin;
        }
        if x + window_width > screen_width - margin {
            x = screen_width - window_width - margin;
        }
        if y < margin {
            y = margin;
        }
        if y + window_height > screen_height - margin {
            y = screen_height - window_height - margin;
        }

        let _ = window.set_position(tauri::Position::Physical(
            tauri::PhysicalPosition { x, y }
        ));
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(not(windows))]
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

            let settings = Arc::new(Mutex::new(Settings::load(&app_handle)));

            let clipboard_manager = Arc::new(ClipboardManager::new());

            let sync_manager = Arc::new(SyncManager::new());

            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                clipboard_manager,
                settings,
                sync_manager,
            });

            setup_tray(app)?;

            let state = app.state::<AppState>();
            state.clipboard_manager.start(app_handle.clone());
            state.sync_manager.start(app_handle.clone());

            // Register global shortcut
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

            let shortcut: Shortcut = "Ctrl+Shift+V".parse().unwrap();
            let app_handle_clone = app_handle.clone();

            if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    if let Some(window) = app_handle_clone.get_webview_window("main") {
                        show_window_near_mouse(&window);
                    }
                }
            }) {
                log::error!("Failed to register global shortcut: {}", e);
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
            commands::get_plugins,
            commands::toggle_plugin,
            commands::get_settings,
            commands::update_settings,
            commands::trigger_sync,
            commands::get_sync_status,
            commands::show_window,
            commands::hide_window,
            commands::get_mouse_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
