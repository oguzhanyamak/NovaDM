# Download Details Panel Implementation

## Overview

This document explains the architecture and design decisions for the Download Details panel in NovaDM, designed to match the familiar IDM/FDM user experience.

## Why the Panel is Read-Only

### Separation of Concerns

The details panel is read-only for several important reasons:

1. **Single Responsibility**: The panel's job is to display information, not to modify it. All actions (pause, resume, retry, delete) are handled by the download engine and history service.

2. **Data Integrity**: Download metadata is managed by the download engine. Allowing direct edits could create inconsistencies between the UI and the actual download state.

3. **Audit Trail**: The timeline and status information represent actual events. Making these read-only preserves the integrity of the download history.

4. **Simplicity**: Users can see all relevant information at a glance without navigating to different screens or modals.

## Why Actions Remain Outside Business Logic

### UI vs Service Separation

Actions in the panel are UI triggers that call existing services:

1. **No Duplication**: The download engine already has pause, resume, cancel, and retry functionality. The panel simply triggers these existing operations.

2. **Consistency**: All actions go through the same code paths, ensuring consistent behavior whether triggered from the list or the details panel.

3. **Testability**: Business logic is tested in the service layer, not in UI components.

4. **Future Extensibility**: When we add Tauri commands for these actions, the panel will automatically work without changes.

## Why Details are Grouped into Sections

### Information Architecture

The details are organized into logical sections for better usability, following IDM/FDM patterns:

1. **General**: Core information that users need most often (filename, status, URL, file size, progress)

2. **Performance**: Real-time metrics that help users understand download speed and progress

3. **Timeline**: Historical events that show the download's lifecycle

4. **Technical**: HTTP and connection details useful for debugging

This grouping:
- Reduces cognitive load by organizing related information
- Allows users to quickly find what they need
- Provides a clear structure for future additions (SHA256, MD5, per-worker stats)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        DetailsPanel                           │
│  (Main container, handles collapsed state)                     │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│ DetailsSection│     │ DetailsSection│     │ DetailsSection│
│   (General)   │     │ (Performance) │     │  (Timeline)   │
└───────────────┘     └───────────────┘     └───────────────┘
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│  DetailsRow   │     │  DetailsRow   │     │  DetailsRow   │
│ (label/value) │     │ (label/value) │     │ (label/value) │
└───────────────┘     └───────────────┘     └───────────────┘
```

### Store Architecture

```
useDetailsStore (Zustand)
├── selection: { downloadId, source }
├── details: DownloadDetails | null
├── isCollapsed: boolean
└── Actions: selectDownload, clearSelection, setCollapsed
```

### Service Architecture

```
detailsService
├── buildFromDownload(download, bandwidthLimit) → DownloadDetails
├── buildFromHistory(entry, bandwidthLimit) → DownloadDetails
├── formatBytes(bytes) → string
├── formatSpeed(bytesPerSecond) → string
└── formatDuration(seconds) → string
```

## UI Design (IDM/FDM Style)

The panel follows IDM/FDM conventions:

1. **Two-column property grid**: Labels on left, values on right for efficient scanning
2. **Properties header**: Simple "Properties" title with collapse button
3. **Bottom action bar**: Action buttons at the bottom with secondary background
4. **Collapsed sidebar**: When collapsed, shows a thin sidebar with expand button
5. **Section headers**: Clear section titles with consistent spacing

## Future Compatibility

The architecture supports future additions without redesign:

1. **SHA256/MD5**: Add to `DownloadDetails` interface and display in General section

2. **Per-worker statistics**: Add to `PerformanceMetrics` and display in Performance section

3. **Transfer graph**: Add a new `DetailsSection` for visualization

4. **Piece map**: Add a new `DetailsSection` for showing download progress by chunk

5. **Resizable width**: The panel uses CSS width that can be made adjustable with a resize handle

## Performance Optimizations

1. **useMemo**: Details are only rebuilt when selection or underlying data changes

2. **Selective re-renders**: The panel only re-renders when its specific data changes

3. **Collapsed state**: Panel can be collapsed to save space and reduce rendering

4. **No unnecessary updates**: The `updateSetting` function in settings store doesn't set loading state to prevent scroll jumping