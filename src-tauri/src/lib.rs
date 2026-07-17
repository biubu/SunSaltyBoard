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
    WindowEvent,
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

/// Returns `(x, y, width, height)` in screen coordinates of the most likely
/// focused window owned by the given PID. Uses `CGWindowListCopyWindowInfo` to
/// enumerate on-screen windows and picks the largest normal-layer (layer == 0)
/// window, which corresponds to the main/key window for typical apps.
#[cfg(target_os = "macos")]
fn macos_focused_window_bounds(pid: i32) -> Option<(i32, i32, i32, i32)> {
    use objc2_core_foundation::{CFArray, CFDictionary, CFNumber, CFNumberType, CFRetained, CFString};
    use objc2_core_graphics::{CGWindowListCopyWindowInfo, CGWindowListOption};

    let option =
        CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements;
    let windows: Option<CFRetained<CFArray>> = CGWindowListCopyWindowInfo(option, 0);
    let windows = windows?;

    let dict_array: &CFArray = windows.as_opaque();
    let count = dict_array.count();
    if count <= 0 {
        return None;
    }

    // Build the dictionary keys. CFString construction is cheap for short
    // ASCII literals; we don't bother caching since this runs only on show.
    let key_owner_pid = CFString::from_static_str("kCGWindowOwnerPID");
    let key_layer = CFString::from_static_str("kCGWindowLayer");
    let key_bounds = CFString::from_static_str("kCGWindowBounds");
    let key_x = CFString::from_static_str("X");
    let key_y = CFString::from_static_str("Y");
    let key_w = CFString::from_static_str("Width");
    let key_h = CFString::from_static_str("Height");

    let mut best: Option<(i32, i32, i32, i32, i64)> = None;

    for i in 0..count {
        let raw = unsafe { dict_array.value_at_index(i) };
        if raw.is_null() {
            continue;
        }
        let dict: &CFDictionary = unsafe { &*(raw as *const CFDictionary) };

        // Owner PID
        let pid_raw =
            unsafe { dict.value((&*key_owner_pid) as *const CFString as *const std::ffi::c_void) };
        if pid_raw.is_null() {
            continue;
        }
        let pid_num: &CFNumber = unsafe { &*(pid_raw as *const CFNumber) };
        let mut owner_pid: i32 = 0;
        let ok = unsafe {
            pid_num.value(
                CFNumberType::SInt32Type,
                &mut owner_pid as *mut i32 as *mut std::ffi::c_void,
            )
        };
        if !ok || owner_pid != pid {
            continue;
        }

        // Layer (skip menu bar, tooltips, etc.)
        let layer_raw = unsafe {
            dict.value((&*key_layer) as *const CFString as *const std::ffi::c_void)
        };
        let mut layer: i32 = i32::MAX;
        if !layer_raw.is_null() {
            let layer_num: &CFNumber = unsafe { &*(layer_raw as *const CFNumber) };
            unsafe {
                layer_num.value(
                    CFNumberType::SInt32Type,
                    &mut layer as *mut i32 as *mut std::ffi::c_void,
                );
            }
        }
        if layer != 0 {
            continue;
        }

        // Bounds (CFDictionary with X/Y/Width/Height)
        let bounds_raw = unsafe {
            dict.value((&*key_bounds) as *const CFString as *const std::ffi::c_void)
        };
        if bounds_raw.is_null() {
            continue;
        }
        let bounds_dict: &CFDictionary = unsafe { &*(bounds_raw as *const CFDictionary) };

        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut w: f64 = 0.0;
        let mut h: f64 = 0.0;

        for (key, out) in [
            (&key_x, &mut x),
            (&key_y, &mut y),
            (&key_w, &mut w),
            (&key_h, &mut h),
        ] {
            let v_raw = unsafe {
                bounds_dict.value((&**key) as *const CFString as *const std::ffi::c_void)
            };
            if v_raw.is_null() {
                continue;
            }
            let v_num: &CFNumber = unsafe { &*(v_raw as *const CFNumber) };
            unsafe {
                v_num.value(
                    CFNumberType::Float64Type,
                    out as *mut f64 as *mut std::ffi::c_void,
                );
            }
        }

        if w <= 1.0 || h <= 1.0 {
            continue;
        }

        let area = (w * h) as i64;
        match best {
            Some((_, _, _, _, a)) if a >= area => {}
            _ => best = Some((x as i32, y as i32, w as i32, h as i32, area)),
        }
    }

    best.map(|(x, y, w, h, _)| (x, y, w, h))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn show_window_near_mouse(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    remember_frontmost_app(window);

    let window_size = window
        .outer_size()
        .unwrap_or(tauri::PhysicalSize { width: 300, height: 600 });
    let window_width = window_size.width as i32;
    let window_height = window_size.height as i32;

    let monitors = window.available_monitors().ok();
    let find_monitor = |mx: i32, my: i32| -> Option<(i32, i32, i32, i32)> {
        monitors.as_ref()?.iter().find_map(|m| {
            let pos = m.position();
            let size = m.size();
            if mx >= pos.x
                && mx <= pos.x + size.width as i32
                && my >= pos.y
                && my <= pos.y + size.height as i32
            {
                Some((pos.x, pos.y, size.width as i32, size.height as i32))
            } else {
                None
            }
        })
    };
    let fallback_screen = ||
        -> (i32, i32) {
            monitors
                .as_ref()
                .and_then(|m| m.first().map(|m| {
                    let s = m.size();
                    (s.width as i32, s.height as i32)
                }))
                .unwrap_or((1920, 1080))
        };

    // On macOS, prefer the focused app's window position so the popup
    // appears near the input box the user is typing into.
    #[cfg(target_os = "macos")]
    let focused_bounds = {
        let state = window.state::<AppState>();
        state
            .previous_frontmost_app
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|app| app.processIdentifier()))
            .and_then(macos_focused_window_bounds)
    };
    #[cfg(not(target_os = "macos"))]
    let focused_bounds: Option<(i32, i32, i32, i32)> = None;

    // Compute target (mx, my) and the containing screen's bounds.
    let (target_x, target_y, screen_x, screen_y, screen_w, screen_h) = if let Some((wx, wy, ww, wh)) =
        focused_bounds
    {
        // Place popup just above the bottom-center of the focused window,
        // which is where input boxes typically live (chat, terminal, IDE).
        let center_x = wx + ww / 2;
        let bottom_y = wy + wh;
        let screen = find_monitor(center_x, bottom_y).unwrap_or_else(|| {
            let (sw, sh) = fallback_screen();
            (0, 0, sw, sh)
        });
        (center_x, bottom_y, screen.0, screen.1, screen.2, screen.3)
    } else {
        use enigo::Mouse;
        let enigo = match enigo::Enigo::new(&enigo::Settings::default()) {
            Ok(e) => e,
            Err(_) => {
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
                return;
            }
        };
        match enigo.location() {
            Ok((mx, my)) => {
                let (mx, my) = (mx as i32, my as i32);
                let screen = find_monitor(mx, my).unwrap_or_else(|| {
                    let (sw, sh) = fallback_screen();
                    (0, 0, sw, sh)
                });
                (mx, my, screen.0, screen.1, screen.2, screen.3)
            }
            Err(_) => {
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
                return;
            }
        }
    };

    // Default position: centered horizontally on the target, slightly above it.
    let mut x = target_x - (window_width / 2);
    let mut y = target_y - window_height - 50;
    // If the focused window is small, hug its bottom edge instead.
    if let Some((_, _, ww, _)) = focused_bounds {
        if ww < window_width + 40 {
            y = target_y - window_height;
        }
    }

    let margin = 10;
    // Clamp to the containing screen.
    if x < screen_x + margin {
        x = screen_x + margin;
    }
    if x + window_width > screen_x + screen_w - margin {
        x = screen_x + screen_w - window_width - margin;
    }
    if y < screen_y + margin {
        y = screen_y + margin;
    }
    if y + window_height > screen_y + screen_h - margin {
        y = screen_y + screen_h - window_height - margin;
    }

    let _ = window.set_position(tauri::Position::Physical(
        tauri::PhysicalPosition { x, y }
    ));

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
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let minimize_to_tray = window
                    .state::<AppState>()
                    .settings
                    .lock()
                    .map(|settings| settings.minimize_to_tray)
                    .unwrap_or(false);
                if minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    window.app_handle().exit(0);
                }
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Hide the window immediately at startup. On Windows, even with
            // `visible: false` in tauri.conf.json, the WebView2 compositor can
            // briefly paint a default-black background before our first
            // hide() call lands, causing a noticeable black flash. Forcing
            // hide() as the very first thing in setup eliminates that race.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

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
            state.clipboard_manager.start(app_handle.clone(), state.settings.clone());
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
