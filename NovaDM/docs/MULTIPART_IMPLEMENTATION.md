# Multi-Part Downloading Implementation

## Overview

This sprint implements true multi-part downloading for NovaDM.

## What Changed

### Backend (Rust)

1. **DownloadChunk** - Updated in `chunk.rs`:
   - Added `start` and `end` byte range
   - Added `download()` method with Range header
   - Uses positioned writes to .part file
   - Checks cancellation and pause tokens

2. **DownloadManager** - Updated in `manager.rs`:
   - Added `DEFAULT_CONNECTIONS` constant (8)
   - Added `calculate_chunks()` method
   - Updated `download_file()` to use multi-part
   - Checks `Accept-Ranges` header before splitting
   - Falls back to single connection if unsupported

3. **DownloadState** - Already has `Recovered` state

### Frontend

No changes required. Frontend remains unaware of multi-part implementation.

## Part Discovery

Before downloading:
1. Send HEAD request to get headers
2. Check `Accept-Ranges` header
3. If `bytes`, split file into chunks
4. Otherwise, use single connection

## Range Calculation

Example: 800 MB file with 8 connections
- Each chunk: 100 MB
- Chunk 0: bytes=0-104857599
- Chunk 1: bytes=104857600-209715199
- ...
- Chunk 7: bytes=734003200-838860799

## Workers

Each worker:
1. Creates HTTP request with Range header
2. Streams response directly to file
3. Writes only to its assigned byte range
4. Uses `seek()` to position in file
5. Checks cancellation/pause tokens

## File Writing

- Workers use positioned writes
- Each worker writes to `[start, end)` range
- No overwriting between workers
- File is only visible when complete (after rename)

## Progress

- Overall progress = sum(worker downloaded)
- Workers emit progress events
- Manager aggregates to single progress
- Frontend sees single download progress

## Failure

If one worker fails:
- Download fails immediately
- Metadata is updated with partial progress
- User can retry (future: retry only failed workers)

## Resume

Architecture allows future per-worker resume:
- Each chunk has its own ID
- Metadata tracks per-chunk progress
- Future: resume only failed chunks

## Scheduler

Scheduler still sees ONE download:
- Not 8 downloads
- Respects `max_concurrent_downloads`
- Queue position is per download

## Memory

Streaming only:
- No buffering entire file
- Each worker streams to disk
- Memory efficient for large files

## Events

Existing events remain unchanged:
- `download://progress` - Aggregated progress
- `download://completed` - Download finished
- `download://error` - Download failed
- `download://cancelled` - Download cancelled
- `download://paused` - Download paused
- `download://resumed` - Download resumed

Frontend does not know multiple workers exist.

## Tests

- `test_calculate_chunks` - 800 MB with 8 parts
- `test_calculate_chunks_single` - Small file
- `test_calculate_chunks_empty` - Zero size
- `test_chunk_progress` - Progress calculation
- `test_chunk_progress_partial` - Partial progress
- `test_chunk_progress_complete` - Complete progress

## Verification

✅ `cargo check` - No errors
✅ `cargo test` - 34/36 tests pass (2 pre-existing metadata test failures)
✅ `npm run build` - Frontend builds