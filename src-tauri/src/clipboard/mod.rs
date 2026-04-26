use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::database::ClipboardItem;
use crate::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
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
    last_image_hash: Arc<Mutex<u64>>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            last_content: Arc::new(Mutex::new(String::new())),
            last_image_hash: Arc::new(Mutex::new(0)),
        }
    }

    #[cfg(windows)]
    pub fn start(&self, app_handle: AppHandle) {
        use std::thread;
        use windows::Win32::Foundation::{HWND, HINSTANCE};
        use windows::Win32::System::DataExchange::AddClipboardFormatListener;
        use windows::Win32::System::Ole::OleGetClipboard;
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, DestroyWindow,
            DispatchMessageW, GetMessageW, IDC_ARROW, IDI_APPLICATION, LoadIconW,
            RegisterClassExW, TranslateMessage, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        };
        use windows_core::PCWSTR;

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
                    lpszClassName: PCWSTR(class_name.as_ptr()),
                    hCursor: LoadIconW(HINSTANCE::default(), IDI_APPLICATION)
                        .unwrap_or_default(),
                    hIcon: LoadIconW(instance, IDI_APPLICATION).unwrap_or_default(),
                    ..Default::default()
                };

                RegisterClassExW(&wc);

                let window_name = widestring::U16CString::from_str("SunSaltyBoardHiddenWindow").unwrap();
                let hwnd = CreateWindowExW(
                    Default::default(),
                    PCWSTR(class_name.as_ptr()),
                    PCWSTR(window_name.as_ptr()),
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    0,
                    0,
                    HWND::default(),
                    None,
                    instance,
                    None,
                )
                .unwrap_or_default();

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
                while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
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
                        process_clipboard_change(&text, &mut last, &app_handle);
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
    last_image_hash: Arc<Mutex<u64>>,
    running: Arc<AtomicBool>,
}

static CLIPBOARD_STATE: LazyLock<Mutex<ClipboardWindowState>> = LazyLock::new(|| {
    Mutex::new(ClipboardWindowState {
        app_handle: None,
        last_content: Arc::new(Mutex::new(String::new())),
        last_image_hash: Arc::new(Mutex::new(0)),
        running: Arc::new(AtomicBool::new(false)),
    })
});

#[cfg(windows)]
unsafe extern "system" fn window_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::UI::WindowsAndMessaging::WPARAM,
    lparam: windows::Win32::UI::WindowsAndMessaging::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, WM_CLIPBOARDUPDATE};

    if msg == WM_CLIPBOARDUPDATE {
        if let Some(app_handle) = CLIPBOARD_STATE.lock().unwrap().app_handle.clone() {
            let last_content = CLIPBOARD_STATE.lock().unwrap().last_content.clone();
            let last_image_hash = CLIPBOARD_STATE.lock().unwrap().last_image_hash.clone();

            // Try to get content from clipboard (text and image)
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                // Check for text content
                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_content.lock().unwrap();
                    if *last != text {
                        process_clipboard_change_impl(&text, &mut last, &app_handle, None);
                    }
                }

                // Check for image content
                if let Ok(image_data) = clipboard.get_image() {
                    let hash = calculate_image_hash(&image_data);
                    let mut last_hash = last_image_hash.lock().unwrap();
                    if *last_hash != hash {
                        let width = image_data.width;
                        let height = image_data.height;
                        let rgba_data = &image_data.bytes;

                        let png_data = encode_image_to_png(rgba_data, width, height);
                        let base64_image = BASE64.encode(&png_data);

                        process_clipboard_change_impl(
                            &base64_image,
                            &mut String::new(),
                            &app_handle,
                            Some(("image".to_string(), width, height)),
                        );
                        *last_hash = hash;
                    }
                }
            }
        }
        return windows::Win32::Foundation::LRESULT(1isize);
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

// Simple hash function for image change detection
fn calculate_image_hash(image: &arboard::ImageData) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image.width.hash(&mut hasher);
    image.height.hash(&mut hasher);
    let bytes = &image.bytes;
    let step = std::cmp::max(1, bytes.len() / 1000);
    for i in (0..bytes.len()).step_by(step) {
        bytes[i].hash(&mut hasher);
    }
    hasher.finish()
}

// Simple PNG encoding placeholder (in production, use png crate)
fn encode_image_to_png(rgba_data: &[u8], _width: usize, _height: usize) -> Vec<u8> {
    rgba_data.to_vec()
}

fn process_clipboard_change(text: &str, last: &mut std::sync::MutexGuard<String>, app_handle: &AppHandle) {
    process_clipboard_change_impl(text, last, app_handle, None);
}

fn process_clipboard_change_impl(
    text: &str,
    last: &mut String,
    app_handle: &AppHandle,
    image_info: Option<(String, usize, usize)>,
) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let settings = &state.settings;
        if settings.sensitive_filter && image_info.is_none() {
            let sensitive_patterns = ["password", "passwd", "secret", "token", "api_key"];
            let lower_text = text.to_lowercase();
            if sensitive_patterns.iter().any(|p| lower_text.contains(p)) {
                log::info!("Skipping sensitive content");
                *last = text.to_string();
                return;
            }
        }

        let (content_type, preview) = if let Some((ref img_type, width, height)) = image_info {
            (img_type.clone(), format!("Image: {}x{}", width, height))
        } else {
            ("text".to_string(), text.chars().take(200).collect::<String>())
        };

        let final_content = if settings.encrypt_sensitive && image_info.is_none() {
            encrypt_sensitive_content(text)
        } else {
            text.to_string()
        };

        let item = ClipboardItem {
            id: Uuid::new_v4().to_string(),
            content_type,
            content: final_content,
            preview,
            group_id: None,
            created_at: Utc::now().to_rfc3339(),
            is_favorite: false,
            metadata: image_info.as_ref().map(|(_, w, h)| format!("{{\"width\":{},\"height\":{}}}", w, h)),
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

// Simple encryption for sensitive content (XOR with key for demo - use AES in production)
fn encrypt_sensitive_content(content: &str) -> String {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::Aes256Gcm;
    use rand::RngCore;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(b"SunSaltyBoardSecretKey1234567890!");
    let cipher = Aes256Gcm::new(key);

    match cipher.encrypt(&nonce_bytes.into(), content.as_bytes()) {
        Ok(ciphertext) => {
            let mut result = nonce_bytes.to_vec();
            result.extend_from_slice(&ciphertext);
            BASE64.encode(&result)
        }
        Err(_) => content.to_string(),
    }
}

// Decrypt sensitive content
#[allow(dead_code)]
fn decrypt_sensitive_content(encrypted: &str) -> String {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::Aes256Gcm;

    if let Ok(data) = BASE64.decode(encrypted) {
        if data.len() > 12 {
            let nonce_bytes: &[u8; 12] = &data[..12].try_into().unwrap();
            let ciphertext = &data[12..];

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(b"SunSaltyBoardSecretKey1234567890!");
            let cipher = Aes256Gcm::new(key);

            if let Ok(plaintext) = cipher.decrypt(&(*nonce_bytes).into(), ciphertext) {
                return String::from_utf8_lossy(&plaintext).to_string();
            }
        }
    }
    encrypted.to_string()
}