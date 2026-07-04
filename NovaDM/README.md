# NovaDM - Modern Download Manager

A modern, feature-rich download manager built with Tauri v2, React, TypeScript, and Tailwind CSS.

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **UI Framework**: Tailwind CSS + shadcn/ui components
- **State Management**: Zustand
- **Icons**: Lucide React

## Project Structure

```
NovaDM/
├── src/                          # Frontend source code
│   ├── components/               # Reusable UI components
│   │   ├── Sidebar.tsx          # Navigation sidebar with app branding
│   │   └── index.ts             # Barrel export for components
│   │
│   ├── pages/                    # Page components
│   │   ├── Downloads.tsx        # Main downloads view with empty state
│   │   ├── History.tsx          # Download history view
│   │   ├── Settings.tsx         # Settings view (placeholder)
│   │   └── index.ts             # Barrel export for pages
│   │
│   ├── hooks/                    # Custom React hooks
│   │   └── use-downloads.ts     # Download management hook
│   │
│   ├── services/                 # API service layer
│   │   └── download.ts          # Download API service (placeholder)
│   │
│   ├── store/                    # State management
│   │   └── downloads.ts         # Zustand store for downloads
│   │
│   ├── types/                    # TypeScript type definitions
│   │   └── index.ts             # Download, History, and View types
│   │
│   ├── lib/                      # Utility functions
│   │   └── utils.ts             # cn() helper for class names
│   │
│   ├── App.tsx                   # Main app component with routing
│   ├── App.css                   # Tailwind CSS imports and theme
│   └── main.tsx                  # React entry point
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs              # Application entry point
│   │   └── lib.rs               # Tauri command registration
│   │
│   ├── api/                      # Tauri API commands
│   │   └── mod.rs               # Command handlers for downloads
│   │
│   ├── download/                 # Download management logic
│   │   └── mod.rs               # DownloadManager and types
│   │
│   ├── storage/                  # Persistent storage
│   │   └── mod.rs               # StorageManager and config
│   │
│   ├── utils/                    # Rust utility functions
│   │   └── mod.rs               # Helper functions (formatting, IDs)
│   │
│   └── Cargo.toml               # Rust dependencies
│
├── package.json                  # Node.js dependencies
├── vite.config.ts                # Vite configuration
├── tsconfig.json                 # TypeScript configuration
└── index.html                    # HTML entry point
```

## Architecture

### Frontend Architecture

The frontend follows a clean separation of concerns:

- **components/**: Reusable UI components (Sidebar, etc.)
- **pages/**: Route-level components (Downloads, History, Settings)
- **hooks/**: Custom React hooks for business logic
- **services/**: API communication layer with Tauri backend
- **store/**: Zustand state management
- **types/**: TypeScript interfaces and type definitions
- **lib/**: Shared utility functions

### Backend Architecture

The Rust backend is organized into focused modules:

- **api/**: Tauri command handlers that expose functionality to frontend
- **download/**: Core download management logic (placeholder for now)
- **storage/**: Configuration and persistent data management
- **utils/**: Shared utility functions (formatting, ID generation, etc.)

## Features

- ✅ Modern dark theme with purple accent colors
- ✅ Clean, minimal UI with sidebar navigation
- ✅ Empty state for downloads list
- ✅ Zustand state management (empty store ready for implementation)
- ✅ Type-safe with TypeScript
- ✅ Responsive layout with Tailwind CSS
- ✅ Lucide icons for consistent iconography
- ✅ Placeholder backend structure ready for download logic

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

The application is structured for clean architecture:

1. **Frontend and backend are completely separated**
2. **Communication happens through Tauri commands** (defined in `src-tauri/api/`)
3. **State management is centralized** in Zustand store
4. **Types are shared** between frontend and backend concepts
5. **No download logic implemented yet** - structure is ready for it

## Next Steps

- Implement download logic in `src-tauri/download/`
- Add actual Tauri command implementations
- Connect frontend services to backend API
- Add download creation dialog
- Implement progress tracking
- Add settings persistence

## License

MIT