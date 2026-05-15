use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine};

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
    last_text: Arc<Mutex<String>>,
    last_image: Arc<Mutex<String>>,
    last_html: Arc<Mutex<String>>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            last_text: Arc::new(Mutex::new(String::new())),
            last_image: Arc::new(Mutex::new(String::new())),
            last_html: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn start(&self, app_handle: AppHandle) {
        let running = self.running.clone();
        let last_text = self.last_text.clone();
        let last_image = self.last_image.clone();
        let last_html = self.last_html.clone();

        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            log::info!("Clipboard monitor starting");
            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to initialize clipboard: {}", e);
                    return;
                }
            };

            while running.load(Ordering::SeqCst) {
                let mut got_content = false;

                if let Ok(image) = clipboard.get_image() {
                    let rgba_b64 = STANDARD.encode(&image.bytes);
                    let image_key = format!("{}x{}:{}", image.width, image.height, &rgba_b64[..rgba_b64.len().min(64)]);
                    let mut last = last_image.lock().unwrap();
                    if *last != image_key {
                        let metadata = serde_json::json!({
                            "width": image.width,
                            "height": image.height,
                            "format": "rgba"
                        }).to_string();

                        let event = ClipboardEvent {
                            content_type: ClipboardContentType::Image.as_str().to_string(),
                            content: rgba_b64.clone(),
                            preview: format!("Image {}x{}", image.width, image.height),
                            metadata: Some(metadata),
                        };

                        emit_and_store(&app_handle, &event);
                        *last = image_key;
                        got_content = true;
                    }
                }

                if let Ok(html) = clipboard.get().html() {
                    let mut last = last_html.lock().unwrap();
                    if *last != html {
                        let preview = strip_html_tags(&html).chars().take(200).collect::<String>();

                        let event = ClipboardEvent {
                            content_type: ClipboardContentType::Html.as_str().to_string(),
                            content: html.clone(),
                            preview,
                            metadata: None,
                        };

                        emit_and_store(&app_handle, &event);
                        *last = html;
                        got_content = true;
                    }
                }

                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_text.lock().unwrap();
                    if *last != text {
                        let preview = text.chars().take(200).collect::<String>();
                        let event = ClipboardEvent {
                            content_type: ClipboardContentType::Text.as_str().to_string(),
                            content: text.clone(),
                            preview,
                            metadata: None,
                        };

                        emit_and_store(&app_handle, &event);
                        *last = text;
                        got_content = true;
                    }
                }

                if !got_content {
                    thread::sleep(Duration::from_millis(500));
                } else {
                    thread::sleep(Duration::from_millis(200));
                }
            }

            log::info!("Clipboard monitor stopped");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;
    for c in html.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
        } else if !inside_tag {
            result.push(c);
        }
    }
    result
}

fn emit_and_store(app_handle: &AppHandle, event: &ClipboardEvent) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(db) = state.db.lock() {
            let dedup_key = if event.content_type == "image" {
                event.preview.clone()
            } else {
                event.content.clone()
            };

            if let Ok(Some(existing_id)) = db.find_by_content(&dedup_key) {
                if let Err(e) = db.update_item_timestamp(&existing_id) {
                    log::error!("Failed to update item timestamp: {}", e);
                }
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

                if let Err(e) = app_handle.emit("clipboard-changed", &item) {
                    log::error!("Failed to emit clipboard event: {}", e);
                }
            }
        }
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
