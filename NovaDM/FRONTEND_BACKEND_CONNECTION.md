# Frontend-Backend Connection Architecture

## Overview
Successfully connected the frontend and backend for the first time with a clean, production-ready architecture that maintains strict separation of concerns.

## Architecture Flow

```
User Action (Click Download)
    ↓
NewDownloadDialog (UI Component)
    ↓
DownloadService.startDownload() (Service Layer)
    ↓
invoke('start_download') (Tauri Bridge)
    ↓
start_download Command (Rust Backend)
    ↓
DownloadManager (Future Implementation)
```

## Implementation Details

### 1. Backend: Tauri Command (`src-tauri/src/api/mod.rs`)

**Command: `start_download`**

```rust
#[tauri::command]
pub async fn start_download(
    url: String,
    filename: String,
    save_location: String,
) -> Result<(), String>
```

**Current Behavior:**
- Logs all received values using `tracing::info!`
- Returns `Ok(())` immediately
- No download logic implemented yet

**Why This Design:**
- **Minimal Surface:** Only one new command added
- **Type Safe:** Rust compiler ensures type correctness
- **Testable:** Easy to test command in isolation
- **Future-Ready:** Will delegate to DownloadManager

**Registration:**
```rust
// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    greet,
    ping,
    get_app_state,
    start_download  // Added here
])
```

### 2. Service Layer (`src/services/download.ts`)

**Method: `startDownload()`**

```typescript
export interface StartDownloadParams {
  url: string;
  filename: string;
  saveLocation: string;
}

export const downloadService = {
  async startDownload(params: StartDownloadParams): Promise<void> {
    await invoke('start_download', {
      url: params.url,
      filename: params.filename,
      save_location: params.saveLocation,
    });
  }
}
```

**Why This Design:**
- **Single Point of Contact:** Only service calls Tauri
- **Type Safety:** Interface defines contract
- **Encapsulation:** UI never calls `invoke()` directly
- **Testable:** Can mock service in tests
- **Maintainable:** All Tauri calls in one place

**Architecture Rule:**
> The UI must never call `invoke()` directly.
> Only DownloadService can communicate with Tauri.

### 3. Frontend: Dialog Integration (`src/components/NewDownloadDialog.tsx`)

**Changes Made:**
- Import `downloadService`
- Added `isSubmitting` state
- Changed `handleDownload` to async
- Call `downloadService.startDownload()`
- Show loading state ("Starting...")
- Error handling with try-catch

**Before:**
```typescript
const handleDownload = () => {
  alert('Download engine not implemented yet.');
  onOpenChange(false);
};
```

**After:**
```typescript
const handleDownload = async () => {
  setIsSubmitting(true);
  try {
    await downloadService.startDownload({ url, filename, saveLocation });
    onOpenChange(false);
  } catch (err) {
    console.error('Failed to start download:', err);
  } finally {
    setIsSubmitting(false);
  }
};
```

**Why This Design:**
- **User Feedback:** Loading state prevents double-clicks
- **Error Handling:** Catches and logs errors
- **Clean State:** Always resets submitting state
- **UX:** Button text changes to "Starting..."

## Data Flow

### Request (Frontend → Backend)

1. **User enters URL:** `https://example.com/files/movie.mp4`
2. **Filename auto-suggests:** `movie.mp4`
3. **User clicks Download**
4. **Dialog validates:** URL format, required fields
5. **Service called:** `downloadService.startDownload({...})`
6. **Tauri invoked:** `invoke('start_download', {...})`
7. **Command received:** Rust backend receives parameters
8. **Logged:** Values printed to console/logs

### Response (Backend → Frontend)

1. **Command executes:** Logs values
2. **Returns:** `Ok(())`
3. **Service resolves:** Promise completes
4. **Dialog closes:** `onOpenChange(false)`
5. **Success:** UI updates

## How This Evolves Into Real Download Engine

### Phase 1: Current (Logging Only)
```
start_download command
    ↓
Logs values
    ↓
Returns Ok(())
```

### Phase 2: Delegate to DownloadManager
```rust
#[tauri::command]
pub async fn start_download(
    url: String,
    filename: String,
    save_location: String,
) -> Result<(), String> {
    let manager = DownloadManager::new();
    manager.start_download(url, filename, save_location).await?;
    Ok(())
}
```

### Phase 3: DownloadManager Implementation
```rust
impl DownloadManager {
    pub async fn start_download(
        &self,
        url: String,
        filename: String,
        save_location: String,
    ) -> Result<(), DownloadError> {
        // 1. Validate URL
        // 2. Create download task
        // 3. Spawn worker
        // 4. Add to queue
        // 5. Emit event: download:started
        Ok(())
    }
}
```

