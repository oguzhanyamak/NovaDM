# Crash Recovery Implementation

## Overview

This sprint implements crash recovery and session restoration for NovaDM.

## What Changed

### Backend (Rust)

1. **RecoveryCandidate** - New struct in `recovery.rs`:
   - download_id
   - filename
   - url
   - partial_path
   - downloaded_bytes
   - total_bytes
   - resume_supported
   - created_at

2. **RecoveryService** - New service in `recovery.rs`:
   - `scan()` - Scans metadata directory
   - Validates metadata and partial files
   - Returns valid recovery candidates

3. **DownloadState** - Extended in `scheduler.rs`:
   - Added `Recovered` state

4. **MetadataRepository** - Extended in `metadata.rs`:
   - `load_from_path()` - Load from specific path
   - `delete_from_path()` - Delete from specific path
   - `get_base_path()` - Get base path

### Frontend

1. **EventService** - Updated in `event.ts`:
   - Added `DownloadRecoveredData` interface
   - Added `registerRecoveredListener` method

2. **Store** - Updated in `downloads.ts`:
   - Added `markAsRecovered` method

3. **DownloadStatusBadge** - Updated in `DownloadStatusBadge.tsx`:
   - Added 'recovered' status display

4. **DownloadCard** - Updated in `DownloadCard.tsx`:
   - Added 'recovered' status type

## Crash Recovery Flow

```
Application Startup
    ↓
RecoveryService.scan()
    ↓
For each metadata file:
    - Load metadata
    - Validate partial file exists
    - If valid: create RecoveryCandidate
    - If invalid: delete metadata
    ↓
Return candidates to frontend
    ↓
Frontend populates download list with status: Recovered
```

## Why Recovery is Opt-In

- User decides when to resume
- No automatic network requests on startup
- User can review recovered downloads
- User can cancel recovered downloads
- Prevents unwanted bandwidth usage

## Why Startup Must Never Fail

- Recovery errors are logged, not thrown
- Invalid metadata is cleaned up silently
- Missing partial files are cleaned up silently
- Application starts even with corrupted data
- User can still use the application

## Error Handling

| Error | Action |
|-------|--------|
| Broken metadata | Log warning, skip |
| Missing .part | Delete metadata, skip |
| Corrupted JSON | Log warning, skip |
| Directory not found | Return empty list |

## State Transitions

```
Recovered
    ↓
Downloading (future resume)
    ↓
Completed

Recovered
    ↓
Cancelled
```

## Tests

- `test_empty_directory` - Empty metadata directory
- `test_metadata_discovered` - Valid metadata found
- `test_missing_part` - Missing partial file
- `test_broken_metadata` - Corrupted JSON
- `test_multiple_recoveries` - Multiple candidates