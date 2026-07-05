// History service for communicating with Tauri backend
// All Tauri communication happens through this service only

import { invoke } from '@tauri-apps/api/core';
import { HistoryEntry } from '../types';

export const historyService = {
  async getHistory(): Promise<HistoryEntry[]> {
    return await invoke<HistoryEntry[]>('get_history', {});
  },

  async deleteEntry(id: string): Promise<void> {
    await invoke('delete_history_entry', { id });
  },

  async deleteEntries(ids: string[]): Promise<void> {
    await invoke('delete_history_entries', { ids });
  },

  async clearHistory(): Promise<void> {
    await invoke('clear_history', {});
  },
};