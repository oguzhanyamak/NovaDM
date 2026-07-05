# Bandwidth Limiter Implementation

## What Changed

### Backend (Rust)

1. **BandwidthLimiter** - New struct in `bandwidth.rs`:
   - Token bucket algorithm
   - Thread-safe with `Arc<RwLock<>>`
   - Cloneable for sharing across workers
   - Default: unlimited (0)

2. **DownloadManager** - Updated in `manager.rs`:
   - Added `bandwidth_limiter` field
   - Added `set_bandwidth_limit()` method
   - Added `get_bandwidth_limit()` method
   - Passes limiter to `download_file()` and `try_start_next()`

3. **DownloadChunk** - Updated in `chunk.rs`:
   - Added `bandwidth_limiter` field
   - Added `with_bandwidth_limiter()` method
   - Calls `limiter.acquire()` before each write

4. **API** - Added `set_bandwidth_limit` command

## Why Token Bucket

The token bucket algorithm was chosen because:
- It allows natural bursts when bandwidth is available
- It doesn't require busy waiting
- It's simple and efficient
- It's well-suited for download limiting

## Why Global

The limiter is global (not per-download) because:
- Users want to limit total bandwidth
- Easier to configure
- Prevents overwhelming the network
- Works with multi-part downloads

## Algorithm

1. Tokens are added at the rate of `limit` bytes per second
2. Each write consumes tokens
3. If no tokens available, worker sleeps
4. Limit changes take effect immediately

## Configuration

Settings page options:
- Unlimited
- 1 MB/s
- 2 MB/s
- 5 MB/s
- 10 MB/s
- 20 MB/s
- 50 MB/s
- 100 MB/s

## Performance

- Works with single connection
- Works with multi-part downloads
- Works with queued downloads
- Works with resume
- No busy waiting
- Workers sleep naturally when limited

## Tests

- `test_unlimited` - Unlimited bandwidth
- `test_limited` - Limited bandwidth
- `test_dynamic_limit_change` - Dynamic limit change

## Verification

✅ `cargo check` - No errors
✅ `cargo test` - 39 tests pass
✅ `npm run build` - Frontend builds