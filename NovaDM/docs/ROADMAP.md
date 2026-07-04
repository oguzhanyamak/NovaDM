# NovaDM Roadmap

## Alpha Release (Current)

### Core Features
- [x] HTTP download with streaming
- [x] Progress events
- [x] Cancellation with CancellationToken
- [x] File conflict resolution (auto-rename)
- [x] Open file after completion
- [x] Show in folder
- [x] Structured error handling
- [x] UUID-based download IDs
- [x] Clean architecture

### Quality Improvements
- [x] File conflict handling (rename automatically)
- [x] Open file after completion
- [x] Show in folder
- [x] Better error differentiation
- [x] Unit tests for conflict resolution
- [x] Documentation

## Beta Release

### Download Management
- [ ] Pause/Resume support
  - HTTP Range requests
  - Partial file tracking
  - Resume offset storage
- [ ] Download queue
  - Concurrent download limits
  - Queue position display
  - Priority ordering
- [ ] Retry logic
  - Exponential backoff
  - Configurable retry count
  - Resume on retry

### UI/UX
- [ ] Settings page
  - Default download folder
  - Concurrent download limit
  - Connection timeout
- [ ] History page
  - Completed downloads
  - Failed downloads
  - Clear history
- [ ] Download details
  - File size
  - Download speed
  - Time remaining
  - Source URL

### Storage
- [ ] Persistent settings
  - Save to app config directory
  - Restore on startup
  - Migration support

## 1.0 Release

### Advanced Features
- [ ] Multi-part downloads
  - Parallel chunk downloading
  - Server support detection
- [ ] Browser extension
  - Chrome/Firefox integration
  - Video download detection
- [ ] Checksum verification
  - SHA256/MD5 support
  - Automatic verification
- [ ] Scheduled downloads
  - Start at specific time
  - Recurring downloads

### Platform Support
- [ ] Windows (complete)
- [ ] macOS (complete)
- [ ] Linux (complete)
- [ ] Mobile (iOS/Android)

## Future Vision

### 2.0+
- [ ] Torrent support
- [ ] Cloud storage integration
- [ ] Download categories/tags
- [ ] Bandwidth scheduling
- [ ] Plugin system

## Technical Debt

### Cleanup
- [ ] Remove unused placeholder code
- [ ] Add integration tests
- [ ] E2E tests with Tauri
- [ ] Performance benchmarks

### Documentation
- [ ] API documentation
- [ ] User guide
- [ ] Developer guide