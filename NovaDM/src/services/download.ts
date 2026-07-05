// Download service for communicating with Tauri backend
// All Tauri communication happens through this service only

import { invoke } from '@tauri-apps/api/core';
import { Download, DownloadHistory } from '../types';

export interface StartDownloadParams {
  url: string;
  filename: string;
  saveLocation: string;
}

export interface RecoveryCandidate {
  download_id: string;
  filename: string;
  url: string;
  partial_path: string;
  downloaded_bytes: number;
  total_bytes: number | null;
  resume_supported: boolean;
  created_at: number;
}

export const downloadService = {
  async startDownload(params: StartDownloadParams): Promise<void> {
    // Only this service calls Tauri commands
    await invoke('start_download', {
      url: params.url,
      filename: params.filename,
      save_location: params.saveLocation,
    });
  },

  async cancelDownload(id: string): Promise<void> {
    // Only this service calls Tauri commands
    await invoke('cancel_download', { id });
  },

  async openFile(path: string): Promise<void> {
    // Only this service calls Tauri commands
    await invoke('open_file', { path });
  },

  async showInFolder(path: string): Promise<void> {
    // Only this service calls Tauri commands
    await invoke('show_in_folder', { path });
  },

  async retryDownload(id: string): Promise<void> {
    // Only this service calls Tauri commands
    await invoke('retry_download', { id });
  },

  async getDownloads(): Promise<Download[]> {
    // Placeholder - will call Tauri command
    return [];
  },

  async getHistory(): Promise<DownloadHistory[]> {
    // Placeholder - will call Tauri command
    return [];
  },

  async pauseDownload(id: string): Promise<void> {
    // Placeholder - will call Tauri command
    await invoke('pause_download', { id });
  },

  async resumeDownload(id: string): Promise<void> {
    // Placeholder - will call Tauri command
    await invoke('resume_download', { id });
  },

  async getRecoveryCandidates(): Promise<RecoveryCandidate[]> {
    // Only this service calls Tauri commands
    return await invoke('get_recovery_candidates', {});
  },
};