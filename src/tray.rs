// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, menu::{Menu, MenuItem}};
use crate::log_dbg;
use crate::config::AppResult;

// Simple struct to hold menu item identifiers we care about.
pub struct TrayHandles {
    pub about_id: String,
    pub exit_id: String,
}

pub struct TrayManager {
    icon: TrayIcon,
    handles: TrayHandles,
}

include!(concat!(env!("OUT_DIR"), "/icon_rgba.rs"));

pub fn load_icon() -> AppResult<Icon> { Ok(Icon::from_rgba(ICON_RGBA.to_vec(), ICON_WIDTH, ICON_HEIGHT)?) }

pub fn build_tray(version: &str) -> AppResult<(TrayIcon, TrayHandles)> {
    let icon = load_icon()?;
    let menu = Menu::new();
    let about_item = MenuItem::new("About", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
    menu.append(&about_item)?; menu.append(&exit_item)?;
    let about_id = about_item.id().0.clone();
    let exit_id = exit_item.id().0.clone();
    let tooltip = format!("Restore Win+K: Bluetooth Devices Panel v{version}");
    let tray_icon: TrayIcon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(&tooltip)
        .with_icon(icon)
        .build()?;
    log_dbg!("tray: icon created/recreated");
    Ok((tray_icon, TrayHandles { about_id, exit_id }))
}

impl TrayManager {
    pub fn new(version: &str) -> AppResult<Self> {
        let (icon, handles) = build_tray(version)?;
        Ok(Self { icon, handles })
    }
    pub fn about_id(&self) -> &str { &self.handles.about_id }
    pub fn exit_id(&self) -> &str { &self.handles.exit_id }
    pub fn recreate(&mut self, version: &str) -> AppResult {
        let (icon, handles) = build_tray(version)?;
        self.icon = icon; // old icon dropped here
        self.handles = handles;
        Ok(())
    }
}