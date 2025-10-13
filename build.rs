// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use std::fs;
use std::path::Path;

fn main() {
    // Expose version info to code for dynamic tooltip / logging
    if let Ok(ver) = std::env::var("CARGO_PKG_VERSION") {
        println!("cargo:rustc-env=APP_VERSION={ver}");
    }
    if cfg!(target_os = "windows") {
        embed_resource::compile("resources.rc", embed_resource::NONE);
    }

    // Generate icon RGBA at build time from app.ico for accurate tray display
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("icon_rgba.rs");
    match generate_icon_rs(&dest_path) {
        Ok(_) => println!("cargo:rerun-if-changed=app.ico"),
        Err(e) => {
            eprintln!("cargo:warning=Failed to generate icon from app.ico: {e}");
        }
    }
}

fn generate_icon_rs(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read("app.ico")?;
    let cursor = std::io::Cursor::new(bytes);
    let icon_dir = ico::IconDir::read(cursor)?;
    // Pick largest image
    let mut largest = None;
    for entry in icon_dir.entries() {
        let w = entry.width();
        let h = entry.height();
        let area = (w as u32) * (h as u32);
        if largest.map(|(_,_,a)| area > a).unwrap_or(true) {
            largest = Some((w, h, area));
        }
    }
    let (w, h, _) = largest.ok_or("No icon entries")?;
    // Decode the matching entry
    let mut rgba: Option<(Vec<u8>, u32, u32)> = None;
    for entry in icon_dir.entries() {
        if entry.width() == w && entry.height() == h {
            let img = entry.decode()?; // returns an image buffer (RGBA)
            let pixels = img.rgba_data().to_vec();
            rgba = Some((pixels, w as u32, h as u32));
            break;
        }
    }
    let (pixels, w, h) = rgba.ok_or("Failed to decode largest icon frame")?;
    let mut out = String::new();
    out.push_str("// Auto-generated from app.ico at build time.\n");
    out.push_str(&format!("pub const ICON_WIDTH: u32 = {w};\n"));
    out.push_str(&format!("pub const ICON_HEIGHT: u32 = {h};\n"));
    out.push_str(&format!("pub static ICON_RGBA: [u8; {}] = [", pixels.len()));
    for (i, b) in pixels.iter().enumerate() {
        if i % 20 == 0 { out.push('\n'); }
        out.push_str(&format!("{b},"));
    }
    out.push_str("\n];\n");
    fs::write(path, out)?;
    Ok(())
}