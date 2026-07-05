// Settings service for communicating with Tauri backend
// All Tauri communication happens through this service only

import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types';

export const settingsService = {
  async getSettings(): Promise<AppSettings> {
    return await invoke<AppSettings>('get_settings', {});
  },

  async saveSettings(settings: AppSettings): Promise<void> {
    await invoke('save_settings', { settings });
  },

  async updateSetting(key: string, value: unknown): Promise<AppSettings> {
    return await invoke<AppSettings>('update_setting', { key, value });
  },

  async exportSettings(): Promise<string> {
    return await invoke<string>('export_settings', {});
  },

  async importSettings(json: string): Promise<AppSettings> {
    return await invoke<AppSettings>('import_settings', { json });
  },

  async resetSettings(): Promise<AppSettings> {
    return await invoke<AppSettings>('reset_settings', {});
  },

  async selectFolder(): Promise<string | null> {
    const result = await invoke<string | null>('select_folder', {});
    return result;
  },
};