# Partial File Management Implementation

## What Changed

### Backend (Rust)

1. **PartialFileManager** - New struct in `partial_file.rs`:
   - `part_path()` - Generate .part file path
   - `final_path()` - Get final file path
   - `finalize()` - Atomic rename with sync
   - `cleanup()` - Delete .part file

2. **DownloadMetadata** - Extended in `metadata.rs`:
   - Added `partial_path: Option<PathBuf>` field
   - `#[serde(default)]` for backward compatibility
   - `set_partial_path()` method

3. **DownloadManager** - Updated in `manager.rs`:
   - Added `partial_file_manager` field
   - Downloads write to .part files
   - On completion: flush, sync, rename
   - On failure: keep .part file
   - On cancellation: delete .part file

### Tests

Added 6 tests in `partial_file.rs`:
- `test_part_path_generation`
- `test_part_path_no_extension`
- `test_final_path`
- `test_cleanup_nonexistent`
- `test_finalize_nonexistent`
- `test_finalize_and_cleanup`

## Verification

✅ `cargo check` - No errors
✅ `cargo test` - 28 tests pass
✅ `npm run build` - Frontend builds (no changes)

## Architecture

```
DownloadManager
    ↓
PartialFileManager
    ↓
movie.mp4.part → movie.mp4 (atomic rename)
```

## Why .part Files?

1. **Atomic completion**: Users never see partial files
2. **Resume support**: Partial files can be reused
3. **Crash recovery**: Incomplete downloads detected
4. **Data integrity**: File only visible when complete

## Why Failed Downloads Keep .part

- Resume can use the partial file
- Metadata already exists
- No data loss on network failure

## Why Cancelled Downloads Delete .part

- User explicitly cancelled
- No intent to resume
- Clean state for retry