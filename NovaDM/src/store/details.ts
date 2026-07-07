// Download Details Store
// Manages the selected download and its details

import { create } from 'zustand';
import { Download } from '../types';
import { HistoryEntry } from '../types';
import { DownloadDetails, DownloadSelection } from '../types/download-details';

interface DetailsState {
  // Selection
  selection: DownloadSelection;
  // Details for the selected download
  details: DownloadDetails | null;
  // Panel collapsed state (persisted)
  isCollapsed: boolean;

  // Actions
  selectDownload: (id: string, source: 'downloads' | 'history') => void;
  clearSelection: () => void;
  setDetails: (details: DownloadDetails | null) => void;
  toggleCollapsed: () => void;
  setCollapsed: (collapsed: boolean) => void;
}

export const useDetailsStore = create<DetailsState>((set) => ({
  selection: { downloadId: null, source: null },
  details: null,
  isCollapsed: false,

  selectDownload: (id, source) =>
    set({
      selection: { downloadId: id, source },
    }),

  clearSelection: () =>
    set({
      selection: { downloadId: null, source: null },
      details: null,
    }),

  setDetails: (details) => set({ details }),

  toggleCollapsed: () =>
    set((state) => ({
      isCollapsed: !state.isCollapsed,
    })),

  setCollapsed: (collapsed) => set({ isCollapsed: collapsed }),
}));

// Helper to get download by id
export function getDownloadById(downloads: Download[], id: string): Download | undefined {
  return downloads.find((d) => d.id === id);
}

// Helper to get history entry by id
export function getHistoryEntryById(history: HistoryEntry[], id: string): HistoryEntry | undefined {
  return history.find((h) => h.id === id);
}