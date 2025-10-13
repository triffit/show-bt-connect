// SPDX-License-Identifier: MIT
//! Taskbar (Explorer) restart watcher: listens for the TaskbarCreated broadcast and triggers a callback.
use std::ptr::null_mut;
use std::sync::{OnceLock, Mutex};
use std::time::{Instant, Duration};
use crate::log_dbg;
use crate::config::AppResult;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{WNDCLASSW, RegisterClassW, CreateWindowExW, RegisterWindowMessageW, DestroyWindow, DefWindowProcW};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use crate::wide_strings::{WIDE_CLASSNAME, WIDE_TASKBAR_CREATED};

static CALLBACK: OnceLock<Box<dyn Fn() + Send + Sync + 'static>> = OnceLock::new();
static TASKBAR_CREATED_MSG_ID: OnceLock<u32> = OnceLock::new();
static LAST_RECREATE: OnceLock<Mutex<Instant>> = OnceLock::new();
const RECREATE_DEBOUNCE: Duration = Duration::from_millis(300);

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if let Some(taskbar_msg) = TASKBAR_CREATED_MSG_ID.get() {
        if msg == *taskbar_msg {
            let now = Instant::now();
            let do_call = {
                let m = LAST_RECREATE.get_or_init(|| Mutex::new(Instant::now() - RECREATE_DEBOUNCE));
                let mut guard = m.lock().unwrap();
                if now.duration_since(*guard) >= RECREATE_DEBOUNCE { *guard = now; true } else { false }
            };
            if do_call {
                log_dbg!("taskbar: TaskbarCreated received -> recreate tray");
                if let Some(cb) = CALLBACK.get() { cb(); }
            } else {
                log_dbg!("taskbar: TaskbarCreated suppressed (debounce)");
            }
            return 0;
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub struct TaskbarWatcher(HWND);

impl Drop for TaskbarWatcher { fn drop(&mut self) { unsafe { DestroyWindow(self.0); } } }

pub fn start<F>(callback: F) -> AppResult<TaskbarWatcher>
where F: Fn() + Send + Sync + 'static {
    CALLBACK.set(Box::new(callback)).ok();
    unsafe {
    let class_name = WIDE_CLASSNAME;
        let hinstance = GetModuleHandleW(null_mut());
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name.as_ptr(),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };
        if RegisterClassW(&wc) == 0 { return Err("RegisterClassW failed".into()); }
        let hwnd = CreateWindowExW(0, class_name.as_ptr(), class_name.as_ptr(), 0, 0,0,0,0, null_mut(), null_mut(), hinstance, null_mut());
        if hwnd.is_null() { return Err("CreateWindowExW failed".into()); }
    let name = WIDE_TASKBAR_CREATED;
        let msg_id = RegisterWindowMessageW(name.as_ptr());
        if msg_id == 0 { return Err("RegisterWindowMessageW failed".into()); }
        TASKBAR_CREATED_MSG_ID.set(msg_id).ok();
    log_dbg!("taskbar: watcher ready (hwnd={:?}, msg_id={msg_id})", hwnd);
        Ok(TaskbarWatcher(hwnd))
    }
}
