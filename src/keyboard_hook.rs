// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering, AtomicU32};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use windows_sys::Win32::Foundation::{LRESULT, WPARAM, LPARAM, HINSTANCE};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, SetWindowsHookExW, CallNextHookEx, GetMessageW, MSG,
    UnhookWindowsHookEx, PostThreadMessageW, WM_QUIT, WM_KEYDOWN, WM_SYSKEYDOWN,
};
// Define VK constants manually (not all exposed as constants in windows-sys 0.59 feature set)
const VK_LWIN: i32 = 0x5B;
const VK_RWIN: i32 = 0x5C;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use std::time::{Instant, Duration};
use crate::config::PASS_THROUGH_WINDOW_MS;
use std::thread::JoinHandle;

// Virtual key codes for Win+K combination
const VK_K: i32 = 0x4B;

static HOOK_SET: AtomicBool = AtomicBool::new(false);
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static STATE: Lazy<Mutex<HookState>> = Lazy::new(|| Mutex::new(HookState::default()));

#[derive(Default)]
struct HookState {
    win_down: bool,
    swallowed_first: bool,
    first_k_time: Option<Instant>,
    callback: Option<Box<dyn Fn() + Send + 'static>>,
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 { return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam); }
    let kbd = &*(lparam as *const KBDLLHOOKSTRUCT);
    let vk = kbd.vkCode as i32;
    let is_keydown = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
    if is_keydown {
        let mut state = STATE.lock().unwrap();
        match vk {
            VK_LWIN | VK_RWIN => { state.win_down = true; },
            VK_K if state.win_down => {
                let now = Instant::now();
                let window_ok = state.first_k_time.is_some_and(|t| now.duration_since(t) < Duration::from_millis(PASS_THROUGH_WINDOW_MS));
                
                if !state.swallowed_first {
                    // First Win+K: swallow and launch Bluetooth panel
                    state.swallowed_first = true;
                    state.first_k_time = Some(now);
                    if let Some(cb) = &state.callback { cb(); }
                    return 1;
                } else if window_ok {
                    // Second Win+K within window: pass through to Windows for native Cast flyout
                    // Don't return 1, fall through to CallNextHookEx
                } else {
                    // Outside window: treat as new first press
                    state.swallowed_first = true;
                    state.first_k_time = Some(now);
                    if let Some(cb) = &state.callback { cb(); }
                    return 1;
                }
            },
            _ => {}
        }
    } else if vk == VK_LWIN || vk == VK_RWIN {
        let mut state = STATE.lock().unwrap();
        state.win_down = false;
        state.swallowed_first = false;
        state.first_k_time = None;
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

pub struct HookGuard(Option<JoinHandle<()>>);

impl Drop for HookGuard {
    fn drop(&mut self) {
        let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
        if tid != 0 { unsafe { PostThreadMessageW(tid, WM_QUIT, 0, 0); } }
        if let Some(handle) = self.0.take() {
            // Join with a short timeout by spawning a watcher; if join hangs we just detach.
            // (Keyboard hook thread should exit promptly after WM_QUIT.)
            let start = Instant::now();
            // Best-effort join (no std timeout join; emulate by try_join pattern)
            // We'll block up to ~200ms; acceptable at process teardown.
            while start.elapsed() < Duration::from_millis(200) {
                if handle.is_finished() { let _ = handle.join(); break; }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        HOOK_SET.store(false, Ordering::SeqCst);
    }
}

pub fn install_win_k_hook<F: Fn() + Send + 'static>(callback: F) -> Result<HookGuard, String> {
    if HOOK_SET.swap(true, Ordering::SeqCst) { return Err("Keyboard hook already installed".into()); }
    {
        let mut st = STATE.lock().unwrap();
        st.callback = Some(Box::new(callback));
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        HOOK_THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
        let hinstance: HINSTANCE = unsafe { GetModuleHandleW(ptr::null()) };
        let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hinstance, 0) };
        if hook.is_null() {
            let _ = tx.send(false);
            HOOK_SET.store(false, Ordering::SeqCst);
            return;
        }
        let _ = tx.send(true);
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        while unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) } > 0 {}
        unsafe { UnhookWindowsHookEx(hook); }
    });
    match rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(true) => Ok(HookGuard(Some(handle))),
        Ok(false) => Err("Failed to install keyboard hook".into()),
        Err(_) => { HOOK_SET.store(false, Ordering::SeqCst); Err("Timed out waiting for hook install".into()) }
    }
}