# Retry Implementation

## What Changed

### Backend (Rust)

1. **DownloadError::InvalidState** - New error variant for invalid state operations

2. **DownloadManager::retry_download** - New method:
   - Validates download is not active or queued
   - Retrieves task from failed_downloads map
   - Generates new UUID for retry
   - Emits `download://retry` event
   - Enters scheduler (respects concurrent limit)

3. **DownloadManager::start_download_immediately** - Updated:
   - Stores failed task in failed_downloads map on error

4. **DownloadScheduler::contains** - New method:
   - Checks if a task is in the queue

5. **retry_download Tauri command** - New command in api/mod.rs

### Frontend (TypeScript)

1. **EventService** - Added:
   - `DownloadRetryData` interface
   - `registerRetryListener` method

2. **DownloadService** - Added:
   - `retryDownload(id)` method

3. **Store** - Added:
   - `retryDownload(id)` action

4. **DownloadCard** - Added:
   - Retry button for failed downloads
   - Uses RefreshCw icon

5. **Downloads page** - Added:
   - Retry event listener

## Verification

✅ `cargo check` - No errors
✅ `cargo test` - 14 tests pass
✅ `npm run build` - Frontend builds

## Architecture

### Retry Flow

```
Failed Download
    ↓
User clicks Retry
    ↓
downloadService.retryDownload(id)
    ↓
Tauri retry_download command
    ↓
DownloadManager.retry_download()
    - Check active → Error
    - Check queued → Error
    - Get from failed_downloads
    - Generate new UUID
    - Emit download://retry
    - Enter scheduler
    ↓
Scheduler (respects max_concurrent)
    ↓
download://queued or download://started
```

### State Transitions

```
Failed → Queued → Downloading → Completed
                     ↓
                     Failed (retryable)
```

## Future: Auto Retry

The architecture supports automatic retry:

```rust
// Add to DownloadScheduler
retry_queue: VecDeque<String>
retry_counts: HashMap<String, u32>
max_retries: u32

// On error, if under max_retries:
retry_queue.push_back(id)
```

This would:
- Use exponential backoff
- Run after new downloads
- Be cancellable by user