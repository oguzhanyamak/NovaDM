// Download service for communicating with Tauri backend
// Will be implemented with actual API calls

import { invoke } from '@tauri-apps/api/core';
import { Download, DownloadHistory } from '../types';

export const downloadService = {
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