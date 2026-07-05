# Crash Recovery Implementation

## What Changed

### Backend (Rust)

1. **RecoveryCandidate** - New struct in `recovery.rs`
2. **RecoveryService** - New service in `recovery.rs`
3. **DownloadState** - Added `Recovered` state
4. **MetadataRepository** - Added `load_from_path`, `delete_from_path`, `get_base_path`

### Frontend

1. **EventService** - Added `registerRecoveredListener`
2. **Store** - Added `markAsRecovered`
3. **DownloadStatusBadge** - Added 'recovered' status
4. **DownloadCard** - Added 'recovered' status type

## Verification

✅ `cargo check` - No errors
✅ `cargo test` - 30 tests pass
✅ `npm run build` - Frontend builds

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
Recovered → Downloading (future resume) → Completed
Recovered → Cancelled