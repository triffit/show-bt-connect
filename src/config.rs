// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit

//! Centralized constants and common result type alias.

use std::time::Duration;

pub type AppResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Timing windows
pub const CLICK_DEBOUNCE: Duration = Duration::from_millis(250);
pub const TOGGLE_MIN_HIDE: Duration = Duration::from_millis(800);
pub const PASS_THROUGH_WINDOW_MS: u64 = 1200; // Win+K pass-through window for second press
