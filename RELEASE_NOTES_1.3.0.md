# Release Notes - Version 1.3.0

**Release Date**: October 17, 2025

## 🎵 New Feature: Audio Device Selection

This release adds comprehensive audio device management directly from the tray icon!

### What's New

- **Audio Devices Submenu**: Right-click the tray icon to see a new "Audio Devices" submenu
- **Quick Device Switching**: Click any audio output device to set it as your default
- **Visual Indicators**: Checkmark (✓) shows which device is currently active
- **Auto-Detection**: Devices automatically appear/disappear when connected/disconnected
  - Works great with Bluetooth headphones, USB audio devices, etc.
- **Event-Driven Updates**: Uses Windows Core Audio callbacks - no CPU waste from polling!

### How to Use

1. Right-click the tray icon
2. Hover over "Audio Devices" 
3. Click any device to make it the default audio output
4. The checkmark moves to show the new active device

### Technical Details

- Sets default for all Windows audio roles (Console, Multimedia, Communications)
- Implements Windows Core Audio `IMMNotificationClient` interface
- Proper COM reference counting and vtable implementation
- Zero warnings with strict Rust clippy lints
- Idiomatic Rust code following best practices

### File Information

- **Filename**: `restore-wink-bt.exe`
- **Version**: 1.3.0
- **Size**: 329,728 bytes (322 KB)
- **SHA256**: `DD9FD6CEFADEB6DC193511E832D291932A269352A9E0299FA05C689ADBDD3FD4`

### Breaking Changes

None - fully backward compatible with previous versions.

### Bug Fixes

- **Win+K Cast Access**: Fixed second Win+K behavior - now correctly passes through to Windows for native Cast/Project flyout (when Win key is held and K is pressed twice within 1.2 seconds)

### Known Issues

None

---

## Previous Features (Still Included)

- ✅ Win+K keyboard hook for quick Bluetooth panel access
- ✅ Tray icon for manual Bluetooth panel launching
- ✅ Cast access on second Win+K press (within 1.2 seconds)
- ✅ Single-instance enforcement
- ✅ Auto-recovery from Explorer crashes
- ✅ About dialog with version info
- ✅ Minimal CPU usage and memory footprint

---

## Installation

Simply run `restore-wink-bt.exe` - no installation required. The executable is fully self-contained.

## Upgrade Instructions

Replace your existing `restore-wink-bt.exe` with the new version. The app will automatically use the new audio device features.

---

**Full changelog**: See [CHANGELOG.md](CHANGELOG.md)
