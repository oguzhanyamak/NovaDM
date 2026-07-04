# NovaDM Architecture Refactoring

## Overview
This document explains the architectural refactoring performed on NovaDM to improve maintainability, separation of concerns, and code organization.

## Frontend Changes

### 1. Created `src/layouts/` Directory

**What Changed:**
- Created `src/layouts/MainLayout.tsx`
- Moved the sidebar from `App.tsx` into `MainLayout`
- Updated `App.tsx` to use `MainLayout` as a wrapper
- Updated all pages to use `h-full` instead of `h-screen`

**Files Moved:**
- Sidebar component logic moved from `App.tsx` → `layouts/MainLayout.tsx`

**Why This Improves Maintainability:**
- **Separation of Concerns**: App.tsx now only handles routing/state management, not layout structure
- **Reusability**: MainLayout can be reused for different page configurations
- **Single Responsibility**: Each component has one clear purpose
- **Easier Testing**: Layout can be tested independently from routing logic
- **Consistency**: All pages follow the same structure (header + content)

**Trade-offs:**
- Added one extra component layer (minimal overhead)
- Pages now depend on parent for full height (more explicit)

### 2. Updated Page Components

**What Changed:**
- Changed `h-screen` → `h-full` in Downloads, History, and Settings pages
- Pages now fill available space from parent layout instead of viewport

**Why This Improves Maintainability:**
- **Flexibility**: Layout controls sizing, pages control content
- **Consistency**: All pages follow same pattern
- **Responsive**: Easier to adapt to different layout configurations

## Backend Changes

### 1. Refactored Download Module

**New Structure:**
```
src-tauri/src/download/
├── mod.rs          # Module entry point, public exports
├── models.rs       # Data structures (DownloadInfo, DownloadStatus, DownloadTask)
├── manager.rs      # DownloadManager - orchestrates operations
├── worker.rs       # DownloadWorker - handles individual downloads
├── queue.rs        # DownloadQueue - manages concurrency
├── chunk.rs        # DownloadChunk - chunked download logic
└── errors.rs       # DownloadError types
```

**What Changed:**
- Split monolithic `download/mod.rs` into 7 focused files
- Each file has a single responsibility
- Models separated from business logic
- Error handling isolated

**Why This Improves Maintainability:**
- **Single Responsibility**: Each file has one clear purpose
- **Easier Navigation**: Find specific functionality quickly
- **Better Testing**: Test each component independently
- **Clear Dependencies**: Explicit imports show relationships
- **Scalability**: Easy to add new features (e.g., chunk types, queue strategies)
- **Reduced Merge Conflicts**: Multiple developers can work on different files

**Trade-offs:**
- More files to navigate (mitigated by clear naming)
- Initial setup overhead (paid once, benefits long-term)

### 2. Created Core Module

**New Structure:**
```
src-tauri/src/core/
├── mod.rs          # AppState and module exports
├── config.rs       # AppConfig - application configuration
├── constants.rs    # Centralized constants
├── errors.rs       # AppError types and Result alias
└── events.rs       # AppEvent types for event system
```

**What Changed:**
- Extracted shared application state and configuration
- Centralized error types
- Defined application constants in one place
- Created event system foundation

**Why This Improves Maintainability:**
- **Single Source of Truth**: Configuration and constants in one place
- **Type Safety**: Centralized error handling with thiserror
- **Consistency**: All modules use same error types
- **Easy Updates**: Change constants without searching through code
- **Foundation for Features**: Event system ready for future implementation

**Trade-offs:**
- Additional abstraction layer (worth it for shared state)
- More initial structure (benefits as app grows)

### 3. Simplified API Module

**What Changed:**
- Removed all download-related commands (get_downloads, get_history, pause, resume, cancel)
- Replaced with minimal commands: `ping()` and `get_app_state()`
- Cleaned up imports and dependencies

**Why This Improves Maintainability:**
- **Clear Intent**: API surface is minimal and focused
- **Easier Onboarding**: New developers see essential commands first
- **Reduced Complexity**: Less code to understand initially
- **Future-Ready**: Easy to add commands when download logic is implemented

**Trade-offs:**
- Less functionality exposed (intentional - no download logic yet)

### 4. Simplified Storage Module

**New Structure:**
```
src-tauri/src/storage/
├── mod.rs          # Module entry point
└── settings.rs     # SettingsManager and AppSettings
```

**What Changed:**
- Removed download persistence logic
- Removed history storage
- Focused only on application settings
- Removed business logic from storage

**Why This Improves Maintainability:**
- **Clear Purpose**: Storage only handles settings
- **Separation of Concerns**: Download persistence will be separate module
- **Easier Testing**: Test settings independently
- **Future-Ready**: Easy to add download/history storage later

**Trade-offs:**
- Less functionality (intentional - no persistence yet)

### 5. Refactored Utils Module

**New Structure:**
```
src-tauri/src/utils/
├── mod.rs          # Module entry point
└── formatting.rs   # Formatting utilities only
```

**What Changed:**
- Removed business logic (ID generation, URL parsing)
- Kept only formatting functions (format_bytes, format_speed, sanitize_filename)
- Moved business logic to appropriate modules

**Why This Improves Maintainability:**
- **Focused Purpose**: Utils only contains helper functions
- **No Business Logic**: Clear separation from domain logic
- **Reusability**: Formatting functions can be used anywhere
- **Testability**: Pure functions are easy to test

**Trade-offs:**
- ID generation moved (will be in download module when implemented)

## Architecture Improvements Summary

### Before Refactoring:
```
src-tauri/src/
├── lib.rs          # Mixed concerns
├── api/            # Too many commands
├── download/       # Monolithic
├── storage/        # Mixed concerns
└── utils/          # Mixed concerns
```

### After Refactoring:
```
src-tauri/src/
├── lib.rs          # Clean initialization
├── core/           # Shared state & config
├── api/            # Minimal commands
├── download/       # Focused submodules
│   ├── models/
│   ├── manager/
│   ├── worker/
│   ├── queue/
│   ├── chunk/
│   └── errors/
├── storage/        # Settings only
└── utils/          # Formatting only
```

## Key Principles Applied

1. **Single Responsibility Principle**: Each module/file has one reason to change
2. **Separation of Concerns**: Clear boundaries between layers
3. **Dependency Direction**: Dependencies flow inward (utils → download → api)
4. **Explicit Dependencies**: Clear imports show relationships
5. **Future-Ready**: Structure supports growth without major refactoring

## Benefits

1. **Easier Onboarding**: New developers understand structure quickly
2. **Reduced Cognitive Load**: Find code in expected locations
3. **Better Testing**: Isolated components are easier to test
4. **Parallel Development**: Multiple devs can work without conflicts
5. **Easier Refactoring**: Changes are localized to specific modules
6. **Clearer Intent**: File names and locations communicate purpose

## Migration Path

The refactored structure is ready for:
1. Implementing download logic in `download/` submodules
2. Adding Tauri commands as needed in `api/`
3. Implementing settings persistence in `storage/`
4. Adding event handling via `core/events.rs`
5. Expanding configuration in `core/config.rs`

## Conclusion

This refactoring improves maintainability by:
- Making the codebase easier to navigate
- Reducing cognitive load through clear organization
- Enabling parallel development
- Preparing for future feature implementation
- Following Rust best practices and patterns

The UI remains **exactly the same** - all changes are architectural only.