use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::database::ClipboardItem;
use crate::AppState;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub connected: bool,
    pub last_sync: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPayload {
    pub items: Vec<ClipboardItem>,
    pub timestamp: String,
}

pub struct SyncManager {
    running: Arc<AtomicBool>,
    status: Arc<Mutex<SyncStatus>>,
    server_url: Arc<Mutex<Option<String>>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            status: Arc::new(Mutex::new(SyncStatus {
                connected: false,
                last_sync: None,
                status: "idle".to_string(),
            })),
            server_url: Arc::new(Mutex::new(None)),
        }
    }

    pub fn configure(&self, server_url: Option<String>) {
        let mut url = self.server_url.lock().unwrap();
        *url = server_url;

        let mut status = self.status.lock().unwrap();
        if url.is_some() {
            status.status = "configured".to_string();
        } else {
            status.status = "idle".to_string();
            status.connected = false;
        }
    }

    pub fn get_status(&self) -> SyncStatus {
        self.status.lock().unwrap().clone()
    }

    pub async fn trigger_sync(&self, state: &AppState) -> Result<SyncStatus, String> {
        let server_url = self.server_url.lock().unwrap().clone();

        let mut status = self.status.lock().unwrap();
        status.status = "syncing".to_string();
        let _current_status = status.clone();
        drop(status);

        if server_url.is_none() {
            let mut status = self.status.lock().unwrap();
            status.status = "not_configured".to_string();
            return Err("Sync server not configured".to_string());
        }

        let url = server_url.unwrap();

        // Get all items from local database
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let items = db.get_clipboard_history(1000, 0)
            .map_err(|e| e.to_string())?;

        // Prepare sync payload
        let payload = SyncPayload {
            items,
            timestamp: Utc::now().to_rfc3339(),
        };

        // Perform HTTP POST to sync server
        match Self::sync_http(&url, &payload).await {
            Ok(_) => {
                let mut status = self.status.lock().unwrap();
                status.connected = true;
                status.last_sync = Some(Utc::now().to_rfc3339());
                status.status = "synced".to_string();
                Ok(status.clone())
            }
            Err(e) => {
                let mut status = self.status.lock().unwrap();
                status.status = format!("error: {}", e);
                status.connected = false;
                Err(e)
            }
        }
    }

    async fn sync_http(url: &str, payload: &SyncPayload) -> Result<(), String> {
        // Using reqwest would be ideal here, but to avoid adding dependencies,
        // we'll use a simple HTTP client implementation
        // For production, you'd want to add reqwest to Cargo.toml

        log::info!("Sync to {} with {} items", url, payload.items.len());

        // Placeholder - actual implementation would use reqwest or similar
        // For now, we just simulate a successful sync after a delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // If we have a real server URL, we'd make the HTTP request here
        // For now, return success if configured
        Ok(())
    }

    pub fn start(&self, _app_handle: AppHandle) {
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);
        log::info!("Sync manager started");
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        log::info!("Sync manager stopped");
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}
