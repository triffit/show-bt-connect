// SPDX-License-Identifier: MIT
//! Single instance enforcement via named mutex.
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use std::ffi::c_void;

// SAFETY: Declared manually because the minimal windows-sys feature set used does not expose
// CreateMutexW; signature matches Win32 API. Only called with null security attributes and
// immutable name pointer derived from a Rust UTF-16 buffer that lives for call duration.
extern "system" { fn CreateMutexW(lpMutexAttributes: *mut c_void, bInitialOwner: i32, lpName: *const u16) -> HANDLE; }

pub enum InstanceCheck {
    First,
    AlreadyRunning,
    Failed(u32),
}


/// Wide version: accepts a pre-encoded, null-terminated UTF-16 slice.
pub fn ensure_single_instance_wide(mutex_name_wide: &[u16]) -> InstanceCheck {
    debug_assert!(!mutex_name_wide.is_empty() && *mutex_name_wide.last().unwrap() == 0);
    unsafe {
        let h = CreateMutexW(std::ptr::null_mut(), 0, mutex_name_wide.as_ptr());
        if h.is_null() { return InstanceCheck::Failed(GetLastError()); }
        let err = GetLastError();
        if err == ERROR_ALREADY_EXISTS { InstanceCheck::AlreadyRunning } else { InstanceCheck::First }
    }
}
