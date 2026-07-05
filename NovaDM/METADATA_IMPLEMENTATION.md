
# Metadata Implementation

## What Changed

### Backend (Rust)

1. **DownloadMetadata** - New struct in `metadata.rs`:
   - `download_id`, `url`, `filename`, `output_path`
   - `total_bytes`, `downloaded_bytes`
   - `etag`, `last_modified` (for resume validation)
   - `created_at`, `updated_at` (timestamps)

2. **MetadataRepository** - New struct in `metadata.rs`:
   - `save()` - Write metadata to JSON file
   - `load()` - Read metadata from disk
   - `update()` - Update existing metadata
   - `delete()` - Remove metadata file

3. **DownloadManager** - Updated in `manager.rs`:
   - Added `metadata_repo` field
   - Creates metadata on download start
   - Updates metadata every 1MB
   - Deletes metadata on completion/cancellation
   - Errors are logged, not thrown

### Tests

Added 4 tests in `metadata.rs`:
- `test_save_and_load_metadata`
- `test_update_metadata`
- `test_delete_metadata`
- `test_load_nonexistent`

## Verification

✅ `cargo check` - No errors
✅ `cargo test` - 18 tests pass
✅ `npm run build` - Frontend builds (no changes)

## Architecture

```
DownloadManager
    ↓
MetadataRepository
    ↓
<temp_dir>/novadm/metadata/<id>.json
```

## Why Metadata Exists

- **Resume Support**: Track progress for pause/resume
- **Crash Recovery**: Recover downloads after crash
- **Validation**: ETag/Last-Modified verify file integrity
- **User Experience**: Show download history

## Why Repository Pattern

- Business logic doesn't know about JSON
- Easy to swap storage (JSON → SQLite)
- Testable without file system
- Errors don't crash downloads

## How This Enables Pause/Resume

When pause is implemented:
1. Keep metadata (don't delete)
2. Load metadata on resume
3. Check ETag/Last-Modified match
4. Use HTTP Range header to resume
5. Continue from `downloaded_bytes`