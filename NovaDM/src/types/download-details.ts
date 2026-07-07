// Download Details types
// These types represent the detailed information for a download

import { Download, HistoryEntry } from './index';

// Timeline event types
export type TimelineEventType = 'created' | 'started' | 'paused' | 'resumed' | 'completed' | 'failed' | 'cancelled';

export interface TimelineEvent {
  type: TimelineEventType;
  timestamp: number;
}

// Technical information from HTTP response
export interface TechnicalInfo {
  http_status?: number;
  server?: string;
  content_type?: string;
  etag?: string;
  last_modified?: string;
  accept_ranges?: string;
  connection_count: number;
}

// Performance metrics
export interface PerformanceMetrics {
  current_speed: number;
  average_speed: number;
  peak_speed: number;
  estimated_time_remaining?: number;
  elapsed_time: number;
  bandwidth_limit: number;
}

// Download details - combines all information
export interface DownloadDetails {
  // General
  id: string;
  filename: string;
  status: Download['status'] | HistoryEntry['status'];
  output_folder: string;
  output_file: string;
  url: string;
  file_size: number;
  downloaded_bytes: number;
  remaining_bytes: number;
  progress: number;
  resume_supported: boolean;
  checksum?: string;

  // Performance
  performance: PerformanceMetrics;

  // Timeline
  timeline: TimelineEvent[];

  // Technical
  technical: TechnicalInfo;
}

// Selection state
export interface DownloadSelection {
  downloadId: string | null;
  source: 'downloads' | 'history' | null;
}