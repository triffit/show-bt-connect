ShowBTConnect
=============

Ultra-light Windows tray helper that restores the fast Windows 10 Win+K Bluetooth Devices connect panel (first chord) while preserving Cast on the second chord.

Status: Active minimalist utility. Current version: 1.4.0. Recent updates: executable rename, UTF-16 module refactor, context menu improvements.

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
- **Audio Devices submenu** (quick switching between audio output devices)

What It Does
------------
1. Tries `ms-actioncenter:controlcenter/bluetooth` (Action Center URI)
2. Falls back to `ms-settings:bluetooth` if the first fails

Key Features
------------
- Single self‑contained executable (`ShowBTConnect.exe`)
- Embedded icon & version metadata (`resources.rc` + build script)
- Build‑time ICO -> RGBA extraction (no image decoding at runtime)
- Size optimized (LTO Thin, opt-level=z, stripped): ~320–330 KB (with audio features)
- Single-instance enforcement (named mutex; silent exit on second launch)
- Debounce heuristic to avoid rapid relaunch storms
- Accurate Win+K emulation: first chord Bluetooth, second chord Cast (pass‑through window ~1.2s)
- **Audio Device Selection**: Tray menu for quick switching between audio outputs
  - Event-driven updates (Windows Core Audio `IMMNotificationClient` callbacks)
  - Automatic detection of connected/disconnected devices (Bluetooth, USB, etc.)
  - Visual checkmark indicator for current default device
  - Sets default for all roles (Console, Multimedia, Communications)
- About dialog (tray menu) with version pulled from build metadata
- Graceful keyboard hook shutdown (unhook on Exit)
- RAII keyboard hook guard (auto-unhook on drop – no manual shutdown path needed)
- Failure-only logging always active; success logging opt-in via `--features verbose-log`
- `#![deny(warnings)]` enforced for consistently clean builds
- Zero clippy warnings with idiomatic Rust code

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
target/x86_64-pc-windows-msvc/release/ShowBTConnect.exe
```
------------------------
Enable extra diagnostic output (including successful URI launches):
```
cargo build --release --features verbose-log
```
Without the feature, only failures (ShellExecute codes <= 32, mutex failure) log.

Runtime Usage
-------------
1. Launch the exe (console hidden in non-debug builds).
2. Press Win+K or left-click tray icon to show Bluetooth devices.
3. Hold Win and press K again quickly for Cast (pass-through).
4. Right-click tray icon for:
   - **Audio Devices**: Select audio output device (checkmark shows current default)
   - **About**: Version and credits
   - **Exit**: Quit application

Distribution
------------
Copy just the EXE (`ShowBTConnect.exe`). Everything required (icon, metadata) is embedded.

Verifying Integrity (PowerShell)
--------------------------------
```
Get-FileHash dist\ShowBTConnect.exe -Algorithm SHA256
```
(Record the hash before publishing.)

Single Instance Behavior
------------------------
A named global mutex `Global\\ShowBTConnectMutex` prevents multiple copies. Second launches exit silently.

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
 - Windows `windows-sys` crate migration to trim `winapi` surface.
 - CI automation (GitHub Actions: build + release artifact + hash publish).
 - Integration tests for mutex single-instance behavior.
- Alternative fallback if Microsoft changes the URI schemes.
- Optional balloon / tooltip feedback on launch failure detection (placeholder logic prepared).

Development Overview
--------------------
Minimal dependency set: `winit`, `tray-icon`, `windows-sys` (curated Win32 feature list), `once_cell` plus build-time `ico` & `embed-resource`.

Wide UTF-16 Helpers
-------------------
The module `src/utf16_strings.rs` provides two tiny helpers:

- `encode_utf16(s: &str) -> Vec<u16>`: UTF-16 encode without a terminating null.
- `encode_utf16_null(s: &str) -> Vec<u16>`: UTF-16 encode and append a trailing `\0` (for most Win32 APIs).

These replace repeated `OsStr::new(...).encode_wide().chain(once(0))` patterns across mutex creation, window class registration, message box titles, ShellExecuteW calls, etc., improving readability and reducing minor copy/paste risk. No external crate is pulled in for this to keep the binary minimal.

License
-------
MIT License (see `LICENSE`).

Release Checklist
-----------------
1. Update `CHANGELOG.md` with version + notable changes.
2. Set `APP_VERSION` via build script env (or tag) and build release:
	- `cargo clean && cargo build --release`
3. Verify size & hash:
	- Check file size (<330 KB expected)
	- `Get-FileHash target/x86_64-pc-windows-msvc/release/ShowBTConnect.exe -Algorithm SHA256`
4. Smoke test:
	- Run `ShowBTConnect.exe --version` (prints version & exits)
	- Launch normally; test first and second Win+K behavior; tray About / Exit
	- Restart Explorer (taskkill /IM explorer.exe /F; start explorer) and confirm tray icon auto-reappears.
5. Publish artifact + hash.
6. (Optional) Sign binary if code signing is available.

Binary Size & Stripping
-----------------------
The executable is already built with size-focused settings (see `.cargo/config.toml`):

- `opt-level = "z"`, `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`
- Linker flags: `/OPT:REF /OPT:ICF` (dead code & identical COMDAT folding)

Stripping Options:

1. Manual (stable, explicit – recommended for release pipeline transparency):
   PowerShell:
   ```powershell
   cargo build --release
   # Use llvm-strip (ships with Rust toolchain) – safer than full symbol removal tools.
   & (Join-Path (Split-Path (Get-Command rustc).Source) "..\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-strip.exe") `
	   --strip-debug `
	   target\x86_64-pc-windows-msvc\release\ShowBTConnect.exe
   ```
   (You can also copy `llvm-strip.exe` path into an environment variable for reuse.)

2. Automatic (stable): uncomment `strip = true` in `[profile.release]` inside `.cargo/config.toml`. Rust will invoke the platform tool; explicit control is lost, so manual verification may be harder.

3. Nightly `-Z strip` (historical): superseded by stable `strip = true`; not needed now.

Measuring Size:
```powershell
Get-Item target\x86_64-pc-windows-msvc\release\ShowBTConnect.exe | Select-Object Length
```

Trade-offs:
- `lto = "fat"` can shave a few more KB at the cost of longer build times.
- Removing even minimal logging (feature gate) could reduce a few more bytes; current impact is nominal.
- Code signing will add bytes (signature blob); measure after signing for distribution numbers.
