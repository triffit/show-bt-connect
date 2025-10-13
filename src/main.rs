// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bluetooth;
mod tray;
mod keyboard_hook;

use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::event::{Event, WindowEvent};
use tray_icon::{TrayIconBuilder, TrayIconEvent, TrayIcon, menu::{Menu, MenuItem, MenuEvent}};

// Debounce / heuristic constants (ms)
const CLICK_DEBOUNCE_MS: u128 = 250;
const TOGGLE_MIN_HIDE_MS: u128 = 800;

// Version injected by build.rs (fallback to placeholder if missing)
const VERSION: &str = match option_env!("APP_VERSION") { Some(v) => v, None => "0.0.0" };

// Lightweight debug logging macro (compiled out in release)
#[cfg(debug_assertions)]
macro_rules! log_dbg { ($($t:tt)*) => { eprintln!("[btm] {}", format!($($t)*)); }; }
#[cfg(not(debug_assertions))]
macro_rules! log_dbg { ($($t:tt)*) => {}; }

#[derive(Debug, Clone)]
pub struct AppState {
    pub last_click_time: std::time::Instant,
    pub last_launch_time: std::time::Instant,
    pub last_user_thought_open: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            last_click_time: std::time::Instant::now(),
            last_launch_time: std::time::Instant::now(),
            last_user_thought_open: false,
        }
    }

    fn toggle_bluetooth_ui_internal(&mut self) -> bool {
        let now = std::time::Instant::now();
        // Debounce click storms
        if now.duration_since(self.last_click_time).as_millis() < CLICK_DEBOUNCE_MS { return true; }
        self.last_click_time = now;

        // Heuristic: if user believes it is open AND at least 800ms passed since last launch, we attempt to "close" by launching action center again (toggling sometimes hides) else we open again.
    let since_launch = now.duration_since(self.last_launch_time).as_millis();
    if self.last_user_thought_open && since_launch > TOGGLE_MIN_HIDE_MS {
            // Attempt to re-launch to cause hide behavior if Windows treats it as toggle; then mark closed.
            std::thread::spawn(|| { let _ = bluetooth::show_bluetooth_ui(); });
            self.last_user_thought_open = false;
            true
        } else {
            // Open path
            std::thread::spawn(|| { let _ = bluetooth::show_bluetooth_ui(); });
            self.last_launch_time = now;
            self.last_user_thought_open = true;
            true
        }
    }

    fn launch_bluetooth_async(&mut self) -> bool {
        self.toggle_bluetooth_ui_internal()
    }
}


#[derive(Debug)]
enum UserEvent {
    TrayEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
    WinKHook,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Hide console window for release builds
    #[cfg(not(debug_assertions))]
    {
        use winapi::um::wincon::GetConsoleWindow;
        use winapi::um::winuser::{ShowWindow, SW_HIDE};
        unsafe {
            let console_window = GetConsoleWindow();
            if !console_window.is_null() {
                ShowWindow(console_window, SW_HIDE);
            }
        }
    }

    // Enforce single instance using a named mutex
    unsafe {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::synchapi::CreateMutexW;
        use winapi::um::errhandlingapi::GetLastError;
        use winapi::shared::winerror::ERROR_ALREADY_EXISTS;

        let name: Vec<u16> = OsStr::new("Global\\ShowBluetoothManagerMutex")
            .encode_wide().chain(std::iter::once(0)).collect();
        let handle = CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr());
        if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            return Ok(()); // Exit silently (second instance)
        }
        // (Optional) We intentionally never close the mutex handle so it stays valid until process exit.
    }
    
    // Starting Restore Win+K: Bluetooth Devices Panel (single-instance enforced)
    
    // Initialize the event loop with custom event type
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();
    
    // Load app state
    let mut app_state = AppState::new();
    
    // Create tray icon
    let icon = tray::load_icon()?;
    
    // Create context menu
    let menu = Menu::new();
    let about_item = MenuItem::new("About", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
    menu.append(&about_item)?;
    menu.append(&exit_item)?;
    let about_id = about_item.id().0.clone();
    let exit_id = exit_item.id().0.clone();
    
    // Build tray icon
    let tooltip = format!("Restore Win+K: Bluetooth Devices Panel v{VERSION} - Click to open Bluetooth devices");
    let _tray_icon: TrayIcon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(&tooltip)
        .with_icon(icon)
        .build()?;
        
    // Try to force Win+K via low-level hook first
    let hook_proxy = event_loop_proxy.clone();
    let _ = keyboard_hook::install_win_k_hook(move || {
        let _ = hook_proxy.send_event(UserEvent::WinKHook);
    });

    // (Removed global-hotkey fallback to reduce size.)
    
    // Set up event handlers
    let proxy_clone = event_loop_proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy_clone.send_event(UserEvent::TrayEvent(event));
    }));
    
    let proxy_clone = event_loop_proxy.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let _ = proxy_clone.send_event(UserEvent::MenuEvent(event));
    }));
    
    // (Removed global hotkey event handler.)
    
    // Clone state for event handling
    // Restore Win+K: Bluetooth Devices Panel started successfully. Look for the icon in your system tray.
    log_dbg!("Started version {VERSION}");
    
    // Event handling - this keeps the app running
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);
        
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                // Application shutting down
                elwt.exit();
            },
            Event::UserEvent(user_event) => {
                match user_event {
                    UserEvent::TrayEvent(tray_event) => {
                        match tray_event {
                            TrayIconEvent::Click { button: tray_icon::MouseButton::Left, .. } => {
                                let _success = app_state.launch_bluetooth_async();
                            },
                            _ => {}
                        }
                    },
                    UserEvent::MenuEvent(menu_event) => {
                        let id_str = menu_event.id.0.as_str();
                        if id_str == about_id.as_str() {
                            show_about_dialog();
                        } else if id_str == exit_id.as_str() {
                            keyboard_hook::shutdown_hook();
                            elwt.exit();
                        }
                    },
                    UserEvent::WinKHook => {
                        log_dbg!("Win+K first press intercepted -> launching Bluetooth UI");
                        let _success = app_state.launch_bluetooth_async();
                    }
                }
            },
            _ => {}
        }
    })?;
    
    Ok(())
}

fn show_about_dialog() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{MessageBoxW, MB_OK, MB_ICONINFORMATION};
    let text = format!("Restore Win+K: Bluetooth Devices Panel\nVersion {VERSION}\nRestores the fast Win+K Bluetooth devices panel (first press) without losing Cast (second press).\n© 2025 Triffit");
    let wide: Vec<u16> = OsStr::new(&text).encode_wide().chain(std::iter::once(0)).collect();
    let title_wide: Vec<u16> = OsStr::new("About").encode_wide().chain(std::iter::once(0)).collect();
    unsafe { MessageBoxW(std::ptr::null_mut(), wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONINFORMATION); }
}
