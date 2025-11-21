// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

#[macro_use]
mod log; // exports log_dbg! macro
mod audio_device;
mod bluetooth;
mod tray;
mod keyboard_hook;
mod app_state;
mod single_instance;
mod utf16_strings;
mod config;

use app_state::AppState;
use audio_device::{set_default_audio_device, register_device_change_callback};
use single_instance::{ensure_single_instance_wide, InstanceCheck};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::event::{Event, WindowEvent};
use tray_icon::{TrayIconEvent, menu::{MenuEvent}};

// Version injected by build.rs (fallback to placeholder if missing)
const VERSION: &str = match option_env!("APP_VERSION") { Some(v) => v, None => "0.0.0" };
use crate::utf16_strings::UTF16_MUTEX_NAME;

#[derive(Debug)]
enum UserEvent { TrayEvent(TrayIconEvent), MenuEvent(MenuEvent), WinKHook, RefreshAudioDevices }

use crate::config::AppResult;

fn main() -> AppResult {
    // Early CLI flags (before windows_subsystem hides console in release)
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-V" => { println!("ShowBTConnect {VERSION}"); return Ok(()); },
            _ => {}
        }
    }
    hide_console_window();

    // Single instance
    match ensure_single_instance_wide(UTF16_MUTEX_NAME) {
        InstanceCheck::First => {},
        InstanceCheck::AlreadyRunning => return Ok(()),
        InstanceCheck::Failed(_code) => {
            log_dbg!("single-instance: creation failed code={_code}");
            // continue anyway to avoid silent failure, but could early return
        }
    }

    // Event loop
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();
    let mut state = AppState::new();

    // Tray & menu (reusable builder)
    let mut tray_manager = tray::TrayManager::new()?;
    let mut about_id = tray_manager.about_id().to_string();
    let mut exit_id = tray_manager.exit_id().to_string();

    // Keyboard hook -> user event
    let hook_proxy = event_loop_proxy.clone();
    let _hook_guard = keyboard_hook::install_win_k_hook(move || { let _ = hook_proxy.send_event(UserEvent::WinKHook); })?;

    // Audio device change notifications (event-driven, no polling!)
    let audio_proxy = event_loop_proxy.clone();
    let _audio_notification_guard = register_device_change_callback(std::sync::Arc::new(move || {
        let _ = audio_proxy.send_event(UserEvent::RefreshAudioDevices);
    }))?;

    // Tray + menu handlers
    let proxy_clone = event_loop_proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |event| { let _ = proxy_clone.send_event(UserEvent::TrayEvent(event)); }));
    let proxy_clone = event_loop_proxy.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| { let _ = proxy_clone.send_event(UserEvent::MenuEvent(event)); }));

    log_dbg!("core: started version {VERSION}");

    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => { elwt.exit(); },
            Event::UserEvent(user_event) => {
                match user_event {
                    UserEvent::TrayEvent(tray_event) => {
                        match tray_event {
                            TrayIconEvent::Click { button: tray_icon::MouseButton::Left, .. } => {
                                log_dbg!("tray: left click -> toggle");
                                state.on_tray_left_click();
                            }
                            TrayIconEvent::Click { button: tray_icon::MouseButton::Right, .. } => {
                                log_dbg!("tray: right click -> force close BT panel if open");
                                // Force close by calling show again if we think it's open
                                if state.is_panel_thought_open() {
                                    bluetooth::show_bluetooth_ui();
                                    state.mark_panel_closed();
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                            }
                            _ => {}
                        }
                    },
                    UserEvent::MenuEvent(menu_event) => {
                        let id = menu_event.id.0.as_str();
                        if id == about_id.as_str() { 
                            show_about_dialog(); 
                        }
                        else if id == exit_id.as_str() { 
                            elwt.exit(); 
                        }
                        else if let Some(device_idx) = tray_manager.audio_device_index(id) {
                            // User selected an audio device
                            if let Some(device) = tray_manager.get_audio_device(device_idx) {
                                log_dbg!("audio: user selected device: {}", device.name);
                                match set_default_audio_device(&device.id) {
                                    Ok(()) => {
                                        log_dbg!("audio: successfully set default device");
                                        // Recreate tray to update checkmark
                                        if let Err(_e) = tray_manager.recreate() {
                                            log_dbg!("tray: recreate after device switch failed: {_e}");
                                        } else {
                                            about_id = tray_manager.about_id().to_string();
                                            exit_id = tray_manager.exit_id().to_string();
                                        }
                                    }
                                    Err(_e) => {
                                        log_dbg!("audio: failed to set default device: {_e}");
                                    }
                                }
                            }
                        }
                    },
                    UserEvent::WinKHook => { log_dbg!("hook: Win+K intercepted -> toggle"); state.on_win_k(); },
                    UserEvent::RefreshAudioDevices => {
                        // Audio device change notification (event-driven, triggered only when devices change)
                        if let Err(_e) = tray_manager.recreate() {
                            log_dbg!("audio: device list refresh failed: {}", _e);
                        } else {
                            about_id = tray_manager.about_id().to_string();
                            exit_id = tray_manager.exit_id().to_string();
                        }
                    }
                }
            },
            _ => {}
        }
    });
    Ok(())
}

fn hide_console_window() {
    #[cfg(not(debug_assertions))]
    unsafe {
        use windows_sys::Win32::System::Console::GetConsoleWindow; use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    let cw = GetConsoleWindow(); if !cw.is_null() { ShowWindow(cw, SW_HIDE); }
    }
}

fn show_about_dialog() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONINFORMATION};
    use windows_sys::Win32::Foundation::HWND;
    use crate::utf16_strings::{encode_utf16_null, UTF16_ABOUT};
    
    // Load the application icon from embedded resources
    let icon_handle = unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE};
        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        
        let hinst = GetModuleHandleW(std::ptr::null());
        LoadImageW(hinst, 1 as *const u16, IMAGE_ICON, 0, 0, LR_DEFAULTSIZE)
    };
    
    let text = format!("ShowBTConnect: Show Bluetooth Devices Panel\nRestores the fast Win+K Bluetooth devices panel.\n\nVersion {VERSION}, © 2025, Triffit");
    let wide = encode_utf16_null(&text);
    let title_wide = UTF16_ABOUT;
    
    // Create a message box with custom icon
    if !icon_handle.is_null() {
        // Use MSGBOXPARAMS for custom icon
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxIndirectW, MSGBOXPARAMSW};
        let mut params = MSGBOXPARAMSW {
            cbSize: std::mem::size_of::<MSGBOXPARAMSW>() as u32,
            hwndOwner: std::ptr::null_mut() as HWND,
            hInstance: unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null()) },
            lpszText: wide.as_ptr(),
            lpszCaption: title_wide.as_ptr(),
            dwStyle: MB_OK | 0x00000050, // MB_USERICON
            lpszIcon: icon_handle as *const u16,
            dwContextHelpId: 0,
            lpfnMsgBoxCallback: None,
            dwLanguageId: 0,
        };
        unsafe { MessageBoxIndirectW(&mut params) };
    } else {
        // Fallback to standard icon
        unsafe { MessageBoxW(std::ptr::null_mut(), wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONINFORMATION); }
    }
}
