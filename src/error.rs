use thiserror::Error;

#[derive(Error, Debug)]
pub enum BluetoothManagerError {
    #[error("Failed to initialize system tray: {0}")]
    TrayInitialization(String),
    
    #[error("Failed to register hotkey: {0}")]
    HotkeyRegistration(String),
    
    #[error("Failed to launch Bluetooth UI: {0}")]
    BluetoothLaunch(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Windows API error: {0}")]
    WindowsApi(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BluetoothManagerError>;