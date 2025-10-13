// SPDX-License-Identifier: MIT// (deprecated) Former config/autostart logic removed. Safe to delete.

//! Centralized constants and common result alias.// Keeping stub only because automated deletion failed in this environment.

use std::time::Duration;// No code references this module; removing it from version control is recommended.


#[allow(dead_code)]
pub type AppResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Timing windows
pub const CLICK_DEBOUNCE: Duration = Duration::from_millis(250);
pub const TOGGLE_MIN_HIDE: Duration = Duration::from_millis(800);
pub const PASS_THROUGH_WINDOW_MS: u64 = 1200; // Win+K pass-through cast window
