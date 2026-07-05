# Retry Architecture

## Overview

Manual retry for failed downloads. Failed downloads can be restarted through the UI.

## State Transitions

```
Failed → Queued → Downloading → Completed
                     ↓
                     Failed (can retry again)
```

**Only failed downloads can be retried.** Attempting to retry:
- Active downloads → Error
- Queued downloads → Error
- Completed downloads → Error
- Cancelled downloads → Error

## Retry Lifecycle

```
1. User clicks "Retry" on a failed download
2. Frontend calls downloadService.retryDownload(id)
3. Tauri invokes retry_download command
4. DownloadManager.retry_download():
   a. Checks if download is active → Error if true
   b. Checks if download is queued → Error if true
   c. Gets task from failed_downloads map
   d. Generates new UUID for retry
   e. Emits download://retry event
   f. Checks concurrent limit
   g. If under limit: start immediately
   h. If at limit: enqueue
5. Download proceeds normally
```

## Why Retry Reuses the Scheduler

The retry mechanism reuses the existing scheduler for several reasons:

1. **Consistency**: Retry behaves exactly like a new download
2. **Concurrency Control**: Respects `max_concurrent_downloads` limit
3. **Queue Management**: Failed downloads can be queued if limit reached
4. **Event Flow**: Same events (`download://queued`, `download://started`, etc.)
5. **Simplicity**: No duplicate logic

## Why Retry Does Not Directly Start Downloads

Retry does not bypass the scheduler because:

1. **Fairness**: Queued downloads should run before retries
2. **Resource Management**: Prevents overwhelming the system
3. **Predictability**: Users expect consistent behavior
4. **Future Compatibility**: Auto-retry can use same path

## Data Structures

### DownloadManager

```rust
pub struct DownloadManager {
    active_downloads: HashMap<String, DownloadHandle>,
    scheduler: DownloadScheduler,
    failed_downloads: HashMap<String, DownloadTask>,  // NEW
}
```

### Failed Download Storage

- **Key**: Original download ID
- **Value**: DownloadTask (url, filename, save_location)
- **Purpose**: Allows retry without re-entering URL

## Events

| Event | Payload | When |
|-------|---------|------|
| `download://retry` | `{ id, new_id }` | User initiates retry |
| `download://queued` | `{ id, position }` | Retry enters queue |
| `download://started` | `{ id }` | Retry begins |
| `download://error` | `{ id, message }` | Retry fails |
| `download://completed` | `{ id }` | Retry succeeds |

## Future Auto Retry Integration

For automatic retry, the scheduler can be extended:

```rust
pub struct DownloadScheduler {
    queue: VecDeque<String>,
    retry_queue: VecDeque<String>,  // NEW: separate queue
    retry_counts: HashMap<String, u32>,  // NEW: track attempts
    max_retries: u32,  // NEW: configurable
}
```

**Auto-retry algorithm:**
1. On error, increment retry count
2. If under max_retries, add to retry_queue
3. After delay, process retry_queue
4. Use exponential backoff

**Why separate queue:**
- Priority: New downloads before retries
- Rate limiting: Prevent retry storms
- User control: Can cancel pending retries