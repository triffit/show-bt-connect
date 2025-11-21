// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, menu::{Menu, MenuItem, PredefinedMenuItem, Submenu}};
use crate::log_dbg;
use crate::config::AppResult;
use crate::audio_device::{enumerate_audio_devices, AudioDevice};

// (No longer needed - removed AUDIO_DEVICE_BASE_ID constant)

// Simple struct to hold menu item identifiers and audio device list.
pub struct TrayHandles {
    pub about_id: String,
    pub exit_id: String,
    pub audio_devices: Vec<AudioDevice>,
    pub audio_device_ids: Vec<String>, // Menu IDs for each audio device
}

pub struct TrayManager {
    icon: TrayIcon,
    handles: TrayHandles,
}

include!(concat!(env!("OUT_DIR"), "/icon_rgba.rs"));

pub fn load_icon() -> AppResult<Icon> { Ok(Icon::from_rgba(ICON_RGBA.to_vec(), ICON_WIDTH, ICON_HEIGHT)?) }

pub fn build_tray() -> AppResult<(TrayIcon, TrayHandles)> {
    let icon = load_icon()?;
    let menu = Menu::new();
    
    // Enumerate audio devices
    let audio_devices = enumerate_audio_devices().unwrap_or_else(|_e| {
        log_dbg!("tray: failed to enumerate audio devices: {}", _e);
        Vec::new()
    });

    let audio_device_ids = if !audio_devices.is_empty() {
        let audio_submenu = Submenu::new("Audio Devices", true);
        let mut ids = Vec::new();
        
        for device in &audio_devices {
            let label = if device.is_default {
                format!("✓ {}", device.name)
            } else {
                format!("    {}", device.name)
            };
            let device_item = MenuItem::new(label, true, None);
            ids.push(device_item.id().0.clone());
            audio_submenu.append(&device_item)?;
        }
        
        menu.append(&audio_submenu)?;
        menu.append(&PredefinedMenuItem::separator())?;
        ids
    } else {
        Vec::new()
    };
    
    let about_item = MenuItem::new("About", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
    menu.append(&about_item)?; 
    menu.append(&exit_item)?;
    
    let about_id = about_item.id().0.clone();
    let exit_id = exit_item.id().0.clone();
    
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Show Bluetooth Devices Panel")
        .with_icon(icon)
        .build()?;
    
    log_dbg!("tray: icon created with {} audio device(s)", audio_devices.len());
    
    Ok((tray_icon, TrayHandles { about_id, exit_id, audio_devices, audio_device_ids }))
}

impl TrayManager {
    pub fn new() -> AppResult<Self> {
        let (icon, handles) = build_tray()?;
        Ok(Self { icon, handles })
    }
    pub fn about_id(&self) -> &str { &self.handles.about_id }
    pub fn exit_id(&self) -> &str { &self.handles.exit_id }
    
    /// Get audio device by menu index
    pub fn get_audio_device(&self, idx: usize) -> Option<&AudioDevice> {
        self.handles.audio_devices.get(idx)
    }
    
    /// Check if a menu ID corresponds to an audio device selection
    pub fn audio_device_index(&self, menu_id: &str) -> Option<usize> {
        self.handles.audio_device_ids.iter().position(|id| id == menu_id)
    }
    
    pub fn recreate(&mut self) -> AppResult {
        let (icon, handles) = build_tray()?;
        self.icon = icon; // old icon dropped here
        self.handles = handles;
        Ok(())
    }
}