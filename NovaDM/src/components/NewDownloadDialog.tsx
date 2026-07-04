import { useState, useEffect, useCallback } from 'react';
import { cn } from '../lib/utils';
import { Download as DownloadIcon, FolderOpen } from 'lucide-react';
import { downloadService } from '../services/download';
import { useDownloadsStore } from '../store/downloads';
import type { Download } from '../types';

interface NewDownloadDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewDownloadDialog({ open, onOpenChange }: NewDownloadDialogProps) {
  const [url, setUrl] = useState('');
  const [filename, setFilename] = useState('');
  const [saveLocation, setSaveLocation] = useState('');
  const [urlError, setUrlError] = useState('');
  const [urlTouched, setUrlTouched] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const addDownload = useDownloadsStore((state) => state.addDownload);

  // Set default save location to Downloads folder
  useEffect(() => {
    if (open) {
      const downloadsPath = '~/Downloads/NovaDM';
      setSaveLocation(downloadsPath);
      setUrl('');
      setFilename('');
      setUrlError('');
      setUrlTouched(false);
    }
  }, [open]);

  // Auto-suggest filename from URL
  useEffect(() => {
    if (url && !filename) {
      const suggestedFilename = extractFilenameFromUrl(url);
      if (suggestedFilename) {
        setFilename(suggestedFilename);
      }
    }
  }, [url, filename]);

  const extractFilenameFromUrl = useCallback((urlString: string): string => {
    try {
      const urlObj = new URL(urlString);
      const pathname = urlObj.pathname;
      const segments = pathname.split('/');
      const lastSegment = segments[segments.length - 1];
      
      if (lastSegment && lastSegment.includes('.')) {
        return decodeURIComponent(lastSegment);
      }
      
      return '';
    } catch {
      return '';
    }
  }, []);

  const validateUrl = useCallback((urlString: string): string => {
    if (!urlString.trim()) {
      return 'URL is required';
    }
    
    try {
      const urlObj = new URL(urlString);
      if (!['http:', 'https:'].includes(urlObj.protocol)) {
        return 'URL must start with http:// or https://';
      }
      return '';
    } catch {
      return 'Invalid URL format';
    }
  }, []);

  const handleUrlChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setUrl(value);
    
    if (urlTouched) {
      setUrlError(validateUrl(value));
    }
  };

  const handleUrlBlur = () => {
    setUrlTouched(true);
    setUrlError(validateUrl(url));
  };

  const handleUrlKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      const error = validateUrl(url);
      if (!error) {
        handleDownload();
      }
    }
  };

  const handleDownload = async () => {
    const error = validateUrl(url);
    if (error) {
      setUrlError(error);
      setUrlTouched(true);
      return;
    }

    setIsSubmitting(true);
    
    // Create a download entry
    const newDownload: Download = {
      id: `download-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      name: filename,
      url,
      status: 'downloading',
      progress: 0,
      size: 0,
      downloaded: 0,
      speed: 0,
      createdAt: new Date(),
    };

    // Add to store
    addDownload(newDownload);
    
    try {
      await downloadService.startDownload({
        url,
        filename,
        saveLocation,
      });
      onOpenChange(false);
    } catch (err) {
      console.error('Failed to start download:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCancel = () => {
    onOpenChange(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onOpenChange(false);
    }
  };

  const isFormValid = url.trim() && !validateUrl(url) && filename.trim();

  // Don't render anything if dialog is not open
  if (!open) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center" onKeyDown={handleKeyDown}>
      {/* Backdrop */}
      <div 
        className="fixed inset-0 bg-black/50 backdrop-blur-sm"
        onClick={() => onOpenChange(false)}
      />
      
      {/* Dialog Content */}
      <div className="relative z-50 w-full max-w-lg rounded-lg border border-border bg-card p-6 shadow-lg" onClick={(e) => e.stopPropagation()}>
        <div className="space-y-4">
          <h2 className="text-2xl font-bold text-foreground">
            New Download
          </h2>

          <div className="grid gap-4 py-4">
            {/* URL Field */}
            <div className="grid gap-2">
              <label htmlFor="url" className="text-sm font-medium text-foreground">
                URL
              </label>
              <input
                id="url"
                type="text"
                value={url}
                onChange={handleUrlChange}
                onBlur={handleUrlBlur}
                onKeyDown={handleUrlKeyDown}
                placeholder="https://example.com/file.zip"
                className={cn(
                  'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground',
                  'placeholder:text-muted-foreground',
                  'focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
                  'disabled:cursor-not-allowed disabled:opacity-50',
                  urlTouched && urlError && 'border-red-500 focus:ring-red-500'
                )}
                autoFocus
              />
              {urlTouched && urlError && (
                <p className="text-sm text-red-500">{urlError}</p>
              )}
            </div>

            {/* Save Location Field */}
            <div className="grid gap-2">
              <label htmlFor="saveLocation" className="text-sm font-medium text-foreground">
                Save location
              </label>
              <div className="flex gap-2">
                <input
                  id="saveLocation"
                  type="text"
                  value={saveLocation}
                  readOnly
                  className="flex h-10 flex-1 rounded-md border border-input bg-muted px-3 py-2 text-sm text-muted-foreground"
                />
                <button
                  type="button"
                  onClick={() => alert('Folder picker not implemented yet')}
                  className="h-10 w-10 shrink-0 rounded-md border border-input bg-background hover:bg-accent hover:text-accent-foreground inline-flex items-center justify-center"
                >
                  <FolderOpen className="h-4 w-4" />
                </button>
              </div>
            </div>

            {/* Filename Field */}
            <div className="grid gap-2">
              <label htmlFor="filename" className="text-sm font-medium text-foreground">
                Filename
              </label>
              <input
                id="filename"
                type="text"
                value={filename}
                onChange={(e) => setFilename(e.target.value)}
                placeholder="filename.zip"
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
              />
            </div>
          </div>

          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={handleCancel}
              disabled={isSubmitting}
              className="px-4 py-2 rounded-md border border-input bg-background hover:bg-accent hover:text-accent-foreground disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleDownload}
              disabled={!isFormValid || isSubmitting}
              className="px-4 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center gap-2"
            >
              <DownloadIcon className="h-4 w-4" />
              {isSubmitting ? 'Starting...' : 'Download'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}