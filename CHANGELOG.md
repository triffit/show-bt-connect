# Changelog
## [1.2.6] - 2025-10-13
### Added
- Tray icon resilience restored: listens for `TaskbarCreated` and automatically recreates the tray icon.
### Internal
- Added GDI feature to windows-sys for WNDCLASSW.
- Uses hidden window + broadcast message; minimal overhead.

## [1.2.5] - 2025-10-13
### Added
- Tray icon resilience: auto recreation after Explorer/taskbar restarts (listens for `TaskbarCreated`).
- Completed migration to `windows-sys` (removed legacy `winapi`).
### Changed
- Refactored tray creation into reusable function.
### Notes
- Toggle behavior unchanged; small binary size improvements possible from binding pruning.

## [1.2.4] - 2025-10-13
### Added
- RAII keyboard hook guard (automatic unhook on drop).
- README with usage, feature flag and build details.

### Changed
- Success launch logging now suppressed in minimal release unless `verbose-log` feature enabled.
- Enforced `#![deny(warnings)]` for strict builds.

### Internal
- Failure logging always retained (ShellExecute failure codes, mutex failure).

## [1.2.3] - 2025-10-13
### Fixed
- First-click tray/Win+K Bluetooth panel flashing closed immediately (introduced during modular refactor). Adjusted toggle heuristic to ignore premature hide attempts.

### Added
- Diagnostic multi-strategy Bluetooth launch (action center, settings, explorer indirections) with verbose logging (enable via `--features verbose-log`).

### Internal
- Introduced `verbose-log` Cargo feature for release logging without debug build.
- Additional per-event logging (tray vs Win+K) for troubleshooting.


All notable changes to this project will be documented here.

## [1.2.2] - 2025-10-13
### Added
- (none yet)

### Changed
- Version bump only (no functional changes since 1.1.6).

### Removed
- (none)

### Internal
- Updated crate and resource versions to 1.2.2 / 1.2.2.0.
- Briefly experimented with a raw Win32 message loop (saved as `main_win32_experimental.rs`) but reverted to the winit event loop due to functional issues and modest (~40KB) size reduction.
- Refactor: extracted `app_state`, `single_instance`, and `log` modules; removed per-launch thread spawns (ShellExecuteW already non-blocking); simplified timing logic using `Duration` comparisons.

## [1.1.6] - 2025-10-13
### Added
- About dialog displaying version and restored Win+K behavior.
- Graceful keyboard hook shutdown (unhooks on Exit).
- Build-time version injection for dynamic tooltip.

### Changed
- Branding to **Restore Win+K: Bluetooth Devices Panel** (binary renamed to `restore-wink-bt.exe`).
- Tooltip and About dialog now show new branding and version.
- Win+K handling refined: first chord launches Bluetooth devices panel; second (while holding Win) passes through to Cast.
- Mutex name simplified to `Global\\RestoreWinKBluetoothDevicesPanelMutex` (legacy compatibility removed).

### Removed
- Legacy single-instance compatibility with old mutex name.
- Unused configuration and error modules (previous versions).

### Internal
- Resource file (`resources.rc`) updated (description, product/internal/original filename, version 1.1.6.0).

## [1.1.5] - 2025- earlier
- Prior features: initial tray integration, single-instance enforcement, size optimizations, icon embedding.

---
Semantic versioning is followed informally; patch increments reflect incremental feature polish.

