use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use windows::Win32::System::Registry::*;
use windows::Win32::Foundation::*;
use windows::core::PCWSTR;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

const APP_NAME: &str = "BluetoothManager";
const REGISTRY_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub start_with_windows: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            start_with_windows: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return Ok(config);
            }
        }
        
        Ok(Self::default())
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        
        // Apply the start with windows setting
        self.set_start_with_windows(self.start_with_windows)?;
        
        Ok(())
    }
    
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("BluetoothManager");
        path.push("config.json");
        path
    }
    
    fn set_start_with_windows(&self, enable: bool) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let mut key: HKEY = HKEY::default();
            
            // Open the registry key
            let key_path: Vec<u16> = OsStr::new(REGISTRY_KEY)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            
            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(key_path.as_ptr()),
                0,
                KEY_SET_VALUE | KEY_QUERY_VALUE,
                &mut key,
            );
            
            if result != ERROR_SUCCESS {
                return Err("Failed to open registry key".into());
            }
            
            if enable {
                // Add to startup
                let exe_path = std::env::current_exe()?;
                let exe_path_str = exe_path.to_string_lossy();
                let exe_path_wide: Vec<u16> = OsStr::new(exe_path_str.as_ref())
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                
                let value_name: Vec<u16> = OsStr::new(APP_NAME)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                
                let data_bytes = exe_path_wide.iter()
                    .flat_map(|&w| w.to_le_bytes())
                    .collect::<Vec<u8>>();
                
                let result = RegSetValueExW(
                    key,
                    PCWSTR(value_name.as_ptr()),
                    0,
                    REG_SZ,
                    Some(&data_bytes),
                );
                
                if result != ERROR_SUCCESS {
                    return Err("Failed to set registry value".into());
                }
            } else {
                // Remove from startup
                let value_name: Vec<u16> = OsStr::new(APP_NAME)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                
                let _ = RegDeleteValueW(key, PCWSTR(value_name.as_ptr()));
            }
            
            let _ = RegCloseKey(key);
        }
        
        Ok(())
    }
}

pub fn show_config_dialog(config: &mut Config) {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_YESNO, MB_ICONQUESTION, IDYES};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    let message = if config.start_with_windows {
        "Bluetooth Manager is currently set to start with Windows.\n\nDo you want to disable starting with Windows?"
    } else {
        "Bluetooth Manager is currently NOT set to start with Windows.\n\nDo you want to enable starting with Windows?"
    };
    
    let message_wide: Vec<u16> = OsStr::new(message)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let title_wide: Vec<u16> = OsStr::new("Bluetooth Manager Settings")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    unsafe {
        let result = MessageBoxW(
            None,
            windows::core::PCWSTR(message_wide.as_ptr()),
            windows::core::PCWSTR(title_wide.as_ptr()),
            MB_YESNO | MB_ICONQUESTION,
        );
        
        if result == IDYES {
            config.start_with_windows = !config.start_with_windows;
            let _ = config.save(); // Ignore errors in GUI mode
        }
    }
}
