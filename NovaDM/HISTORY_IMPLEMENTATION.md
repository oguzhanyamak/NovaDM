# History Module Implementation

## Overview

This document explains the architecture and design decisions for the Download History module in NovaDM.

## Why History is Separate from Metadata

### Separation of Concerns

History is stored separately from active download metadata for several important reasons:

1. **Immutability**: Once a download completes, fails, or is cancelled, its history entry is immutable. Unlike metadata which is updated during download progress, history entries represent a final state that should never change.

2. **Different Lifecycle**: Active downloads have a lifecycle that includes:
   - Creation
   - Progress updates
   - Potential pause/resume
   - Final state (completed/failed/cancelled)
   
   History entries are created once at the end of this lifecycle and persist indefinitely.

3. **Different Access Patterns**: 
   - Metadata is accessed frequently during download (every chunk update)
   - History is accessed primarily for display and occasional cleanup
   - This separation allows for different optimization strategies

4. **Data Volume**: History can grow to 100,000+ entries, while metadata only contains active downloads. Separating them prevents the metadata directory from becoming bloated.

### Storage Architecture

```
Active Downloads: ~/.local/share/novadm/metadata/{download_id}.json
History: ~/.local/share/novadm/history/{download_id}.json
```

Each entry is stored as a separate JSON file for:
- Efficient individual entry deletion
- Easy backup/export of specific entries
- Future virtualization support (can load entries on-demand)

## Why Completed Downloads are Immutable

### Data Integrity

Once a download completes, the history entry represents a historical record. Making it immutable ensures:

1. **Audit Trail**: Users can always see what was downloaded, when, and with what result.

2. **No Accidental Modification**: Bugs in the download engine cannot corrupt historical data.

3. **Thread Safety**: Multiple operations can read history without locks since data never changes.

4. **Cacheability**: UI components can safely cache history data without worrying about staleness.

### Implementation

The `HistoryEntry` struct in Rust is designed to be created once and never modified:

```rust
pub struct HistoryEntry {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub output_path: String,
    pub status: HistoryStatus,
    pub file_size: u64,
    pub average_speed: u64,
    pub started_at: u64,
    pub completed_at: u64,
    pub duration: u64,
    pub checksum: Option<String>,
}
```

## Why Search/Filter Live in Frontend State

### Performance Considerations

For 100,000+ entries, we need to be careful about performance:

1. **No Backend Filtering**: All filtering and sorting happens in the frontend store. This is because:
   - Users expect instant search results (no network latency)
   - Filtering is a UI concern, not a data concern
   - The same data can be filtered differently by different components

2. **Memoization**: The `filteredHistory` is computed from `history` + filter + sort + search state. This allows:
   - React to skip re-renders when filtered results haven't changed
   - Efficient updates when only one parameter changes

3. **Virtualization Ready**: The current architecture prepares for future virtualization:
   - `filteredHistory` can be replaced with a virtualized list
   - Only visible items need to be rendered
   - Search/filter state remains the same

### Architecture

```
Backend (Rust)
  └── HistoryRepository
      └── Persists raw history entries

Frontend (React)
  └── HistoryStore (Zustand)
      ├── Raw history data (from backend)
      ├── Filter state (all/completed/failed/cancelled)
      ├── Sort state (newest/oldest/largest/smallest/alphabetical)
      ├── Search query
      └── Filtered/sorted history (computed)
```

## Component Architecture

### HistoryCard
Displays a single history entry with:
- File icon and name
- URL
- File size, date, duration, average speed
- Status badge
- Action buttons (open, show in folder, copy URL, download again, delete)

### HistoryToolbar
Contains:
- Search input (debounced)
- Filter buttons
- Sort dropdown
- Bulk action buttons

### HistoryFilters
Filter buttons for: All, Completed, Failed, Cancelled

### HistorySortSelect
Dropdown for sorting: Newest, Oldest, Largest, Smallest, Alphabetical

### HistorySearch
Debounced search input for instant filtering

## Performance Optimizations

1. **Debounced Search**: 300ms debounce prevents excessive re-filtering during typing

2. **Selective Re-renders**: Zustand store allows components to subscribe to only the state they need

3. **Set-based Selection**: Using `Set<string>` for selected IDs provides O(1) lookup

4. **Prepared for Virtualization**: The `filteredHistory` array can be replaced with a virtualized list component

## Future Enhancements

1. **Virtualization**: Add `react-window` or `react-virtualized` for large lists
2. **Export**: Add ability to export history to JSON/CSV
3. **Statistics**: Add aggregate statistics (total downloaded, average speed, etc.)
4. **Pagination**: For very large datasets, consider pagination instead of loading all