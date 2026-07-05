import { create } from 'zustand';
import { AppSettings } from '../types';
import { settingsService } from '../services/settings';

interface SettingsState {
  // Settings data
  settings: AppSettings;
  // Loading state
  isLoading: boolean;
  // Error state
  error: string | null;

  // Actions
  loadSettings: () => Promise<void>;
  updateSetting: (key: string, value: unknown) => Promise<void>;
  saveSettings: () => Promise<void>;
  exportSettings: () => Promise<string>;
  importSettings: (json: string) => Promise<void>;
  resetSettings: () => Promise<void>;
}

const defaultSettings: AppSettings = {
  download_path: '~/Downloads/NovaDM',
  max_concurrent_downloads: 3,
  bandwidth_limit_kb: 0,
  auto_resume: true,
  auto_retry: true,
  max_retry_attempts: 3,
  theme: 'system',
  open_on_startup: false,
  auto_check_updates: true,
  enable_notifications: true,
  enable_browser_integration: false,
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: defaultSettings,
  isLoading: false,
  error: null,

  loadSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await settingsService.getSettings();
      set({ settings, isLoading: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to load settings',
        isLoading: false 
      });
    }
  },

  updateSetting: async (key: string, value: unknown) => {
    // Don't set isLoading to avoid scroll jump
    try {
      const settings = await settingsService.updateSetting(key, value);
      set({ settings });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to update setting'
      });
    }
  },

  saveSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      await settingsService.saveSettings(get().settings);
      set({ isLoading: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to save settings',
        isLoading: false 
      });
    }
  },

  exportSettings: async () => {
    try {
      return await settingsService.exportSettings();
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to export settings' 
      });
      throw error;
    }
  },

  importSettings: async (json: string) => {
    set({ isLoading: true, error: null });
    try {
      const settings = await settingsService.importSettings(json);
      set({ settings, isLoading: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to import settings',
        isLoading: false 
      });
    }
  },

  resetSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await settingsService.resetSettings();
      set({ settings, isLoading: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to reset settings',
        isLoading: false 
      });
    }
  },
}));