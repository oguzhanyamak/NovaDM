
# Settings Module Implementation

## Overview

This document explains the architecture and design decisions for the Settings module in NovaDM.

## Why Settings are Isolated

### Single Responsibility Principle

Settings are isolated from other modules to ensure:

1. **Single Source of Truth**: All configuration is stored in one place, making it easy to manage and backup.

2. **No Circular Dependencies**: Settings don't depend on download engine, and download engine doesn't depend on settings. This keeps the architecture clean.

3. **Easy Testing**: Settings can be tested independently without mocking download operations.

4. **Clear API Surface**: The settings API is minimal and focused, making it easy to understand and use.

### Storage Architecture

```
Settings: ~/.config/novadm/settings.json (or platform equivalent)
```

Settings are stored in a single JSON file for:
- Easy backup/export
- Atomic updates
- Simple file-based persistence

## Why Placeholders Exist for Future Features

Some settings are implemented as placeholders because:

1. **Open on Startup**: Requires OS-specific integration (Windows registry, macOS login items, Linux autostart). This will be implemented in a future sprint.

2. **Check for Updates**: Requires integration with a release API (GitHub, custom server). This will be implemented when the app has a release process.

3. **Enable Notifications**: Requires notification system integration. This will be implemented when the notification system is ready.

4. **Enable Browser Integration**: Requires browser extension or protocol handler registration. This will be implemented when the browser integration feature is prioritized.

These placeholders allow:
- UI design to be completed early
- API to be stable for future implementation
- Users to see what's coming

## Settings Structure

### Download Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `download_path` | string | `~/Downloads/NovaDM` | Default folder for downloads |
| `max_concurrent_downloads` | number | 3 | Maximum parallel downloads |
| `bandwidth_limit_kb` | number | 0 | Global bandwidth limit (KB/s), 0 = unlimited |
| `auto_resume` | boolean | true | Automatically resume interrupted downloads |
| `auto_retry` | boolean | true | Automatically retry failed downloads |
| `max_retry_attempts` | number | 3 | Maximum retry attempts for failed downloads |

### Appearance Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `theme` | 'system' \| 'dark' \| 'light' | 'system' | UI theme preference |

### General Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `open_on_startup` | boolean | false | Open NovaDM on system startup (placeholder) |
| `auto_check_updates` | boolean | true | Check for updates automatically (placeholder) |
| `enable_notifications` | boolean | true | Enable desktop notifications (placeholder) |
| `enable_browser_integration` | boolean | false | Enable browser integration (placeholder) |

## Validation

Settings are validated on load and save:

1. **Download path**: Must be a valid path (non-existent paths are allowed, they will be created)
2. **Max concurrent downloads**: Must be > 0
3. **Max retry attempts**: Must be > 0 when auto-retry is enabled
4. **Bandwidth limit**: Must be >= 0

Invalid settings fall back to defaults.

## Component Architecture

### SettingsCard
Displays a settings section with:
- Title
- Description
- Content area

### SettingsInput
Input field for:
- Text values
- Number values
- Path values

### SettingsSection
Logical grouping of settings (used in page layout)

## Import / Export

Settings can be exported to JSON and imported from JSON:
- Export creates a downloadable JSON file
- Import validates and applies settings
- Reset returns to defaults

## Performance

- Settings are loaded once on app start
- Changes are saved immediately (no restart required)
- Zustand store provides efficient state management
- Components only re-render when their settings change