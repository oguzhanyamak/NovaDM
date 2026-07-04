import { create } from 'zustand';
import { Download, DownloadHistory } from '../types';

interface DownloadsState {
  downloads: Download[];
  history: DownloadHistory[];
  currentView: 'downloads' | 'history' | 'settings';
  setCurrentView: (view: 'downloads' | 'history' | 'settings') => void;
  addDownload: (download: Download) => void;
  removeDownload: (id: string) => void;
  updateDownload: (id: string, updates: Partial<Download>) => void;
  updateDownloadProgress: (id: string, progress: number | null, downloadedBytes: number, totalBytes: number | null, speed: number) => void;
  completeDownload: (id: string) => void;
  markAsCancelled: (id: string) => void;
  queueDownload: (id: string, position: number) => void;
  startQueuedDownload: (id: string) => void;
  updateQueuePosition: (id: string, position: number) => void;
}

export const useDownloadsStore = create<DownloadsState>((set) => ({ 
  downloads: [],
  history: [],
  currentView: 'downloads',
  setCurrentView: (view) => set({ currentView: view }),
  addDownload: (download) =>
    set((state) => ({
      downloads: [...state.downloads, download],
    })),
  removeDownload: (id) =>
    set((state) => ({
      downloads: state.downloads.filter((d) => d.id !== id),
    })),
  updateDownload: (id, updates) =>
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id ? { ...d, ...updates } : d
      ),
    })),
  updateDownloadProgress: (id, progress, downloadedBytes, totalBytes, speed) =>
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id
          ? {
              ...d,
              progress: progress ?? d.progress,
              downloaded: downloadedBytes,
              size: totalBytes ?? d.size,
              speed,
            }
          : d
      ),
    })),
  completeDownload: (id) =>
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id
          ? {
              ...d,
              status: 'completed' as const,
              progress: 100,
              speed: 0,
              completedAt: new Date(),
            }
          : d
      ),
    })),

  markAsCancelled: (id) =>
    set((state) => ({
      downloads: state.downloads.filter((d) => d.id !== id),
    })),

  queueDownload: (id, position) =>
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id
          ? {
              ...d,
              status: 'pending' as const,
              queuePosition: position,
            }
          : d
      ),
    })),

  startQueuedDownload: (id) =>
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id
          ? {
              ...d,
              status: 'downloading' as const,
              queuePosition: undefined,
            }
          : d
      ),
    })),

  updateQueuePosition: (id, position) =>
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id
          ? {
              ...d,
              queuePosition: position,
            }
          : d
      ),
    })),
}));
