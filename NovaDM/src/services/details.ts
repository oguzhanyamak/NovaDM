// Download Details Service
// Builds detailed information from download data

import { Download } from '../types';
import { HistoryEntry } from '../types';
import { DownloadDetails, TimelineEvent, PerformanceMetrics, TechnicalInfo } from '../types/download-details';

// Format bytes to human readable string
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

// Format speed to human readable string
function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`;
}

// Format time duration
function formatDuration(seconds: number): string {
  if (seconds < 60) return `${Math.round(seconds)}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m ${Math.round(seconds % 60)}s`;
  return `${Math.round(seconds / 3600)}h ${Math.round((seconds % 3600) / 60)}m`;
}

// Build timeline from download
function buildTimeline(download: Download): TimelineEvent[] {
  const events: TimelineEvent[] = [];
  const now = Date.now();

  // Created
  events.push({ type: 'created', timestamp: now - (download.size - download.downloaded) * 10 }); // Estimate

  // Started (if not pending)
  if (download.status !== 'pending') {
    events.push({ type: 'started', timestamp: now - 10000 });
  }

  // Status-specific events
  if (download.status === 'paused') {
    events.push({ type: 'paused', timestamp: now - 5000 });
  } else if (download.status === 'completed') {
    events.push({ type: 'completed', timestamp: now });
  } else if (download.status === 'error') {
    events.push({ type: 'failed', timestamp: now });
  } else if (download.status === 'cancelled') {
    events.push({ type: 'cancelled', timestamp: now });
  }

  return events;
}

// Build timeline from history entry
function buildHistoryTimeline(entry: HistoryEntry): TimelineEvent[] {
  const events: TimelineEvent[] = [];

  // Started
  events.push({ type: 'started', timestamp: entry.started_at });

  // Completed/Failed/Cancelled
  if (entry.status === 'completed') {
    events.push({ type: 'completed', timestamp: entry.completed_at });
  } else if (entry.status === 'failed') {
    events.push({ type: 'failed', timestamp: entry.completed_at });
  } else if (entry.status === 'cancelled') {
    events.push({ type: 'cancelled', timestamp: entry.completed_at });
  }

  return events;
}

// Build performance metrics
function buildPerformance(download: Download, bandwidthLimit: number): PerformanceMetrics {
  const elapsed = download.completedAt 
    ? (download.completedAt.getTime() - download.createdAt.getTime()) / 1000 
    : (Date.now() - download.createdAt.getTime()) / 1000;

  const averageSpeed = elapsed > 0 ? download.downloaded / elapsed : 0;
  const estimatedTimeRemaining = download.speed > 0 
    ? (download.size - download.downloaded) / download.speed 
    : undefined;

  return {
    current_speed: download.speed,
    average_speed: averageSpeed,
    peak_speed: download.speed * 1.2, // Estimate
    estimated_time_remaining: estimatedTimeRemaining,
    elapsed_time: elapsed,
    bandwidth_limit: bandwidthLimit,
  };
}

// Build performance from history entry
function buildHistoryPerformance(entry: HistoryEntry, bandwidthLimit: number): PerformanceMetrics {
  return {
    current_speed: 0,
    average_speed: entry.average_speed,
    peak_speed: entry.average_speed,
    elapsed_time: entry.duration,
    bandwidth_limit: bandwidthLimit,
  };
}

// Build technical info
function buildTechnical(_download: Download): TechnicalInfo {
  return {
    connection_count: 1,
    // These would come from actual download metadata
    http_status: undefined,
    server: undefined,
    content_type: undefined,
    etag: undefined,
    last_modified: undefined,
    accept_ranges: undefined,
  };
}

// Build technical info from history
function buildHistoryTechnical(): TechnicalInfo {
  return {
    connection_count: 1,
  };
}

export const detailsService = {
  // Build details from active download
  buildFromDownload(download: Download, bandwidthLimit: number = 0): DownloadDetails {
    const outputPath = download.name;
    const outputFolder = outputPath.substring(0, outputPath.lastIndexOf('/') + 1) || '/';

    return {
      id: download.id,
      filename: download.name,
      status: download.status,
      output_folder: outputFolder,
      output_file: outputPath,
      url: download.url,
      file_size: download.size,
      downloaded_bytes: download.downloaded,
      remaining_bytes: download.size - download.downloaded,
      progress: download.progress,
      resume_supported: true, // Would come from actual metadata
      timeline: buildTimeline(download),
      performance: buildPerformance(download, bandwidthLimit),
      technical: buildTechnical(download),
    };
  },

  // Build details from history entry
  buildFromHistory(entry: HistoryEntry, bandwidthLimit: number = 0): DownloadDetails {
    const outputPath = entry.output_path;
    const outputFolder = outputPath.substring(0, outputPath.lastIndexOf('/') + 1) || '/';

    return {
      id: entry.id,
      filename: entry.filename,
      status: entry.status,
      output_folder: outputFolder,
      output_file: outputPath,
      url: entry.url,
      file_size: entry.file_size,
      downloaded_bytes: entry.file_size,
      remaining_bytes: 0,
      progress: 100,
      resume_supported: false,
      checksum: entry.checksum,
      timeline: buildHistoryTimeline(entry),
      performance: buildHistoryPerformance(entry, bandwidthLimit),
      technical: buildHistoryTechnical(),
    };
  },

  // Format helpers
  formatBytes,
  formatSpeed,
  formatDuration,
};