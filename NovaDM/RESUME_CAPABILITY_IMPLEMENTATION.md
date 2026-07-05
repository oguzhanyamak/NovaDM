# Resume Capability Implementation

## What Changed

### Backend (Rust)

1. **ResumeCapability** - New struct in `resume_detector.rs`:
   - `resume_supported: bool`
   - `has_content_length: bool`
   - `has_etag: bool`
   - `has_last_modified: bool`

2. **ResumeCapabilityDetector** - New struct in `resume_detector.rs`:
   - `detect(&self, response: &Response) -> ResumeCapability`
   - Checks Accept-Ranges, Content-Length, ETag, Last-Modified

3. **DownloadMetadata** - Extended in `metadata.rs`:
   - Added `resume_supported: bool` field
   - `#[serde(default)]` for backward compatibility
   - `set_resume_capability()` method

4. **DownloadManager** - Updated in `manager.rs`:
   - Added `resume_detector` field
   - Detects capability on download start
   - Stores in metadata

### Tests

Added 4 tests:
- `test_accept_ranges_bytes`
- `test_accept_ranges_missing`
- `test_accept_ranges_none`
- `test_backward_compatibility`

## Verification

✅ `cargo check` - No errors
✅ `cargo test` - 22 tests pass
✅ `npm run build` - Frontend builds (no changes)

## Architecture

```
DownloadManager
    ↓
ResumeCapabilityDetector
    ↓
HTTP Response Headers
```

## Detection Logic

```rust
let accept_ranges = response.headers()
    .get(ACCEPT_RANGES)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_lowercase())
    .unwrap_or_default();

let has_content_length = response.content_length().is_some();

let resume_supported = accept_ranges == "bytes" && has_content_length;
```

## Why This Enables Future Resume

1. **Capability known**: We know if server supports ranges
2. **Headers stored**: ETag/Last-Modified for validation
3. **Metadata persists**: State survives app restart
4. **No breaking changes**: Backward compatible