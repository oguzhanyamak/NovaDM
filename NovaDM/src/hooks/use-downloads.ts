// Custom hooks for download management
// Will be implemented with actual download logic

import { useCallback } from 'react';
import { useDownloadsStore } from '../store/downloads';
import { Download } from '../types';

export function useDownloads() {
  const downloads = useDownloadsStore((state) => state.downloads);
  const addDownload = useDownloadsStore((state) => state.addDownload);
  const removeDownload = useDownloadsStore((state) => state.removeDownload);
  const updateDownload = useDownloadsStore((state) => state.updateDownload);

  const addNewDownload = useCallback((download: Download) => {
    addDownload(download);
  }, [addDownload]);

  const deleteDownload = useCallback((id: string) => {
    removeDownload(id);
  }, [removeDownload]);

  const modifyDownload = useCallback((id: string, updates: Partial<Download>) => {
    updateDownload(id, updates);
  }, [updateDownload]);

  return {
    downloads,
    addDownload: addNewDownload,
    removeDownload: deleteDownload,
    updateDownload: modifyDownload,
  };
}