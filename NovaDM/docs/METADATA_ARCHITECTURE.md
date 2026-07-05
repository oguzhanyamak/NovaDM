# Metadata Architecture

## Overview

Download metadata is persisted to disk to enable pause/resume support in the future.

## Why Metadata Exists

Metadata serves as a checkpoint for downloads:

1. **Resume Support**: If a download is paused, metadata tracks progress
2. **Crash Recovery**: If the app crashes, downloads can be recovered
3. **Validation**: ETag and Last-Modified headers verify file hasn't changed
4. **User Experience**: Users can see download history and retry failed downloads

## Why Repository Pattern Was Chosen

The repository pattern separates business logic from persistence:

```
DownloadManager → MetadataRepository → File System
```

**Benefits:**
- Business logic doesn't know about JSON or file paths
- Easy to swap storage (JSON → SQLite → etc.)
- Testable without file system
- Errors don't crash downloads

## How This Enables Pause/Resume Later

### Current State
- Metadata is saved on download start
- Updated periodically during download
- Deleted on completion or cancellation

### Future Pause/Resume
When pause is implemented:

```rust
// On pause:
// 1. Keep metadata (don't delete)
// 2. Store bytes_downloaded in handle
// 3. Emit download://paused event

// On resume:
// 1. Load metadata
// 2. Check ETag/Last-Modified match
// 3. Use HTTP Range header to resume
// 4. Continue from downloaded_bytes
```

## Metadata Model

```rust
pub struct DownloadMetadata {
    download_id: String,      // Unique identifier
    url: String,            // Source URL
    filename: String,       // Output filename
    output_path: PathBuf,   // Full output path
    total_bytes: Option<u64>, // Total size (if known)
    downloaded_bytes: u64,  // Progress
    etag: Option<String>,   // HTTP ETag
    last_modified: Option<String>, // HTTP Last-Modified
    created_at: u64,        // Unix timestamp
    updated_at: u64,        // Unix timestamp
}
```

## Update Interval

Metadata is updated every 1MB or on completion. This balances:
- **Performance**: Not every chunk triggers a disk write
- **Safety**: Progress is saved frequently enough
- **Efficiency**: Minimal I/O overhead

## Error Handling

Metadata failures are logged but don't crash downloads:

```rust
if let Err(e) = metadata_repo.update(&metadata).await {
    tracing::warn!("Failed to update metadata: {}", e);
}
```

This ensures:
- Downloads continue even if disk is full
- Network issues don't affect download
- User experience is not degraded

## File Storage

One JSON file per download:
```
<temp_dir>/novadm/metadata/<download_id>.json
```

Example:
```json
{
  "download_id": "abc-123",
  "url": "https://example.com/file.zip",
  "filename": "file.zip",
  "output_path": "/downloads/file.zip",
  "total_bytes": 1048576,
  "downloaded_bytes": 524288,
  "etag": "\"abc123\"",
  "last_modified": "Wed, 21 Oct 2023 07:28:00 GMT",
  "created_at": 1693520000,
  "updated_at": 1693520060
}
```

## Future Extensions

### Additional Fields
- `pause_token` - For pause support
- `bytes_downloaded` - Atomic counter for thread safety
- `status` - Current download state

### Storage Options
- SQLite for better performance
- App's config directory instead of temp
- Encryption for sensitive downloads