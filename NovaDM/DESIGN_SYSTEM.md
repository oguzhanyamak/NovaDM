# NovaDM Design System

## Overview
Refactored the NovaDM UI into a reusable desktop design system with composable components. All components are fully typed, themeable, and follow consistent patterns.

## Component Architecture

### Download Components (`components/download/`)

Specialized components for displaying download information and progress.

#### 1. DownloadCard
**Purpose:** Primary container for displaying a single download item in the list.

**Why It Exists:**
- **Consistency:** Ensures all download items follow the same layout and styling
- **Reusability:** Can be used in Downloads page, History page, or any future list view
- **Composition:** Combines multiple smaller components (icon, progress, status, speed)
- **Props-based:** Fully data-driven, no hardcoded content
- **Responsive:** Handles long filenames and URLs with truncation

**Props:**
- `id` - Unique identifier (used for testing)
- `name` - Download filename
- `url` - Source URL
- `status` - Download status (pending/downloading/paused/completed/error)
- `progress` - Progress percentage (0-100)
- `speed` - Download speed in bytes/second
- `className` - Additional styling

**Used In:**
- Downloads page (main list)
- Future: History page, search results

#### 2. DownloadProgress
**Purpose:** Visual progress bar with percentage label.

**Why It Exists:**
- **Separation:** Progress logic isolated from card layout
- **Reusability:** Can be used in different contexts (card, detail view, etc.)
- **Animation:** Smooth transitions with CSS
- **Clamping:** Handles edge cases (negative, >100%)

**Props:**
- `progress` - Progress percentage
- `className` - Additional styling

**Used In:**
- DownloadCard (when downloading or paused)

#### 3. DownloadStatusBadge
**Purpose:** Color-coded status indicator with label.

**Why It Exists:**
- **Visual Hierarchy:** Color-coded status for quick scanning
- **Consistency:** Same status colors throughout app
- **Accessibility:** Clear text labels with color coding
- **Maintainability:** Status logic in one place

**Status Colors:**
- Downloading: Blue
- Paused: Yellow
- Completed: Green
- Error: Red
- Pending: Gray

**Props:**
- `status` - Download status enum
- `className` - Additional styling

**Used In:**
- DownloadCard
- Future: Detail view, notifications

#### 4. DownloadSpeedLabel
**Purpose:** Human-readable download speed display.

**Why It Exists:**
- **Formatting:** Converts bytes/second to KB/s, MB/s, etc.
- **Consistency:** Same formatting everywhere
- **Edge Cases:** Handles 0 speed, very high speeds
- **Localization Ready:** Easy to add i18n later

**Props:**
- `speed` - Speed in bytes/second
- `className` - Additional styling

**Used In:**
- DownloadCard (when downloading)

#### 5. DownloadFileIcon
**Purpose:** File type icon based on extension.

**Why It Exists:**
- **Visual Recognition:** Users identify files by icon
- **Extensibility:** Easy to add new file types
- **Fallback:** Default icon for unknown types
- **Consistency:** Same icon logic everywhere

**Supported Types:**
- Archives: zip, rar, 7z, tar, gz
- Video: mp4, mkv, avi, mov, wmv
- Documents: pdf
- Default: FileText

**Props:**
- `filename` - Filename to extract extension
- `className` - Additional styling

**Used In:**
- DownloadCard
- Future: File picker, detail view

### Common Components (`components/common/`)

Reusable UI primitives used throughout the application.

#### 1. EmptyState
**Purpose:** Placeholder view when no data exists.

**Why It Exists:**
- **UX Pattern:** Standard empty state pattern
- **Flexibility:** Custom icon, title, description, action
- **Consistency:** Same empty state across all pages
- **Accessibility:** Proper heading hierarchy

**Props:**
- `icon` - Custom icon (optional, defaults to Download icon)
- `title` - Heading text
- `description` - Helper text
- `action` - Optional action button/link
- `className` - Additional styling

**Used In:**
- Downloads page (when no downloads)
- Future: History page, settings pages

#### 2. SectionHeader
**Purpose:** Consistent page/section header with optional action.

**Why It Exists:**
- **DRY:** Eliminates repeated header markup
- **Consistency:** Same header structure everywhere
- **Flexibility:** Optional description and action button
- **Alignment:** Proper flex layout

**Props:**
- `title` - Main heading
- `description` - Optional subtitle
- `action` - Optional action element (button, link)
- `className` - Additional styling

**Used In:**
- Downloads page header
- Future: History, Settings, all pages

#### 3. ConfirmationDialog
**Purpose:** Generic confirmation dialog for user actions.

**Why It Exists:**
- **Reusability:** Used for delete, cancel, confirm actions
- **Variants:** Support for destructive (red) and default (primary) actions
- **Accessibility:** Keyboard navigation, focus management
- **Consistency:** Same dialog behavior everywhere

**Props:**
- `open` - Dialog visibility
- `onOpenChange` - Visibility change handler
- `title` - Dialog title
- `description` - Optional description
- `confirmLabel` - Confirm button text
- `cancelLabel` - Cancel button text
- `onConfirm` - Confirm action
- `onCancel` - Cancel action
- `variant` - 'default' | 'destructive'
- `className` - Additional styling

