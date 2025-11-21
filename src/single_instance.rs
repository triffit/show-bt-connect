// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit

//! Single instance enforcement via named mutex.
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use std::ffi::c_void;

// SAFETY: CreateMutexW manually declared for minimal binary size (avoiding additional windows-sys features).
// This is a valid approach for size-optimized binaries. Signature matches Win32 API exactly.
// Only called with null security attributes and immutable name pointer from UTF-16 buffer.
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
