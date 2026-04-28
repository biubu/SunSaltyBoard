use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            last_content: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn start(&self, app_handle: AppHandle) {
        let running = self.running.clone();
        let last_content = self.last_content.clone();

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
                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_content.lock().unwrap();
                    if *last != text {
                        let preview = text.chars().take(200).collect::<String>();
                        let event = ClipboardEvent {
                            content_type: ClipboardContentType::Text.as_str().to_string(),
                            content: text.clone(),
                            preview,
                            metadata: None,
                        };

                        if let Some(state) = app_handle.try_state::<AppState>() {
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

                            if let Ok(db) = state.db.lock() {
                                if let Err(e) = db.insert_clipboard_item(&item) {
                                    log::error!("Failed to insert clipboard item: {}", e);
                                }
                                // Enforce history limit
                                if let Ok(settings) = state.settings.lock() {
                                    let _ = db.prune_history(settings.max_history_size);
                                }
                            }

                            if let Err(e) = app_handle.emit("clipboard-changed", &item) {
                                log::error!("Failed to emit clipboard event: {}", e);
                            }
                        }

                        *last = text;
                    }
                }

                thread::sleep(Duration::from_millis(500));
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
