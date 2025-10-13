# Changelog

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
