# Pause Implementation

## Overview

This sprint introduces a true pause state inside the download engine.

## What Changed

### Backend (Rust)

1. **DownloadHandle** - Extended in `core/mod.rs`:
   - Added `pause_token: CancellationToken` field
   - Uses same cancellation architecture as cancel

2. **DownloadError** - Extended in `errors.rs`:
   - Added `Paused` variant

3. **DownloadState** - Extended in `scheduler.rs`:
   - Added `Paused` state

4. **DownloadManager** - Updated in `manager.rs`:
   - Added `pause_download(id)` method
   - Checks `pause_token` in download loop
   - On pause: flush, update metadata, emit `download://paused`
   - On pause: keep .part file and metadata

### Frontend

1. **EventService** - Updated in `event.ts`:
   - Added `DownloadPausedData` interface
   - Added `registerPausedListener` method
   - Added `pausedUnlistenPromise` and `pausedCallbacks`

2. **Store** - Updated in `downloads.ts`:
   - Added `markAsPaused` method

## Difference Between Pause and Cancel

| Aspect | Pause | Cancel |
|--------|-------|--------|
| .part file | Kept | Deleted |
| Metadata | Kept | Deleted |
| Can resume | Yes (future) | No |
| Scheduler | Starts next | Starts next |
| Event | `download://paused` | `download://cancelled` |

## Why Metadata Survives Pause

- Resume will need the download position
- Resume will need the URL and filename
- Resume will need the partial file path
- No data loss on network interruption

## Why .part Survives Pause

- Resume can continue from where it left off
- No need to re-download already received data
- File is not visible to user (extension is .part)

## How Scheduler Behaves

When a download is paused:
1. Download is removed from `active_downloads`
2. `active_count` decreases
3. `can_start` returns true for next queued download
4. Next download starts immediately

## State Transitions

```
Downloading
    ↓
Paused
```

Only one transition: downloading → paused.

## Error Handling

- `pause_download("unknown")` → `DownloadError::NotFound`
- `pause_download("completed")` → `DownloadError::InvalidState`
- `pause_download("already-paused")` → `DownloadError::InvalidState`

## Tests

- `test_pause_nonexistent` - Pausing unknown download
- `test_max_concurrent_limit` - Scheduler behavior (existing)
- `test_backward_compatibility` - Metadata preserved (existing)