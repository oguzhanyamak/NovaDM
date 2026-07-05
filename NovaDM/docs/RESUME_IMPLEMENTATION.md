# HTTP Resume Implementation

## Overview

This sprint implements true HTTP resume for paused downloads.

## What Changed

### Backend (Rust)

1. **DownloadError** - Extended in `errors.rs`:
   - Added `FileChanged` error
   - Added `ResumeUnsupported` error

2. **DownloadManager** - Updated in `manager.rs`:
   - Added `resume_download(id)` method
   - Added `send_request_with_range(url, start)` method
   - Updated `download_file` to support resume mode
   - Opens .part file in append mode for resume
   - Uses HTTP Range header for partial content

3. **DownloadHandle** - Already has `pause_token` from previous sprint

### Frontend

1. **EventService** - Updated in `event.ts`:
   - Added `DownloadResumedData` interface
   - Added `registerResumedListener` method

2. **Store** - Updated in `downloads.ts`:
   - Added `markAsResumed` method

## HTTP Range

The HTTP `Range` header requests partial content:

```
Range: bytes=1024-
```

This requests the file starting from byte 1024 to the end.

## 206 Partial Content

When a server supports range requests, it responds with:
- Status: `206 Partial Content`
- Content starting from the requested byte

If the server returns `200 OK`, it ignores the Range header and sends the entire file.

## Validation Process

Before resuming, the system validates:

1. **Metadata exists** - Load from disk
2. **resume_supported == true** - Server must support ranges
3. **Partial file exists** - Check .part file on disk
4. **Remote file unchanged** - Verify ETag and Last-Modified match

## Why Overwrite is Forbidden

- Resume must continue from where it left off
- Overwriting would lose already downloaded data
- Append mode ensures data integrity
- File is only visible when complete (after rename)

## Error Handling

| Error | Cause |
|-------|-------|
| `NotFound` | No metadata for download |
| `ResumeUnsupported` | Server doesn't support Range |
| `FileChanged` | ETag or Last-Modified mismatch |
| `InvalidState` | No partial file path |

## State Transitions

```
Paused
    ↓
Downloading (resume)
    ↓
Completed
```

## Progress

- Progress continues from stored `downloaded_bytes`
- Speed calculation starts fresh on resume
- Metadata is updated during download

## Scheduler Behavior

Resume behaves like a new active download:
- Respects `max_concurrent_downloads`
- If at limit, queues the resume
- On completion, metadata is deleted
- On completion, .part is renamed to final file