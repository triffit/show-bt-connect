// SPDX-License-Identifier: MIT
//! Application state & heuristics for Bluetooth panel toggling.
//! Encapsulates timing logic and debouncing so main loop stays minimal.
use crate::bluetooth;
use crate::log_dbg;
use std::time::Instant;
use crate::config::{CLICK_DEBOUNCE, TOGGLE_MIN_HIDE};

// AppState is not clonable due to trait object; implement Clone later if needed.
pub struct AppState {
    last_click_time: Instant,
    last_launch_time: Instant,
    last_user_thought_open: bool,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("last_click_time", &self.last_click_time.elapsed())
            .field("last_launch_time", &self.last_launch_time.elapsed())
            .field("last_user_thought_open", &self.last_user_thought_open)
            .finish()
    }
}

impl AppState {
    pub fn new() -> Self {
        let now = Instant::now();
        // Initialize last_click_time sufficiently in the past so the very first user action is never debounced.
    Self { last_click_time: now - CLICK_DEBOUNCE, last_launch_time: now, last_user_thought_open: false }
    }

    fn log_state(&self, _prefix: &str, _since_last_click: std::time::Duration, _since_launch: std::time::Duration) {
        log_dbg!(
            "state: thought_open={} (debounce_ms={})",
            self.last_user_thought_open, CLICK_DEBOUNCE.as_millis()
        );
    }

    fn toggle_bluetooth_ui_internal(&mut self) {
        let now = Instant::now();
        let since_last_click = now.duration_since(self.last_click_time);
        let since_launch = now.duration_since(self.last_launch_time);
        self.log_state("toggle", since_last_click, since_launch);
        if since_last_click < CLICK_DEBOUNCE {
            log_dbg!("toggle: debounced event ({} ms < {} ms)", since_last_click.as_millis(), CLICK_DEBOUNCE.as_millis());
            return;
        }
        self.last_click_time = now;
        if !self.last_user_thought_open {
            // We believe it's closed: launch to open.
            bluetooth::show_bluetooth_ui();
            self.last_launch_time = now;
            self.last_user_thought_open = true;
            log_dbg!("toggle: panel launch attempt (state: now assumed open)");
        } else {
            // We believe it's open.
            if since_launch > TOGGLE_MIN_HIDE {
                // After minimum visible window -> treat as hide toggle.
                log_dbg!("toggle: hiding heuristic -> relaunch to close ({} ms > {} ms)", since_launch.as_millis(), TOGGLE_MIN_HIDE.as_millis());
                bluetooth::show_bluetooth_ui();
                self.last_user_thought_open = false;
            } else {
                // Too soon to hide; ignore to prevent flicker (double toggle closes immediately).
                log_dbg!(
                    "toggle: ignored while assumed open ({} ms < {} ms) to prevent immediate close/flicker",
                    since_launch.as_millis(), TOGGLE_MIN_HIDE.as_millis()
                );
            }
        }
    }

    pub fn on_tray_left_click(&mut self) { self.toggle_bluetooth_ui_internal(); }
    pub fn on_win_k(&mut self) { self.toggle_bluetooth_ui_internal(); }
}