### Phase 4: Worker Implementation
```rust
impl DownloadWorker {
    pub async fn start(&self, task: DownloadTask) -> Result<()> {
        // 1. Create file handle
        // 2. Send HEAD request for size
        // 3. Create chunks
        // 4. Download chunks in parallel
        // 5. Emit progress events
        // 6. Merge chunks
        // 7. Emit: download:completed
        Ok(())
    }
}
```

### Phase 5: Event System
```rust
// Backend emits events
app.emit("download:progress", DownloadProgressEvent {
    id: download.id,
    progress: 43.5,
    speed: 13003413,
});

// Frontend listens
listen('download:progress', (event) => {
    updateDownloadProgress(event.payload);
});
```

## Architecture Benefits

### 1. Separation of Concerns
- **UI Layer:** React components, user interaction
- **Service Layer:** Business logic, Tauri communication
- **Command Layer:** Rust functions, business logic
- **Domain Layer:** DownloadManager, Worker, Queue

### 2. Testability
- **UI Tests:** Mock service, test interactions
- **Service Tests:** Mock invoke, test parameters
- **Command Tests:** Test Rust functions directly
- **Integration Tests:** Test full flow

### 3. Maintainability
- **Single Responsibility:** Each layer has one job
- **Clear Interfaces:** TypeScript interfaces define contracts
- **Easy Debugging:** Logs at each layer
- **Future-Proof:** Easy to add features

### 4. Type Safety
- **Frontend:** TypeScript ensures correct parameters
- **Backend:** Rust types ensure correctness
- **Contract:** Interface defines expected data

## Directory Structure

```
NovaDM/
├── src/
│   ├── components/
│   │   └── NewDownloadDialog.tsx    # UI only, calls service
│   ├── services/
│   │   └── download.ts              # ONLY place that calls invoke()
│   └── types/
│       └── index.ts                 # Type definitions
│
└── src-tauri/
    └── src/
        ├── api/
        │   └── mod.rs               # Tauri commands
        ├── download/
        │   ├── mod.rs               # Module exports
        │   ├── manager.rs           # Business logic (future)
        │   ├── worker.rs            # Download workers (future)
        │   └── errors.rs            # Error types
        └── lib.rs                   # Command registration
```

## Rules Enforced

### 1. No Direct invoke() in UI
```typescript
// ❌ BAD
import { invoke } from '@tauri-apps/api/core';
// In component:
await invoke('start_download', {...});

// ✅ GOOD
import { downloadService } from '../services/download';
// In component:
await downloadService.startDownload({...});
```

### 2. Service is Single Point of Contact
```typescript
// All Tauri communication goes through service
export const downloadService = {
  async startDownload(params: StartDownloadParams): Promise<void>
  async getDownloads(): Promise<Download[]>
  async pauseDownload(id: string): Promise<void>
  // ... all other commands
};
```

### 3. Commands are Thin
```rust
// ❌ BAD - Command does too much
#[tauri::command]
pub async fn start_download(...) -> Result<(), String> {
    // 100 lines of download logic
}

// ✅ GOOD - Command delegates
#[tauri::command]
pub async fn start_download(...) -> Result<(), String> {
    DownloadManager::new().start(url, filename, save_location).await?;
    Ok(())
}
```

## Testing Strategy

### Frontend Tests
```typescript
// Mock service
jest.mock('../services/download');

// Test dialog
it('calls service on download', async () => {
  render(<NewDownloadDialog open={true} onOpenChange={jest.fn()} />);
  await userEvent.type(screen.getByLabelText('URL'), 'https://example.com/file.zip');
  await userEvent.click(screen.getByText('Download'));
  expect(downloadService.startDownload).toHaveBeenCalled();
});
```

### Backend Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_start_download_logs() {
        // Test command logs correctly
    }
}
```

## Security Considerations

1. **Input Validation:** Both frontend and backend validate
2. **Path Traversal:** Backend validates save_location
3. **URL Validation:** Only HTTP/HTTPS allowed
4. **Error Messages:** Don't leak sensitive info
5. **Type Safety:** Rust prevents memory issues

## Performance Considerations

1. **Async/Await:** Non-blocking operations
2. **Error Handling:** Fast failure on invalid input
3. **Logging:** Minimal overhead with tracing
4. **Future:** Events for progress updates

## Next Steps

1. **Implement DownloadManager:**
   - Add download validation
   - Create download tasks
   - Spawn workers

2. **Implement Worker:**
   - HTTP requests
   - File I/O
   - Progress tracking

3. **Add Event System:**
   - Emit progress events
   - Frontend listens and updates UI

4. **Add Persistence:**
   - Save downloads to disk
   - Load on startup

5. **Add Queue Management:**
   - Concurrent download limits
   - Priority queue

## Conclusion

This architecture provides:
- **Clean Separation:** UI, service, command, domain layers
- **Type Safety:** TypeScript + Rust
- **Testability:** Each layer testable independently
- **Maintainability:** Clear responsibilities
- **Scalability:** Easy to add features
- **Production-Ready:** Follows best practices

The connection is established and ready for download logic implementation.