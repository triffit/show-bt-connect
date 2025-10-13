// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winuser::SW_SHOW;
use std::ptr;

pub fn show_bluetooth_ui() {
    // Launching Bluetooth settings UI
    
    // First try the Action Center approach
    let uri_action_center = "ms-actioncenter:controlcenter/bluetooth";
    
    let result = unsafe {
        let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(std::iter::once(0)).collect();
        let file: Vec<u16> = OsStr::new(uri_action_center).encode_wide().chain(std::iter::once(0)).collect();
        
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOW,
        )
    };
    
    if result as usize > 32 {
        // Bluetooth UI launched successfully via Action Center
        return;
    }
    
    // Action Center failed, trying Settings app
    
    // Fallback: Launch Windows Settings app directly to Bluetooth page
    let uri_settings = "ms-settings:bluetooth";
    
    let result2 = unsafe {
        let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(std::iter::once(0)).collect();
        let file: Vec<u16> = OsStr::new(uri_settings).encode_wide().chain(std::iter::once(0)).collect();
        
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOW,
        )
    };
    
    if result2 as usize > 32 {
        // Bluetooth UI launched successfully via Settings app
    } else {
        // Failed to launch Bluetooth UI - both methods failed
    }
}

