Restore Win+K: Bluetooth Devices Panel (Tray)
============================================

Ultra-light Windows tray helper that restores the fast Windows 10 Win+K Bluetooth Devices connect panel (first chord) while preserving Cast on the second chord. (Acronym: WINKBT)

Status: Active minimalist utility. Recent updates: About dialog, graceful hook shutdown, refined Win+K pass‑through, branding rename.

Branding Update
---------------
Formerly “Show Bluetooth Manager”. Renamed to “Restore Win+K: Bluetooth Devices Panel” to emphasize restored behavior: first Win+K → Bluetooth devices panel, second (while holding Win) → Cast. Existing mutex / prior binary name may appear in older releases.

Background / Restored Behavior
------------------------------
In Windows 10, pressing Win+K opened the Bluetooth *Connect* devices flyout (quick panel to connect headphones, speakers, etc.).
In Windows 11, Microsoft reassigned Win+K to open the *Cast* (wireless display) interface, making the fast Bluetooth connect flow less direct.

This tool restores the Windows 10 style behavior:
- First Win+K press: shows the Bluetooth Connect devices UI (via Action Center / Settings URI sequence) — the app swallows this key chord.
- Still holding Win, press K again within ~1.2s: the system Cast interface is allowed through (second chord is not swallowed), preserving Cast access.

Effectively you regain the quick Bluetooth panel on the first chord, yet still have access to Cast by a quick follow-on K while holding Win.

Current Triggers
----------------
- Tray icon left click (opens / re-opens Bluetooth panel heuristic)
- Win+K low-level keyboard hook (captured via WH_KEYBOARD_LL, original system panel suppressed)
- About menu item (shows version / credits)

What It Does
------------
1. Tries `ms-actioncenter:controlcenter/bluetooth` (Action Center URI)
2. Falls back to `ms-settings:bluetooth` if the first fails

Key Features
------------
- Single self‑contained executable (`restore-wink-bt.exe`)
- Embedded icon & version metadata (`resources.rc` + build script)
- Build‑time ICO -> RGBA extraction (no image decoding at runtime)
- Size optimized (LTO Thin, opt-level=z, stripped): ~280–290 KB
- Single-instance enforcement (named mutex; silent exit on second launch)
- Debounce heuristic to avoid rapid relaunch storms
- Accurate Win+K emulation: first chord Bluetooth, second chord Cast (pass‑through window ~1.2s)
- About dialog (tray menu) with version pulled from build metadata
- Graceful keyboard hook shutdown (unhook on Exit)

Removed / Simplified (Historical)
---------------------------------
- No JSON config or autostart toggling (registry logic removed)
- No fallback hotkey chain; replaced by a low-level Win+K hook only
- No dynamic icon decoding at runtime (was image crate, now removed)
- Removed custom error enum & config modules (size / complexity reduction)

Build
-----
```
cargo build --release
```
Artifacts (with explicit target in `.cargo/config.toml`):
```
target/x86_64-pc-windows-msvc/release/restore-wink-bt.exe
```

Distribution
------------
Copy just the EXE (`restore-wink-bt.exe`). Everything required (icon, metadata) is embedded.

Verifying Integrity (PowerShell)
--------------------------------
```
Get-FileHash dist\restore-wink-bt.exe -Algorithm SHA256
```
(Record the hash before publishing.)

Single Instance Behavior
------------------------
A named global mutex `Global\\RestoreWinKBluetoothDevicesPanelMutex` prevents multiple copies. Second launches exit silently.

Hotkey Hook Notes
-----------------
- Uses a low-level keyboard hook to intercept Win+K.
- Consumes the *first* Win+K sequence to show the Bluetooth Connect panel.
- Keeps internal timing window (~1.2s) while Win is held; a second K press inside that window is passed through so Windows' native Cast interface appears. Releasing Win resets the window.

Future Ideas
------------
- Optional bring-to-front / focus behavior if a second launch attempted.
- Add code signing & release automation (GitHub Actions workflow + hash announce).
- Optional logging toggle for diagnostics.
- Alternative fallback if Microsoft changes the URI schemes.
- Optional balloon / tooltip feedback on launch failure detection (placeholder logic prepared).

Development Overview
--------------------
Minimal dependency set: `winit`, `tray-icon`, `winapi`, `once_cell` plus build-time `ico` & `embed-resource`.

License
-------
MIT License (see `LICENSE`).
