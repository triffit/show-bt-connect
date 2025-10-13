// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use tray_icon::Icon;

include!(concat!(env!("OUT_DIR"), "/icon_rgba.rs"));

pub fn load_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    Ok(Icon::from_rgba(ICON_RGBA.to_vec(), ICON_WIDTH, ICON_HEIGHT)?)
}