# NovaDM Architecture

## Overview

NovaDM is a modern desktop download manager built with Tauri v2, React, and TypeScript. It follows a clean architecture with clear separation between frontend and backend.

## Project Structure

```
NovaDM/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # Reusable UI components
│   │   ├── download/       # Download-specific components
│   │   │   ├── DownloadCard.tsx
│   │   │   ├── DownloadProgress.tsx
│   │   │   ├── DownloadSpeedLabel.tsx
│   │   │   └── DownloadStatusBadge.tsx
│   │   └── common/         # Shared components
│   │       ├── EmptyState.tsx
│   │       ├── SectionHeader.tsx
│   │       └── ConfirmationDialog.tsx
│   ├── pages/              # Page components
│   │   └── Downloads.tsx
│   ├── services/           # Tauri command wrappers
│   │   ├── download.ts     # Download operations
│   │   └── event.ts        # Event listeners
│   ├── store/              # Zustand state management
│   │   └── downloads.ts
│   └── types/              # TypeScript interfaces
│       └── index.ts
│
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── api/            # Tauri command handlers
│   │   │   └── mod.rs
│   │   ├── core/           # App state and config
│   │   │   ├── mod.rs
│   │   │   ├── config.rs
│   │   │   ├── constants.rs
│   │   │   ├── errors.rs
│   │   │   └── events.rs
│   │   ├── download/       # Download logic
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs  # DownloadManager singleton
│   │   │   ├── models.rs
│   │   │   ├── errors.rs
│   │   │   ├── utils.rs    # File conflict resolution
│   │   │   ├── worker.rs
│   │   │   ├── queue.rs
│   │   │   └── chunk.rs
│   │   ├── storage/        # Settings persistence
│   │   │   └── settings.rs
│   │   ├── utils/          # Utility functions
│   │   │   └── formatting.rs
│   │   └── lib.rs
│   └── Cargo.toml
│
└── docs/                   # Documentation
    ├── ARCHITECTURE.md
    └── ROADMAP.md
```

## Backend Architecture

### DownloadManager (Singleton)

The `DownloadManager` is the core of the download engine. It's managed by Tauri as a singleton via `app.manage()`.

```rust
pub struct DownloadManager {
    active_downloads: Arc<RwLock<HashMap<String, DownloadHandle>>>,
}
```

**Key Features:**
- **O(1) Lookup:** HashMap for fast access by download ID
- **Thread Safe:** All state protected by RwLocks
- **CancellationToken:** Each download has a token for graceful cancellation
- **Streaming:** Uses reqwest with streaming for memory-efficient downloads

### Download Flow

```
start_download command
    ↓
DownloadManager.start_download()
    ↓
Create DownloadHandle with CancellationToken
    ↓
Spawn async task
    ↓
HTTP GET with streaming
    ↓
BufWriter for efficient disk writes
    ↓
Emit progress events per chunk
    ↓
On complete: emit download://completed
    ↓
On cancel: delete partial, emit download://cancelled
    ↓
On error: emit download://error
```

### Error Handling

```rust
pub enum DownloadError {
    NetworkError(String),
    HttpError(u16),
    IoError(String),
    InvalidUrl(String),
    NotFound(String),
    Cancelled,
    PermissionDenied,
    DiskFull,
    Timeout,
    NetworkDisconnected,
}
```

## Frontend Architecture

### State Management (Zustand)

```typescript
interface DownloadsState {
  downloads: Download[];
  history: DownloadHistory[];
  currentView: 'downloads' | 'history' | 'settings';
  // Actions
  setCurrentView: (view) => void;
  addDownload: (download) => void;
  updateDownloadProgress: (id, progress, downloadedBytes, totalBytes, speed) => void;
  completeDownload: (id) => void;
  markAsCancelled: (id) => void;
}
```

### Event System

```
Backend emits events:
- download://progress
- download://completed
- download://cancelled
- download://error

Frontend listens via EventService:
- registerProgressListener()
- registerCompletedListener()
- registerCancelledListener()
- registerErrorListener()
```

## Key Design Decisions

### 1. Streaming Downloads

**Why:** Memory efficiency. A 20 GB file only uses ~64 KB of RAM.

**How:** 
- reqwest with `bytes_stream()`
- BufWriter for efficient disk writes
- Process one chunk at a time

### 2. CancellationToken

**Why:** Graceful cancellation without thread interruption.

**How:**
- Check `is_cancelled()` in download loop
- Delete partial file on cancel
- Emit cancelled event

### 3. UUID v4 for Download IDs

**Why:** Unique IDs for each download, no collisions.

**How:**
- `Uuid::new_v4().to_string()`
- Stored in DownloadHandle
- Used for event routing

### 4. File Conflict Resolution

**Why:** Prevent data loss from overwriting.

**How:**
- `resolve_filename_conflict()` in utils.rs
- Automatically renames: `file.txt` → `file (1).txt`

## Future Architecture

### Pause/Resume (Planned)

```rust
DownloadHandle {
    id: String,
    output_path: Option<String>,
    cancellation_token: CancellationToken,
    pause_token: Option<CancellationToken>,  // NEW
    bytes_downloaded: Arc<AtomicU64>,        // NEW
}
```

### Queue System (Planned)

```rust
DownloadManager {
    active_downloads: HashMap<String, DownloadHandle>,
    queue: DownloadQueue,  // NEW
}
```

## Testing

- Unit tests in `src-tauri/src/download/utils.rs`
- Integration tests via Tauri commands
- Frontend tests via React Testing Library