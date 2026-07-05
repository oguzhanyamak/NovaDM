# Resume Capability Detection

## Overview

HTTP resume capability detection for downloads. Determines if a server supports resumable downloads.

## How HTTP Resume Works

HTTP resume uses the `Range` header to request partial content:

```
GET /file.zip HTTP/1.1
Range: bytes=1024-
```

The server responds with:
- `206 Partial Content` - Resume supported
- `200 OK` - Resume not supported (sends entire file)

## Why Accept-Ranges Alone Is Not Enough

The `Accept-Ranges` header indicates support, but:

1. **Not all servers send it**: Some servers support ranges but don't advertise it
2. **Must be "bytes"**: Other values (like "none") indicate no support
3. **Content-Length required**: Without it, we can't know the total size

**Our check:**
```rust
resume_supported = accept_ranges == "bytes" && has_content_length
```

## Why ETag and Last-Modified Are Stored

These headers enable **integrity validation** for resume:

- **ETag**: Unique identifier for the file version
- **Last-Modified**: Timestamp of last modification

When resuming, we verify:
- ETag matches (file hasn't changed)
- Last-Modified matches (file hasn't been updated)

If they differ, the download must restart.

## Detection Flow

```
1. Send HTTP HEAD or GET request
2. Check Accept-Ranges header
3. Check Content-Length header
4. Check ETag header
5. Check Last-Modified header
6. Store in metadata
```

## ResumeCapability Struct

```rust
pub struct ResumeCapability {
    pub resume_supported: bool,      // Can we resume?
    pub has_content_length: bool,    // Do we know total size?
    pub has_etag: bool,            // Do we have ETag?
    pub has_last_modified: bool,   // Do we have Last-Modified?
}
```

## Metadata Integration

```rust
pub struct DownloadMetadata {
    // ... existing fields ...
    pub resume_supported: bool,  // New field
}
```

The `#[serde(default)]` attribute ensures backward compatibility.

## Future Resume Implementation

When pause/resume is implemented:

```rust
// On pause:
// 1. Keep metadata (don't delete)
// 2. Store current position
// 3. Emit download://paused event

// On resume:
// 1. Load metadata
// 2. Check ETag/Last-Modified match
// 3. Send Range: bytes=<downloaded_bytes>-
// 4. Continue from position
```

## Error Handling

Detection failures never crash downloads:
- Log the error
- Continue downloading normally
- Mark `resume_supported = false`