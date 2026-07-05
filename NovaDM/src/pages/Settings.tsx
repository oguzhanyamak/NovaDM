import { FolderOpen, Save, Upload, RotateCcw } from 'lucide-react';
import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SectionHeader } from '../components/common/SectionHeader';
import { SettingsCard } from '../components/settings/SettingsCard';
import { SettingsInput } from '../components/settings/SettingsInput';
import { useSettingsStore } from '../store/settings';

// Theme type
type Theme = 'system' | 'dark' | 'light';

export function Settings() {
  const {
    settings,
    isLoading,
    error,
    loadSettings,
    updateSetting,
    exportSettings,
    importSettings,
    resetSettings,
  } = useSettingsStore();

  const [importError, setImportError] = useState<string | null>(null);

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const handleExport = async () => {
    try {
      const json = await exportSettings();
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'novadm-settings.json';
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Export failed:', err);
    }
  };

  const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      await importSettings(text);
      setImportError(null);
    } catch (err) {
      setImportError(err instanceof Error ? err.message : 'Import failed');
    }
  };

  const handleReset = async () => {
    if (confirm('Reset all settings to defaults? This cannot be undone.')) {
      await resetSettings();
    }
  };

  const handleBrowseFolder = async () => {
    try {
      const selected = await invoke<string | null>('select_folder', {});
      if (selected) {
        await updateSetting('download_path', selected);
      }
    } catch (err) {
      console.error('Failed to select folder:', err);
    }
  };

  // Handle number input changes
  const handleNumberChange = useCallback((key: string, value: string) => {
    const numValue = parseInt(value) || 0;
    updateSetting(key, numValue);
  }, [updateSetting]);

  // Handle text input changes
  const handleTextChange = useCallback((key: string, value: string) => {
    updateSetting(key, value);
  }, [updateSetting]);

  // Handle checkbox changes
  const handleCheckboxChange = useCallback((key: string, checked: boolean) => {
    updateSetting(key, checked);
  }, [updateSetting]);

  return (
    <div className="flex flex-col h-full">
      <header className="border-b border-border bg-card">
        <div className="px-8 py-4">
          <SectionHeader
            title="Settings"
            description="Configure your preferences"
          />
        </div>
      </header>

      <main className="flex-1 overflow-auto p-8">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-muted-foreground">Loading settings...</p>
          </div>
        ) : (
          <div className="max-w-2xl space-y-6">
            {/* Download Settings */}
            <SettingsCard
              title="Download Settings"
              description="Configure download behavior"
            >
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-foreground">Default Download Folder</label>
                  <div className="flex gap-2">
                    <SettingsInput
                      value={settings.download_path}
                      onChange={(value) => handleTextChange('download_path', value)}
                      placeholder="~/Downloads/NovaDM"
                    />
                    <button
                      onClick={handleBrowseFolder}
                      className="px-3 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80"
                      title="Browse"
                    >
                      <FolderOpen className="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-foreground">Maximum Concurrent Downloads</label>
                  <SettingsInput
                    value={settings.max_concurrent_downloads.toString()}
                    onChange={(value) => handleNumberChange('max_concurrent_downloads', value)}
                    type="number"
                    min={1}
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-foreground">Global Bandwidth Limit (KB/s)</label>
                  <SettingsInput
                    value={settings.bandwidth_limit_kb.toString()}
                    onChange={(value) => handleNumberChange('bandwidth_limit_kb', value)}
                    type="number"
                    min={0}
                    placeholder="0 = Unlimited"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium text-foreground">Auto Resume Interrupted Downloads</label>
                  <input
                    type="checkbox"
                    checked={settings.auto_resume}
                    onChange={(e) => handleCheckboxChange('auto_resume', e.target.checked)}
                    className="w-4 h-4 rounded border-border"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium text-foreground">Auto Retry Failed Downloads</label>
                  <input
                    type="checkbox"
                    checked={settings.auto_retry}
                    onChange={(e) => handleCheckboxChange('auto_retry', e.target.checked)}
                    className="w-4 h-4 rounded border-border"
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-foreground">Maximum Retry Attempts</label>
                  <SettingsInput
                    value={settings.max_retry_attempts.toString()}
                    onChange={(value) => handleNumberChange('max_retry_attempts', value)}
                    type="number"
                    min={1}
                  />
                </div>
              </div>
            </SettingsCard>

            {/* Appearance Settings */}
            <SettingsCard
              title="Appearance"
              description="Customize the look and feel"
            >
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-foreground">Theme</label>
                  <div className="flex gap-2">
                    {(['system', 'dark', 'light'] as Theme[]).map((theme) => (
                      <button
                        key={theme}
                        onClick={() => updateSetting('theme', theme)}
                        className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                          settings.theme === theme
                            ? 'bg-primary text-primary-foreground'
                            : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
                        }`}
                      >
                        {theme.charAt(0).toUpperCase() + theme.slice(1)}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </SettingsCard>

            {/* General Settings */}
            <SettingsCard
              title="General"
              description="Application preferences"
            >
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium text-foreground">Open NovaDM on Startup</label>
                  <input
                    type="checkbox"
                    checked={settings.open_on_startup}
                    onChange={(e) => handleCheckboxChange('open_on_startup', e.target.checked)}
                    className="w-4 h-4 rounded border-border"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium text-foreground">Check for Updates Automatically</label>
                  <input
                    type="checkbox"
                    checked={settings.auto_check_updates}
                    onChange={(e) => handleCheckboxChange('auto_check_updates', e.target.checked)}
                    className="w-4 h-4 rounded border-border"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium text-foreground">Enable Notifications</label>
                  <input
                    type="checkbox"
                    checked={settings.enable_notifications}
                    onChange={(e) => handleCheckboxChange('enable_notifications', e.target.checked)}
                    className="w-4 h-4 rounded border-border"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium text-foreground">Enable Browser Integration</label>
                  <input
                    type="checkbox"
                    checked={settings.enable_browser_integration}
                    onChange={(e) => handleCheckboxChange('enable_browser_integration', e.target.checked)}
                    className="w-4 h-4 rounded border-border"
                  />
                </div>
              </div>
            </SettingsCard>

            {/* Import/Export */}
            <SettingsCard
              title="Import / Export"
              description="Backup or restore your settings"
            >
              <div className="space-y-4">
                <div className="flex gap-2">
                  <button
                    onClick={handleExport}
                    className="flex items-center gap-2 px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80"
                  >
                    <Save className="w-4 h-4" />
                    Export Settings
                  </button>

                  <label className="flex items-center gap-2 px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 cursor-pointer">
                    <Upload className="w-4 h-4" />
                    Import Settings
                    <input
                      type="file"
                      accept=".json"
                      onChange={handleImport}
                      className="hidden"
                    />
                  </label>

                  <button
                    onClick={handleReset}
                    className="flex items-center gap-2 px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:bg-destructive/80"
                  >
                    <RotateCcw className="w-4 h-4" />
                    Reset to Defaults
                  </button>
                </div>

                {importError && (
                  <p className="text-sm text-destructive">{importError}</p>
                )}

                {error && (
                  <p className="text-sm text-destructive">{error}</p>
                )}
              </div>
            </SettingsCard>
          </div>
        )}
      </main>
    </div>
  );
}