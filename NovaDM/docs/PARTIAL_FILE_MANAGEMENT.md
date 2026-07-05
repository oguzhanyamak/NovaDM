# Partial File Management

## Overview

Downloads write to `.part` files, then atomically rename to final filename.

## Why .part Files?

1. **Atomic completion**: Users never see partial files
2. **Resume support**: Partial files can be reused for resume
3. **Crash recovery**: Incomplete downloads can be detected
4. **Data integrity**: File is only visible when complete

## How Atomic Rename Works

```
movie.mp4.part → movie.mp4
```

The rename operation is atomic on most filesystems:
- Instant operation (no copy)
- No intermediate state
- Either old or new file exists, never both

## Why Failed Downloads Keep Partial Files

When a download fails:
- `.part` file is preserved
- Metadata is preserved
- Future resume can use them

This enables:
- Resume after network failure
- Resume after app crash
- Resume after system restart

## Why Cancelled Downloads Delete .part

When a user cancels:
- `.part` file is deleted
- Metadata is deleted
- Clean state for retry

## PartialFileManager

```rust
pub struct PartialFileManager;

impl PartialFileManager {
    pub fn part_path(&self, output_path: &Path) -> PathBuf;
    pub fn final_path(&self, part_path: &Path) -> PathBuf;
    pub async fn finalize(&self, part_path: &Path) -> std::io::Result<()>;
    pub async fn cleanup(&self, part_path: &Path) -> std::io::Result<()>;
}
```

## Download Flow

```
1. Create .part file
2. Write chunks to .part
3. On completion:
   - Flush and sync
   - Atomically rename to final
   - Emit download://completed
4. On failure:
   - Keep .part file
   - Emit download://error
5. On cancellation:
   - Delete .part file
   - Emit download://cancelled
```

## Metadata Integration

```rust
pub struct DownloadMetadata {
    pub output_path: PathBuf,     // Final file path
    pub partial_path: Option<PathBuf>,  // .part file path
    // ... other fields
}
```

The `partial_path` is stored for:
- Resume to know where partial file is
- Cleanup on cancellation
- Validation on resume