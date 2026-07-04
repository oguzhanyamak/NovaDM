// Download service for communicating with Tauri backend
// All Tauri communication happens through this service only

import { invoke } from '@tauri-apps/api/core';
import { Download, DownloadHistory } from '../types';

export interface StartDownloadParams {
  url: string;
  filename: string;
  saveLocation: string;
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

  async cancelDownload(id: string): Promise<void> {
    // Placeholder - will call Tauri command
    await invoke('cancel_download', { id });
  },
};
