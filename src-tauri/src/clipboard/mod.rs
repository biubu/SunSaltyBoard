use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};

use crate::database::ClipboardItem;
use crate::{AppState, Settings};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
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
    last_text: Arc<Mutex<String>>,
    last_image: Arc<Mutex<String>>,
    last_html: Arc<Mutex<String>>,
    /// Recent values we wrote to the OS clipboard ourselves, used to suppress
    /// the monitor from re-capturing our own paste actions.
    self_writes: Arc<Mutex<Vec<SelfWrite>>>,
    /// Cached settings to avoid locking on every poll iteration.
    cached_poll_interval_ms: Arc<AtomicU64>,
    cached_monitor_enabled: Arc<AtomicBool>,
}

#[derive(Clone)]
struct SelfWrite {
    content_type: String,
    content_hash: String,
    expires_at_ms: u128,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            last_text: Arc::new(Mutex::new(String::new())),
            last_image: Arc::new(Mutex::new(String::new())),
            last_html: Arc::new(Mutex::new(String::new())),
            self_writes: Arc::new(Mutex::new(Vec::new())),
            cached_poll_interval_ms: Arc::new(AtomicU64::new(2000)),
            cached_monitor_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Record a clipboard write performed by this app so the monitor thread
    /// can ignore the resulting echo. Entries auto-expire after 5 seconds.
    pub fn record_self_write(&self, content_type: &str, content: &str) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let entry = SelfWrite {
            content_type: content_type.to_string(),
            content_hash: content_fingerprint(content_type, content),
            expires_at_ms: now_ms + 5_000,
        };
        if let Ok(mut writes) = self.self_writes.lock() {
            writes.retain(|w| w.expires_at_ms > now_ms);
            writes.push(entry);
        }
    }

    pub fn start(&self, app_handle: AppHandle, settings: Arc<Mutex<Settings>>) {
        let running = self.running.clone();
        let last_text = self.last_text.clone();
        let last_image = self.last_image.clone();
        let last_html = self.last_html.clone();
        let self_writes = self.self_writes.clone();
        let cached_poll_interval_ms = self.cached_poll_interval_ms.clone();
        let cached_monitor_enabled = self.cached_monitor_enabled.clone();

        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            log::info!("Clipboard monitor starting");
            let mut clipboard = loop {
                match arboard::Clipboard::new() {
                    Ok(c) => break c,
                    Err(e) => {
                        log::error!("Failed to initialize clipboard: {}", e);
                        if !running.load(Ordering::SeqCst) {
                            return;
                        }
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            };

            let mut remote_detected = is_remote_session();
            let mut access_ok = true;
            let mut recheck_counter = 0u64;
            let mut settings_refresh_counter = 0u64;

            if remote_detected {
                log::info!("Remote desktop session detected at startup; clipboard reads suspended");
            }

            while running.load(Ordering::SeqCst) {
                // Refresh cached settings every 30 iterations (~60 seconds at 2s poll)
                settings_refresh_counter += 1;
                if settings_refresh_counter % 30 == 0 {
                    if let Ok(s) = settings.lock() {
                        cached_poll_interval_ms.store(s.clipboard_poll_interval_ms.max(200) as u64, Ordering::Relaxed);
                        cached_monitor_enabled.store(s.clipboard_monitor_enabled, Ordering::Relaxed);
                    }
                }

                let poll_interval_ms = cached_poll_interval_ms.load(Ordering::Relaxed);
                let monitor_enabled = cached_monitor_enabled.load(Ordering::Relaxed);

                if !monitor_enabled {
                    thread::sleep(Duration::from_millis(poll_interval_ms));
                    continue;
                }

                let mode = settings
                    .lock()
                    .ok()
                    .map(|s| s.clipboard_monitor_mode.clone())
                    .unwrap_or_default();

                // Re-evaluate remote status periodically (every 60 iterations)
                // to detect when a remote desktop session ends while still
                // respecting env-var-based detection every iteration.
                recheck_counter += 1;
                let is_remote = if mode == "adaptive" {
                    if recheck_counter % 60 == 0 {
                        let fresh = is_remote_session();
                        if fresh != remote_detected {
                            remote_detected = fresh;
                            access_ok = !fresh;
                            if fresh {
                                log::info!("Remote desktop session detected; clipboard reads suspended");
                            } else {
                                log::info!("Remote desktop session ended; clipboard reads resumed");
                            }
                        }
                    }
                    remote_detected
                } else {
                    false
                };

                if is_remote || !access_ok {
                    let sleep_ms = poll_interval_ms.max(5000);
                    thread::sleep(Duration::from_millis(sleep_ms));
                    continue;
                }

                let mut got_content = false;

                match clipboard.get().html() {
                    Ok(html) => {
                        let mut last = last_html.lock().unwrap();
                        if *last != html {
                            if !is_self_write(&self_writes, "html", &html) {
                                let stripped = strip_html_tags(&html);
                                let preview: String = stripped.chars().take(200).collect();

                                let event = ClipboardEvent {
                                    content_type: ClipboardContentType::Html.as_str().to_string(),
                                    content: html.clone(),
                                    preview: preview.clone(),
                                    metadata: None,
                                };

                                emit_and_store(&app_handle, &event);

                                if !preview.is_empty() {
                                    if let Ok(mut writes) = self_writes.lock() {
                                        let now_ms = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .map(|d| d.as_millis())
                                            .unwrap_or(0);
                                        writes.retain(|w| w.expires_at_ms > now_ms);
                                        writes.push(SelfWrite {
                                            content_type: "text".to_string(),
                                            content_hash: content_fingerprint("text", &stripped),
                                            expires_at_ms: now_ms + 5_000,
                                        });
                                    }
                                }
                            }
                            *last = html;
                            got_content = true;
                        }
                    }
                    Err(e) => {
                        log::debug!("get.html failed: {}", e);
                    }
                }

                match clipboard.get_image() {
                    Ok(image) => {
                        let rgba_b64 = STANDARD.encode(&image.bytes);
                        let mut hasher = Sha256::new();
                        hasher.update(&image.bytes);
                        let image_hash = format!("{:x}", hasher.finalize());
                        let image_key = format!("{}x{}:{}", image.width, image.height, image_hash);
                        let mut last = last_image.lock().unwrap();
                        if *last != image_key {
                            if !is_self_write(&self_writes, "image", &rgba_b64) {
                                let metadata = serde_json::json!({
                                    "width": image.width,
                                    "height": image.height,
                                    "format": "rgba",
                                    "sha256": image_hash,
                                }).to_string();

                                let event = ClipboardEvent {
                                    content_type: ClipboardContentType::Image.as_str().to_string(),
                                    content: rgba_b64.clone(),
                                    preview: format!("Image {}x{}", image.width, image.height),
                                    metadata: Some(metadata),
                                };

                                emit_and_store(&app_handle, &event);
                            }
                            *last = image_key;
                            got_content = true;
                        }
                    }
                    Err(e) => {
                        log::debug!("get_image failed: {}", e);
                    }
                }

                match clipboard.get_text() {
                    Ok(text) => {
                        access_ok = true;
                        let mut last = last_text.lock().unwrap();
                        if *last != text {
                            if !is_self_write(&self_writes, "text", &text) {
                                let preview = text.chars().take(200).collect::<String>();
                                let event = ClipboardEvent {
                                    content_type: ClipboardContentType::Text.as_str().to_string(),
                                    content: text.clone(),
                                    preview,
                                    metadata: None,
                                };

                                emit_and_store(&app_handle, &event);
                            }
                            *last = text;
                            got_content = true;
                        }
                    }
                    Err(e) => {
                        log::debug!("get_text failed: {}", e);
                        if mode == "adaptive" {
                            access_ok = false;
                            remote_detected = true;
                            log::warn!("Clipboard text read denied; treating as remote session");
                            continue;
                        }
                    }
                }

                if !got_content {
                    thread::sleep(Duration::from_millis(poll_interval_ms));
                } else {
                    thread::sleep(Duration::from_millis(poll_interval_ms.min(500)));
                }
            }

            log::info!("Clipboard monitor stopped");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn is_self_write(writes: &Arc<Mutex<Vec<SelfWrite>>>, content_type: &str, content: &str) -> bool {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let content_hash = content_fingerprint(content_type, content);
    let Ok(mut writes) = writes.lock() else { return false };
    writes.retain(|w| w.expires_at_ms > now_ms);
    if let Some(pos) = writes
        .iter()
        .position(|w| w.content_type == content_type && w.content_hash == content_hash)
    {
        writes.remove(pos);
        true
    } else {
        false
    }
}

fn content_fingerprint(content_type: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content_type.as_bytes());
    hasher.update([0]);
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn decode_html_entities(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '&' {
            result.push(c);
            continue;
        }
        let mut entity = String::new();
        let mut end_with_semicolon = false;
        while let Some(&next) = chars.peek() {
            chars.next();
            if next == ';' {
                end_with_semicolon = true;
                break;
            }
            entity.push(next);
            if entity.len() > 10 {
                break;
            }
        }
        match entity.as_str() {
            "nbsp" => result.push(' '),
            "amp" => result.push('&'),
            "lt" => result.push('<'),
            "gt" => result.push('>'),
            "quot" => result.push('"'),
            "apos" => result.push('\''),
            "copy" => result.push('\u{00A9}'),
            "reg" => result.push('\u{00AE}'),
            "trade" => result.push('\u{2122}'),
            "mdash" => result.push('\u{2014}'),
            "ndash" => result.push('\u{2013}'),
            "hellip" => result.push('\u{2026}'),
            "laquo" => result.push('\u{00AB}'),
            "raquo" => result.push('\u{00BB}'),
            "lsquo" => result.push('\u{2018}'),
            "rsquo" => result.push('\u{2019}'),
            "ldquo" => result.push('\u{201C}'),
            "rdquo" => result.push('\u{201D}'),
            _ => {
                if let Some(digits) = entity.strip_prefix('#') {
                    let code = if let Some(hex) = digits.strip_prefix(['x', 'X']) {
                        u32::from_str_radix(hex, 16).ok()
                    } else {
                        digits.parse::<u32>().ok()
                    };
                    if let Some(n) = code {
                        if let Some(ch) = char::from_u32(n) {
                            result.push(ch);
                            continue;
                        }
                    }
                }
                result.push('&');
                result.push_str(&entity);
                if end_with_semicolon {
                    result.push(';');
                }
            }
        }
    }
    result
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;
    let mut text_buf = String::new();

    for c in html.chars() {
        match c {
            '<' => {
                if !inside_tag && !text_buf.is_empty() {
                    result.push_str(&decode_html_entities(&text_buf));
                    text_buf.clear();
                }
                inside_tag = true;
            }
            '>' if inside_tag => {
                inside_tag = false;
            }
            _ if !inside_tag => {
                text_buf.push(c);
            }
            _ => {}
        }
    }

    if !text_buf.is_empty() {
        result.push_str(&decode_html_entities(&text_buf));
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn emit_and_store(app_handle: &AppHandle, event: &ClipboardEvent) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let item_to_emit = {
            let Ok(db) = state.db.lock() else { return };
            let dedup_key = event.content.clone();

            if let Ok(Some(existing_id)) = db.find_by_content(&dedup_key) {
                if let Err(e) = db.update_item_timestamp(&existing_id) {
                    log::error!("Failed to update item timestamp: {}", e);
                }
                None
            } else {
                let item = ClipboardItem {
                    id: Uuid::new_v4().to_string(),
                    content_type: event.content_type.clone(),
                    content: event.content.clone(),
                    preview: event.preview.clone(),
                    group_id: None,
                    created_at: Utc::now().to_rfc3339(),
                    is_favorite: false,
                    metadata: event.metadata.clone(),
                };

                if let Err(e) = db.insert_clipboard_item(&item) {
                    log::error!("Failed to insert clipboard item: {}", e);
                }
                if let Ok(settings) = state.settings.lock() {
                    let _ = db.prune_history(settings.max_history_size);
                }
                Some(item)
            }
        };

        if let Some(item) = item_to_emit {
            if let Err(e) = app_handle.emit("clipboard-changed", &item) {
                log::error!("Failed to emit clipboard event: {}", e);
            }
        }
    }
}

fn is_remote_session() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Environment variable checks
        if std::env::var("RDP_SESSION").is_ok()
            || std::env::var("SSH_CONNECTION").is_ok()
            || std::env::var("SSH_CLIENT").is_ok()
            || std::env::var("VNC_DESKTOP").is_ok()
            || std::env::var("VNCSESSIONID").is_ok()
            || std::env::var("REMOTE_HOST").is_ok()
            || std::env::var("XRDP_SESSION").is_ok()
            || std::env::var("PAM_TYPE")
                .map(|v| v == "remote")
                .unwrap_or(false)
        {
            return true;
        }

        // DISPLAY check: xrdp uses :1, :10, :11 etc. while local is usually :0
        if let Ok(display) = std::env::var("DISPLAY") {
            let d = display.trim();
            if !d.is_empty()
                && d != ":0"
                && d != ":0.0"
                && !d.starts_with(":0.")
            {
                return true;
            }
        }

        // One-shot pgrep check for common remote desktop daemons.
        // We check once and rely on internal caching in the caller loop.
        {
            use std::sync::atomic::AtomicU8;
            use std::sync::atomic::Ordering as AtomicOrdering;
            static REMOTE_DAEMON_CHECKED: AtomicU8 = AtomicU8::new(0);
            // 0 = unchecked, 1 = found, 2 = not found

            let cached = REMOTE_DAEMON_CHECKED.load(AtomicOrdering::Relaxed);
            if cached == 1 {
                return true;
            }
            if cached == 0 {
                let found = ["xrdp", "xrdp-sesman", "vino-server", "grd", "gnome-remote-desktop"]
                    .iter()
                    .any(|name| {
                        std::process::Command::new("pgrep")
                            .arg("-x")
                            .arg(name)
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(false)
                    });
                REMOTE_DAEMON_CHECKED.store(if found { 1 } else { 2 }, AtomicOrdering::Relaxed);
                if found {
                    log::info!("Remote desktop daemon detected via pgrep");
                    return true;
                }
            }
        }

        false
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
