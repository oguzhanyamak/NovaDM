# NovaDM - Modern Download Manager

A modern, feature-rich download manager built with Tauri v2, React, TypeScript, and Tailwind CSS.

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **UI Framework**: Tailwind CSS + shadcn/ui components
- **State Management**: Zustand
- **Icons**: Lucide React

## Features (Alpha)

- ✅ HTTP download with streaming (memory efficient)
- ✅ Progress events with real-time updates
- ✅ Download cancellation with CancellationToken
- ✅ File conflict resolution (auto-rename)
- ✅ Open file after completion
- ✅ Show in folder
- ✅ Structured error handling
- ✅ UUID-based download IDs
- ✅ Clean architecture
- ✅ Unit tests

## Project Structure

```
NovaDM/
├── src/                          # Frontend source code
│   ├── components/               # Reusable UI components
│   │   ├── download/             # Download-specific components
│   │   │   ├── DownloadCard.tsx
│   │   │   ├── DownloadProgress.tsx
│   │   │   ├── DownloadSpeedLabel.tsx
│   │   │   ├── DownloadStatusBadge.tsx
│   │   │   └── DownloadFileIcon.tsx
│   │   └── common/               # Shared components
│   │       ├── EmptyState.tsx
│   │       ├── SectionHeader.tsx
│   │       └── ConfirmationDialog.tsx
│   ├── pages/                    # Page components
│   │   ├── Downloads.tsx
│   │   ├── History.tsx
│   │   ├── Settings.tsx
│   │   └── index.ts
│   ├── hooks/                    # Custom React hooks
│   ├── services/                 # API service layer
│   │   ├── download.ts           # Download API service
│   │   └── event.ts              # Event listener service
│   ├── store/                    # State management
│   │   └── downloads.ts          # Zustand store
│   ├── types/                    # TypeScript type definitions
│   │   └── index.ts
│   ├── lib/                      # Utility functions
│   │   └── utils.ts
│   ├── App.tsx
│   ├── App.css
│   └── main.tsx
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── api/                  # Tauri command handlers
│   │   │   └── mod.rs
│   │   ├── core/                 # App state and config
│   │   │   ├── mod.rs
│   │   │   ├── config.rs
│   │   │   ├── constants.rs
│   │   │   ├── errors.rs
│   │   │   └── events.rs
│   │   ├── download/             # Download management logic
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs        # DownloadManager singleton
│   │   │   ├── models.rs
│   │   │   ├── errors.rs
│   │   │   ├── utils.rs          # File conflict resolution
│   │   │   ├── worker.rs
│   │   │   ├── queue.rs
│   │   │   └── chunk.rs
│   │   ├── storage/              # Persistent storage
│   │   │   └── settings.rs
│   │   ├── utils/                # Rust utility functions
│   │   │   └── formatting.rs
│   │   └── lib.rs
│   └── Cargo.toml
│
├── docs/                         # Documentation
│   ├── ARCHITECTURE.md
│   └── ROADMAP.md
│
├── package.json
├── vite.config.ts
├── tsconfig.json
└── index.html
```

## Architecture

### Backend

The Rust backend uses a singleton pattern for the DownloadManager:

- **DownloadManager**: Single instance managed by Tauri, handles all downloads
- **HashMap<String, DownloadHandle>**: O(1) lookup for active downloads
- **CancellationToken**: Graceful cancellation without thread interruption
- **Streaming**: Memory-efficient downloads with reqwest and BufWriter

### Frontend

- **EventService**: Centralized event listener (only this service talks to Tauri)
- **DownloadService**: Tauri command wrapper (only this service invokes commands)
- **Zustand Store**: Centralized state management
- **Clean separation**: UI never directly calls Tauri

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### Installation

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run build
```

## Development

```bash
# Run Rust tests
cd src-tauri && cargo test

# Run frontend build
npm run build
```

## License

MIT