**Used In:**
- Future: Delete download confirmation, cancel download, etc.

## Design Principles

### 1. Composition Over Configuration
Components are designed to be composed together rather than configured with many props.

**Example:**
```tsx
<DownloadCard
  name="Ubuntu.iso"
  status="downloading"
  progress={43}
  speed={13003413}
/>
```
Composes: DownloadFileIcon + DownloadProgress + DownloadStatusBadge + DownloadSpeedLabel

### 2. Single Responsibility
Each component has one clear purpose:
- DownloadCard: Display a download
- DownloadProgress: Show progress bar
- DownloadStatusBadge: Show status badge

### 3. Props Over State
Components receive data via props, not internal state (except UI state like dialogs).

### 4. Theme Consistency
All components use the same design tokens:
- Colors: `bg-card`, `text-foreground`, `border-border`
- Spacing: Consistent padding/margins
- Typography: Consistent font sizes and weights
- Border radius: Consistent rounding

### 5. Accessibility First
- Semantic HTML
- Keyboard navigation
- Focus management
- ARIA labels where needed
- Color + text for status (not color alone)

## Component Hierarchy

```
Downloads Page
├── SectionHeader
│   ├── Title
│   ├── Description
│   └── Action (New Download button)
├── EmptyState (when no downloads)
│   ├── Icon
│   ├── Title
│   ├── Description
│   └── Action
└── DownloadCard[] (when downloads exist)
    ├── DownloadFileIcon
    ├── DownloadInfo
    │   ├── Name
    │   ├── URL
    │   └── DownloadProgress (conditional)
    └── DownloadMeta
        ├── DownloadSpeedLabel (conditional)
        └── DownloadStatusBadge
```

## Benefits

### 1. Maintainability
- **Single Source of Truth:** Each UI pattern defined once
- **Easy Updates:** Change component, updates everywhere
- **Clear Intent:** Component names communicate purpose

### 2. Developer Experience
- **Fast Development:** Compose existing components
- **Type Safety:** Full TypeScript support
- **IDE Support:** Autocomplete for props
- **Documentation:** Component names are self-documenting

### 3. Consistency
- **Visual:** Same patterns throughout app
- **Behavior:** Same interactions everywhere
- **Code:** Same implementation patterns

### 4. Testing
- **Isolated:** Test each component independently
- **Predictable:** Props in, UI out
- **Reusable:** Test once, use everywhere

### 5. Performance
- **Tree Shaking:** Only import what you use
- **Memoization:** Easy to add React.memo
- **Bundle Size:** Shared code reduces duplication

## Usage Examples

### Basic Download Card
```tsx
<DownloadCard
  id="1"
  name="Ubuntu.iso"
  url="https://releases.ubuntu.com/22.04/ubuntu.iso"
  status="downloading"
  progress={43}
  speed={13003413}
/>
```

### Empty State with Action
```tsx
<EmptyState
  icon={<CustomIcon />}
  title="No downloads"
  description="Add a download to get started"
  action={<Button>New Download</Button>}
/>
```

### Section Header
```tsx
<SectionHeader
  title="Downloads"
  description="Manage your downloads"
  action={<Button>New Download</Button>}
/>
```

### Confirmation Dialog
```tsx
<ConfirmationDialog
  open={showDialog}
  onOpenChange={setShowDialog}
  title="Delete download?"
  description="This action cannot be undone."
  confirmLabel="Delete"
  cancelLabel="Cancel"
  variant="destructive"
  onConfirm={handleDelete}
/>
```

## Future Enhancements

### Potential Components
- DownloadCardActions (pause/resume/cancel buttons)
- DownloadSizeLabel (formatted file size)
- DownloadETA (estimated time remaining)
- FileTypeIcon (more file types)
- Tooltip (for truncated text)
- ContextMenu (right-click actions)

### Potential Patterns
- Compound components (DownloadCard.Header, DownloadCard.Body)
- Render props for customization
- Component variants (compact, detailed)
- Theme switching (light/dark)

## Migration Guide

### Before (Hardcoded UI)
```tsx
<div className="bg-card border border-border rounded-lg p-4">
  <div className="flex items-center gap-4">
    <div className="w-10 h-10 rounded-lg bg-secondary">
      <Download className="w-5 h-5" />
    </div>
    <div>
      <h4>{download.name}</h4>
      <p>{download.url}</p>
    </div>
  </div>
</div>
```

### After (Component-based)
```tsx
<DownloadCard
  id={download.id}
  name={download.name}
  url={download.url}
  status={download.status}
  progress={download.progress}
  speed={download.speed}
/>
```

## Conclusion

This design system provides:
- **Reusability:** Components used throughout the app
- **Consistency:** Same look and feel everywhere
- **Maintainability:** Single source of truth for UI patterns
- **Scalability:** Easy to add new components
- **Developer Experience:** Fast, type-safe development

The system is production-ready and follows React and desktop application best practices.