use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::database::ClipboardItem;
use crate::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContentType {
    Text,
    Image,
    File,
    Html,
    Rtf,
    Unknown,
}

impl ClipboardContentType {
    pub fn as_str(&self) -> &str {
        match self {
            ClipboardContentType::Text => "text",
            ClipboardContentType::Image => "image",
            ClipboardContentType::File => "file",
            ClipboardContentType::Html => "html",
            ClipboardContentType::Rtf => "rtf",
            ClipboardContentType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEvent {
    pub content_type: String,
    pub content: String,
    pub preview: String,
    pub metadata: Option<String>,
}

pub struct ClipboardManager {
    running: Arc<AtomicBool>,
    last_content: Arc<Mutex<String>>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            last_content: Arc::new(Mutex::new(String::new())),
        }
    }

    #[cfg(windows)]
    pub fn start(&self, app_handle: AppHandle) {
        use std::thread;
        use std::time::Duration;
        use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
        use windows::Win32::Graphics::Gdi::HGDIOBJ;
        use windows::Win32::System::DataExchange::AddClipboardFormatListener;
        use windows::Win32::System::Ole::OleGetClipboard;
        use windows::Win32::UI::WindowsAndMessaging::{
            DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, RegisterClassExW,
            SendMessageW, TranslateMessage, CreateWindowExW, CS_HREDRAW, CS_VREDRAW,
            HICON, HINSTANCE, IDI_APPLICATION, IDC_ARROW, CW_USEDEFAULT, WM_CLIPBOARDUPDATE,
            WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        };

        let running = self.running.clone();
        let last_content = self.last_content.clone();
        let app_handle_clone = app_handle.clone();

        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            log::info!("Clipboard monitor starting (event-based)");

            unsafe {
                // Initialize OLE for COM
                if OleGetClipboard().is_err() {
                    log::warn!("OLE initialization failed, continuing anyway");
                }

                let instance = HINSTANCE::default();
                let class_name = widestring::U16CString::from_str("SunSaltyBoardClipboard").unwrap();

                let wc = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance,
                    lpszClassName: class_name.as_ptr(),
                    hCursor: windows::Win32::UI::WindowsAndMessaging::LoadCursorW(
                        HINSTANCE::default(),
                        IDC_ARROW,
                    ).unwrap_or_default(),
                    hIcon: LoadIconW(instance, IDI_APPLICATION).unwrap_or_default(),
                    ..Default::default()
                };

                RegisterClassExW(&wc);

                let hwnd = CreateWindowExW(
                    Default::default(),
                    class_name.as_ptr(),
                    widestring::U16CString::from_str("SunSaltyBoardHiddenWindow").unwrap().as_ptr(),
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    0,
                    0,
                    HWND::default(),
                    None,
                    instance,
                    None,
                );

                if hwnd.0.is_null() {
                    log::error!("Failed to create hidden window");
                    return;
                }

                // Register as clipboard format listener
                if let Err(e) = AddClipboardFormatListener(hwnd) {
                    log::error!("Failed to add clipboard format listener: {:?}", e);
                    return;
                }

                // Store app_handle and last_content for the window proc
                CLIPBOARD_STATE.lock().unwrap().app_handle = Some(app_handle_clone);
                CLIPBOARD_STATE.lock().unwrap().last_content = last_content.clone();
                CLIPBOARD_STATE.lock().unwrap().running = running.clone();

                // Message loop
                let mut msg = Default::default();
                while unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0).into() } {
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                }

                // Cleanup
                let _ = DestroyWindow(hwnd);
                log::info!("Clipboard monitor stopped");
            }
        });
    }

    #[cfg(not(windows))]
    pub fn start(&self, app_handle: AppHandle) {
        // Fallback to polling for non-Windows platforms
        let running = self.running.clone();
        let last_content = self.last_content.clone();

        running.store(true, Ordering::SeqCst);

        std::thread::spawn(move || {
            log::info!("Clipboard monitor starting (polling mode)");
            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to initialize clipboard: {}", e);
                    return;
                }
            };

            while running.load(Ordering::SeqCst) {
                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_content.lock().unwrap();
                    if *last != text {
                        process_clipboard_change(
                            &text,
                            &mut last,
                            &app_handle,
                        );
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            log::info!("Clipboard monitor stopped");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global state for clipboard window proc
struct ClipboardWindowState {
    app_handle: Option<AppHandle>,
    last_content: Arc<Mutex<String>>,
    running: Arc<AtomicBool>,
}

static CLIPBOARD_STATE: Mutex<ClipboardWindowState> = Mutex::new(ClipboardWindowState {
    app_handle: None,
    last_content: std::sync::Mutex::new(String::new()),
    running: std::sync::atomic::AtomicBool::new(false),
});

#[cfg(windows)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::WM_CLIPBOARDUPDATE;

    if msg == WM_CLIPBOARDUPDATE {
        if let Some(app_handle) = CLIPBOARD_STATE.lock().unwrap().app_handle.clone() {
            let last_content = CLIPBOARD_STATE.lock().unwrap().last_content.clone();
            
            // Try to get text from clipboard
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_content.lock().unwrap();
                    if *last != text {
                        process_clipboard_change_impl(&text, &mut last, &app_handle);
                    }
                }
            }
        }
        return LRESULT(1);
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn process_clipboard_change(text: &str, last: &mut std::sync::MutexGuard<String>, app_handle: &AppHandle) {
    process_clipboard_change_impl(text, last, app_handle);
}

fn process_clipboard_change_impl(text: &str, last: &mut String, app_handle: &AppHandle) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        // Check sensitive content filter
        let settings = &state.settings;
        if settings.sensitive_filter {
            let sensitive_patterns = ["password", "passwd", "secret", "token", "api_key"];
            let lower_text = text.to_lowercase();
            if sensitive_patterns.iter().any(|p| lower_text.contains(p)) {
                log::info!("Skipping sensitive content");
                *last = text.clone();
                return;
            }
        }

        let preview = text.chars().take(200).collect::<String>();
        let item = ClipboardItem {
            id: Uuid::new_v4().to_string(),
            content_type: "text".to_string(),
            content: text.to_string(),
            preview,
            group_id: None,
            created_at: Utc::now().to_rfc3339(),
            is_favorite: false,
            metadata: None,
        };

        if let Ok(db) = state.db.lock() {
            if let Err(e) = db.insert_clipboard_item(&item) {
                log::error!("Failed to insert clipboard item: {}", e);
            }
        }

        if let Err(e) = app_handle.emit("clipboard-changed", &item) {
            log::error!("Failed to emit clipboard event: {}", e);
        }
    }

    *last = text.to_string();
}

// Windows API imports
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{LoadIconW, IDI_APPLICATION};
