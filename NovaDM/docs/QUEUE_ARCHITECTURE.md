# Download Queue Architecture

## Overview

The download queue system ensures only a configurable number of downloads run simultaneously. It uses a FIFO (First-In-First-Out) queue with O(1) operations.

## Data Structures

### DownloadManager

```rust
pub struct DownloadManager {
    active_downloads: Arc<RwLock<HashMap<String, DownloadHandle>>>,
    scheduler: Arc<DownloadScheduler>,
}
```

- **HashMap**: O(1) lookup for active downloads by ID
- **Arc<RwLock>**: Thread-safe shared state

### DownloadScheduler

```rust
pub struct DownloadScheduler {
    queue: Arc<RwLock<VecDeque<String>>>,
    queued_tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    max_concurrent: usize,
}
```

- **VecDeque**: O(1) push/pop from both ends (FIFO queue)
- **HashMap**: O(1) task lookup by ID
- **max_concurrent**: Configurable limit (default: 3)

## Why VecDeque Was Chosen

| Operation | VecDeque | Vec |
|-----------|----------|-----|
| push_back | O(1) | O(1) amortized |
| pop_front | O(1) | O(n) - requires shift |
| front | O(1) | O(1) |
| len | O(1) | O(1) |

**VecDeque is optimal for FIFO queues** because:
- `pop_front()` is O(1) - no element shifting
- Memory is contiguous - better cache locality
- No need for `Vec::remove(0)` which is O(n)

## Scheduler Algorithm

```
start_download(url, filename, save_location):
    1. Generate UUID
    2. Create DownloadTask
    3. If active_count < max_concurrent:
        - start_download_immediately(task)
    4. Else:
        - enqueue(task)
        - emit download://queued { id, position }

start_download_immediately(task):
    1. Create DownloadHandle with CancellationToken
    2. Add to active_downloads HashMap
    3. Emit download://started { id }
    4. Spawn async worker
    5. On completion:
        - Remove from active_downloads
        - try_start_next()

try_start_next():
    1. If queue not empty:
        - pop_next() from queue
        - start_download_immediately(task)
        - Recursively call try_start_next()

cancel_download(id):
    1. If in active_downloads:
        - Cancel token
        - Remove from active
    2. Else if in queue:
        - Remove from queue
    3. Else:
        - Return NotFound error
```

## Events

| Event | Payload | When |
|-------|---------|------|
| `download://queued` | `{ id, position }` | Download added to queue |
| `download://started` | `{ id }` | Download begins |
| `download://progress` | `{ id, progress, downloaded_bytes, total_bytes, speed }` | Per-chunk |
| `download://completed` | `{ id }` | Download finishes |
| `download://cancelled` | `{ id }` | Download cancelled |
| `download://error` | `{ id, message }` | Download fails |

## Complexity Analysis

| Operation | Time | Space |
|-----------|------|-------|
| start_download | O(1) | O(1) |
| cancel_download | O(1) | O(1) |
| enqueue | O(1) | O(1) |
| dequeue | O(1) | O(1) |
| get_position | O(n) | O(1) |
| try_start_next | O(1) | O(1) |

## Future Extension Points

### Priority Downloads
```rust
// Add priority field to DownloadTask
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub save_location: String,
    pub priority: u32,  // NEW
}

// Use BinaryHeap instead of VecDeque
// VecDeque -> BinaryHeap for priority ordering
```

### Retry Queue
```rust
// Add retry count to DownloadTask
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub save_location: String,
    pub retry_count: u32,  // NEW
}

// Separate retry queue in DownloadScheduler
pub struct DownloadScheduler {
    queue: VecDeque<String>,
    retry_queue: VecDeque<String>,  // NEW
    // ...
}
```

### Pause/Resume
```rust
// Add pause token to DownloadHandle
pub struct DownloadHandle {
    pub id: String,
    pub output_path: Option<String>,
    pub cancellation_token: CancellationToken,
    pub pause_token: Option<CancellationToken>,  // NEW
    pub bytes_downloaded: Arc<AtomicU64>,         // NEW
}

// Add pause state to DownloadState
pub enum DownloadState {
    Pending,
    Queued,
    Downloading,
    Paused,      // NEW
    Completed,
    Cancelled,
    Failed,
}
```

### Bandwidth Limiting
```rust
// Add rate limiter to DownloadManager
pub struct DownloadManager {
    active_downloads: HashMap<String, DownloadHandle>,
    scheduler: DownloadScheduler,
    rate_limiter: Arc<Semaphore>,  // NEW
}
```

## Testing

Tests cover:
- FIFO ordering
- Queue position tracking
- Max concurrent limit
- Cancel queued download
- Enqueue/dequeue operations