// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use winapi::shared::minwindef::{LRESULT, WPARAM, LPARAM, HINSTANCE, DWORD};
use winapi::um::winuser::{KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN, VK_LWIN, SetWindowsHookExW, CallNextHookEx, GetMessageW, MSG, VK_RWIN, UnhookWindowsHookEx};
use winapi::um::libloaderapi::GetModuleHandleW;

static HOOK_SET: AtomicBool = AtomicBool::new(false);
static STATE: Lazy<Mutex<HookState>> = Lazy::new(|| Mutex::new(HookState::default()));

#[derive(Default)]
struct HookState {
    win_down: bool,
    callback: Option<Box<dyn Fn() + Send + 'static>>,
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kbd = &*(lparam as *const KBDLLHOOKSTRUCT);
        let vk = kbd.vkCode as i32;
        let is_keydown = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
        if is_keydown {
            let mut state = STATE.lock().unwrap();
            if vk == VK_LWIN as i32 || vk == VK_RWIN as i32 { state.win_down = true; }
            else if vk == VK_K as i32 && state.win_down {
                if let Some(cb) = &state.callback { cb(); }
                state.win_down = false; // reset to avoid repeats
                return 1; // swallow event so system Win+K panel doesn't open (we handle)
            }
        } else {
            // On key up we can clear win flag if it was win key
            let vk = kbd.vkCode as i32;
            if vk == VK_LWIN as i32 || vk == VK_RWIN as i32 { STATE.lock().unwrap().win_down = false; }
        }
    }
    CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
}

const VK_K: i32 = 0x4B; // Virtual key code for 'K'

pub fn install_win_k_hook<F: Fn() + Send + 'static>(callback: F) -> Result<(), String> {
    if HOOK_SET.swap(true, Ordering::SeqCst) { return Ok(()); }
    {
        let mut st = STATE.lock().unwrap();
        st.callback = Some(Box::new(callback));
    }
    std::thread::spawn(|| unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(ptr::null());
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hinstance, 0 as DWORD);
        if hook.is_null() {
            HOOK_SET.store(false, Ordering::SeqCst);
            return;
        }
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {}
        let _ = UnhookWindowsHookEx(hook);
    });
    Ok(())
}