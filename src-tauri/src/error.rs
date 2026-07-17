use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    Clipboard(String),
    Settings(String),
    Sync(String),
    Hotkey(String),
    Window(String),
    IO(String),
    Serialization(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Clipboard(msg) => write!(f, "Clipboard error: {}", msg),
            AppError::Settings(msg) => write!(f, "Settings error: {}", msg),
            AppError::Sync(msg) => write!(f, "Sync error: {}", msg),
            AppError::Hotkey(msg) => write!(f, "Hotkey error: {}", msg),
            AppError::Window(msg) => write!(f, "Window error: {}", msg),
            AppError::IO(msg) => write!(f, "IO error: {}", msg),
            AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IO(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
