// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

#[macro_use]
mod log; // exports log_dbg! macro
mod bluetooth;
mod tray;
mod keyboard_hook;
mod app_state;
mod single_instance;
mod taskbar_restart;
mod wide_strings;
mod config;

use app_state::AppState;
use single_instance::{ensure_single_instance_wide, InstanceCheck};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::event::{Event, WindowEvent};
use tray_icon::{TrayIconEvent, menu::{MenuEvent}};

// Version injected by build.rs (fallback to placeholder if missing)
const VERSION: &str = match option_env!("APP_VERSION") { Some(v) => v, None => "0.0.0" };
use crate::wide_strings::WIDE_MUTEX_NAME;

#[derive(Debug)]
enum UserEvent { TrayEvent(TrayIconEvent), MenuEvent(MenuEvent), WinKHook, TrayRecreate }

use crate::config::AppResult;

fn main() -> AppResult {
    // Early CLI flags (before windows_subsystem hides console in release)
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-V" => { println!("restore-wink-bt {VERSION}"); return Ok(()); },
            _ => {}
        }
    }
    hide_console_window();

    // Single instance
    match ensure_single_instance_wide(WIDE_MUTEX_NAME) {
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
    let mut tray_manager = tray::TrayManager::new(VERSION)?;
    let mut about_id = tray_manager.about_id().to_string();
    let mut exit_id = tray_manager.exit_id().to_string();

    // Taskbar restart watcher: recreate tray icon when Explorer restarts.
    // Taskbar restart watcher: recreate tray after Explorer restarts
    let recreate_proxy = event_loop_proxy.clone();
    let _taskbar_watcher = taskbar_restart::start(move || { let _ = recreate_proxy.send_event(UserEvent::TrayRecreate); })?;

    // Keyboard hook -> user event
    let hook_proxy = event_loop_proxy.clone();
    let _hook_guard = keyboard_hook::install_win_k_hook(move || { let _ = hook_proxy.send_event(UserEvent::WinKHook); })?;

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
                        if let TrayIconEvent::Click { button: tray_icon::MouseButton::Left, .. } = tray_event {
                            log_dbg!("tray: left click -> toggle");
                            state.on_tray_left_click();
                        }
                    },
                    UserEvent::MenuEvent(menu_event) => {
                        let id = menu_event.id.0.as_str();
                        if id == about_id.as_str() { show_about_dialog(); }
                        else if id == exit_id.as_str() { elwt.exit(); }
                    },
                    UserEvent::WinKHook => { log_dbg!("hook: Win+K intercepted -> toggle"); state.on_win_k(); },
                    UserEvent::TrayRecreate => {
                        log_dbg!("taskbar: restart detected -> recreating tray icon");
                        if let Err(_e) = tray_manager.recreate(VERSION) { log_dbg!("tray: recreate failed: {_e}"); }
                        about_id = tray_manager.about_id().to_string();
                        exit_id = tray_manager.exit_id().to_string();
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
    use crate::wide_strings::{to_wide_null, WIDE_ABOUT};
    let text = format!("Restore Win+K: Bluetooth Devices Panel\nVersion {VERSION}\nRestores the fast Win+K Bluetooth devices panel (first press) without losing Cast (second press).\n© 2025 Triffit");
    // NOTE: If About is opened frequently and VERSION is static during process lifetime, we could
    // cache this wide buffer in a Lazy<Vec<u16>> or const (with compile-time version) to avoid
    // allocation here. Kept dynamic to reflect runtime-injected VERSION without extra storage.
    let wide = to_wide_null(&text);
    let title_wide = WIDE_ABOUT;
    unsafe { MessageBoxW(std::ptr::null_mut(), wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONINFORMATION); }
}
