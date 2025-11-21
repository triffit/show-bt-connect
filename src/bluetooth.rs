// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW;
use std::ptr;
use crate::log_dbg;
use crate::utf16_strings::{encode_utf16_null, UTF16_OPEN};

pub fn show_bluetooth_ui() -> bool {
    // Primary: Action Center flyout (closest to legacy Win+K behavior)
    if launch_uri("ms-actioncenter:controlcenter/bluetooth") { return true; }
    log_dbg!("bluetooth: primary action center URI failed; falling back to Settings");
    // Fallback: Settings Bluetooth page
    launch_uri("ms-settings:bluetooth")
}

fn launch_uri(uri: &str) -> bool {
    let operation = UTF16_OPEN;
    let file = encode_utf16_null(uri);
    let code = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOW,
        ) as isize
    };
    if code > 32 {
        #[cfg(any(debug_assertions, feature = "verbose-log"))]
        log_dbg!("bluetooth: uri launch ok ({} code={})", uri, code);
        true
    } else {
        // Always log failures even in release minimal mode for troubleshooting.
        log_dbg!("bluetooth: uri launch failed ({} code={})", uri, code);
        false
    }
}

