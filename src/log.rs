// SPDX-License-Identifier: MIT
// Central lightweight logging macro (compiled out in release unless explicitly enabled)
// Usage: log_dbg!("message {}", value);
#[cfg(any(debug_assertions, feature = "verbose-log"))]
#[macro_export]
macro_rules! log_dbg { ($($t:tt)*) => { eprintln!("[ShowBTConnect] {}", format!($($t)*)); }; }
#[cfg(not(any(debug_assertions, feature = "verbose-log")))]
#[macro_export]
macro_rules! log_dbg { ($($t:tt)*) => {}; }
