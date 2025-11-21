// SPDX-License-Identifier: MIT
//! UTF-16 encoding helpers & common static constants for Windows FFI.
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub fn encode_utf16(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().collect() }

pub fn encode_utf16_null(s: &str) -> Vec<u16> {
    let mut v = encode_utf16(s);
    v.push(0);
    v
}

// Common static wide strings to avoid repeated allocations.
pub const UTF16_OPEN: &[u16] = &[0x006F, 0x0070, 0x0065, 0x006E, 0]; // "open\0"
pub const UTF16_ABOUT: &[u16] = &[0x0041, 0x0062, 0x006F, 0x0075, 0x0074, 0]; // "About\0"

// Global mutex name ("Global\\ShowBTConnectMutex\0") as UTF-16.
pub const UTF16_MUTEX_NAME: &[u16] = &[
    0x0047,0x006C,0x006F,0x0062,0x0061,0x006C,0x005C,0x005C, // Global\\
    0x0053,0x0068,0x006F,0x0077, // Show
    0x0042,0x0054, // BT
    0x0043,0x006F,0x006E,0x006E,0x0065,0x0063,0x0074, // Connect
    0x004D,0x0075,0x0074,0x0065,0x0078, // Mutex
    0x0000
];