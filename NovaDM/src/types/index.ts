export interface Download {
  id: string;
  name: string;
  url: string;
  status: 'pending' | 'downloading' | 'paused' | 'recovered' | 'completed' | 'error' | 'cancelled';
  progress: number;
  size: number;
  downloaded: number;
  speed: number;
  createdAt: Date;
  completedAt?: Date;
  error?: string;
  queuePosition?: number;
}

export type HistoryStatus = 'completed' | 'failed' | 'cancelled';

export interface HistoryEntry {
  id: string;
  filename: string;
  url: string;
  output_path: string;
  status: HistoryStatus;
  file_size: number;
  average_speed: number;
  started_at: number;
  completed_at: number;
  duration: number;
  checksum?: string;
}

export type ViewType = 'downloads' | 'history' | 'settings';

export type HistoryFilter = 'all' | 'completed' | 'failed' | 'cancelled';

export type HistorySort = 'newest' | 'oldest' | 'largest' | 'smallest' | 'alphabetical';

// Settings types
export type Theme = 'system' | 'dark' | 'light';

export interface AppSettings {
  // Download Settings
  download_path: string;
  max_concurrent_downloads: number;
  bandwidth_limit_kb: number;
  auto_resume: boolean;
  auto_retry: boolean;
  max_retry_attempts: number;

  // Appearance
  theme: Theme;

  // General
  open_on_startup: boolean;
  auto_check_updates: boolean;
  enable_notifications: boolean;
  enable_browser_integration: boolean;
}
