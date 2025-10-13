Show Bluetooth Manager (Tray)
=============================

Ultra-light Windows tray helper to launch the Bluetooth connect UI instantly.

Background / Restored Behavior
------------------------------
In Windows 10, pressing Win+K opened the Bluetooth *Connect* devices flyout (quick panel to connect headphones, speakers, etc.).
In Windows 11, Microsoft reassigned Win+K to open the *Cast* (wireless display) interface, making the fast Bluetooth connect flow less direct.

This tool restores the Windows 10 style behavior:
- First Win+K press: shows the Bluetooth Connect devices UI (via Action Center / Settings URI sequence).
- While still holding the Win key, pressing K again (i.e. a second K keydown with Win held) will allow the system Cast devices UI to appear (the app only swallows the first Win+K to prioritize Bluetooth).

Effectively you regain the quick Bluetooth panel on the first chord, yet still have access to Cast by a quick follow-on K while holding Win.

Current Triggers
----------------
- Tray icon left click (opens / re-opens Bluetooth panel heuristic)
- Win+K low-level keyboard hook (captured via WH_KEYBOARD_LL, original system panel suppressed)

What It Does
------------
1. Tries `ms-actioncenter:controlcenter/bluetooth` (Action Center URI)
2. Falls back to `ms-settings:bluetooth` if the first fails

Key Features
------------
- Single self‑contained executable (`show-bluetooth-manager.exe`)
- Embedded icon & version metadata (`resources.rc` + build script)
- Build‑time ICO -> RGBA extraction (no image decoding at runtime)
- Size optimized (LTO Thin, opt-level=z, stripped): ~280–290 KB
- Single-instance enforcement (named mutex; silent exit on second launch)
- Debounce heuristic to avoid rapid relaunch storms

Removed / Simplified (Historical)
---------------------------------
- No JSON config or autostart toggling (registry logic removed)
- No fallback hotkey chain; replaced by a low-level Win+K hook only
- No dynamic icon decoding at runtime (was image crate, now removed)

Build
-----
```
cargo build --release
```
Artifacts (with explicit target in `.cargo/config.toml`):
```
target/x86_64-pc-windows-msvc/release/show-bluetooth-manager.exe
```

Distribution
------------
Copy just the EXE (optionally place in a `dist/` folder). Everything required (icon, metadata) is embedded.

Verifying Integrity (PowerShell)
--------------------------------
```
Get-FileHash dist\show-bluetooth-manager.exe -Algorithm SHA256
```
(Record the hash before publishing.)

Single Instance Behavior
------------------------
A named global mutex `Global\\ShowBluetoothManagerMutex` prevents multiple copies. Second launches exit silently.

Hotkey Hook Notes
-----------------
- Uses a low-level keyboard hook to intercept Win+K.
- Consumes the *first* Win+K sequence to show the Bluetooth Connect panel.
- If you keep holding Win and press K again, Windows' native Cast interface can still appear (second sequence not swallowed), so both workflows remain accessible.

Future Ideas
------------
- Optional bring-to-front / focus behavior if a second launch attempted.
- Add code signing & release automation (GitHub Actions workflow + hash announce).
- Optional logging toggle for diagnostics.
- Alternative fallback if Microsoft changes the URI schemes.

Development Overview
--------------------
Minimal dependency set: `winit`, `tray-icon`, `winapi`, `once_cell` plus build-time `ico` & `embed-resource`.

License
-------
MIT License (see `LICENSE`).
