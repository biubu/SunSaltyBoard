use std::sync::{Arc, Mutex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::commands::http_client;
use crate::database::ClipboardItem;
use crate::AppState;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    Idle,
    Configured,
    Syncing,
    Synced,
    NotConfigured,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub connected: bool,
    pub last_sync: Option<String>,
    pub status: SyncState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPayload {
    pub items: Vec<ClipboardItem>,
    pub timestamp: String,
}

pub struct SyncManager {
    status: Arc<Mutex<SyncStatus>>,
    server_url: Arc<Mutex<Option<String>>>,
    auth_token: Arc<Mutex<Option<String>>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(SyncStatus {
                connected: false,
                last_sync: None,
                status: SyncState::Idle,
            })),
            server_url: Arc::new(Mutex::new(None)),
            auth_token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn configure(&self, server_url: Option<String>) {
        let mut url = self.server_url.lock().unwrap();
        *url = server_url;

        let mut status = self.status.lock().unwrap();
        if url.is_some() {
            status.status = SyncState::Configured;
        } else {
            status.status = SyncState::Idle;
            status.connected = false;
        }
    }

    pub fn set_auth_token(&self, token: Option<String>) {
        let mut t = self.auth_token.lock().unwrap();
        *t = token;
    }

    pub fn get_status(&self) -> SyncStatus {
        self.status.lock().unwrap().clone()
    }

    pub async fn trigger_sync(&self, state: &AppState) -> Result<SyncStatus, String> {
        let server_url = self.server_url.lock().unwrap().clone();
        let auth_token = self.auth_token.lock().unwrap().clone();

        {
            let mut status = self.status.lock().unwrap();
            status.status = SyncState::Syncing;
        }

        let url = match server_url {
            Some(u) if !u.is_empty() => u,
            _ => {
                let mut status = self.status.lock().unwrap();
                status.status = SyncState::NotConfigured;
                return Err("Sync server not configured".to_string());
            }
        };

        // Get all items from local database (lock dropped before await)
        let items = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.get_clipboard_history(1000, 0)
                .map_err(|e| e.to_string())?
        };

        let payload = SyncPayload {
            items,
            timestamp: Utc::now().to_rfc3339(),
        };

        match Self::sync_http(&url, auth_token.as_deref(), &payload).await {
            Ok(()) => {
                let mut status = self.status.lock().unwrap();
                status.connected = true;
                status.last_sync = Some(Utc::now().to_rfc3339());
                status.status = SyncState::Synced;
                Ok(status.clone())
            }
            Err(e) => {
                // Reset the status to a clean error string instead of
                // accumulating prefixes on repeated failures.
                let mut status = self.status.lock().unwrap();
                status.status = SyncState::Error;
                status.connected = false;
                Err(e)
            }
        }
    }

    async fn sync_http(url: &str, auth_token: Option<&str>, payload: &SyncPayload) -> Result<(), String> {
        log::info!("Syncing {} items to {}", payload.items.len(), url);

        let mut req = http_client().post(url).json(payload);
        if let Some(tok) = auth_token {
            if !tok.is_empty() {
                req = req.bearer_auth(tok);
            }
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        if status.is_success() {
            log::info!("Sync completed successfully");
            Ok(())
        } else {
            Err(format!("Server returned status {}", status.as_u16()))
        }
    }

    pub fn start(&self, _app_handle: AppHandle) {
        log::info!("Sync manager started (manual trigger only)");
    }

    pub fn stop(&self) {
        log::info!("Sync manager stopped");
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}
