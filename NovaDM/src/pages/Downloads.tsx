import { useState, useEffect } from 'react';
import { Plus } from 'lucide-react';
import { SectionHeader } from '../components/common/SectionHeader';
import { EmptyState } from '../components/common/EmptyState';
import { DownloadCard } from '../components/download/DownloadCard';
import { NewDownloadDialog } from '../components/NewDownloadDialog';
import { useDownloadsStore } from '../store/downloads';
import { eventService } from '../services/event';
import { downloadService } from '../services/download';
import type { DownloadProgressData, DownloadCompletedData, DownloadErrorData } from '../services/event';

export function Downloads() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const downloads = useDownloadsStore((state) => state.downloads);
  const addDownload = useDownloadsStore((state) => state.addDownload);
  const updateDownloadProgress = useDownloadsStore((state) => state.updateDownloadProgress);
  const completeDownload = useDownloadsStore((state) => state.completeDownload);
  const updateDownload = useDownloadsStore((state) => state.updateDownload);

  // Register event listeners
  useEffect(() => {
    const unlistenProgress = eventService.registerProgressListener(
      (data: DownloadProgressData) => {
        // Handle indeterminate progress (when content-length is unknown)
        const progress = data.progress ?? 0;
        updateDownloadProgress(data.id, progress, data.speed);
      }
    );

    const unlistenCompleted = eventService.registerCompletedListener(
      (data: DownloadCompletedData) => {
        completeDownload(data.id);
      }
    );

    const unlistenError = eventService.registerErrorListener(
      (data: DownloadErrorData) => {
        console.error('Download error:', data.message);
        updateDownload(data.id, {
          status: 'error',
          error: data.message,
        });
      }
    );

    return () => {
      unlistenProgress();
      unlistenCompleted();
      unlistenError();
    };
  }, [updateDownloadProgress, completeDownload, updateDownload]);

  const handleNewDownload = async () => {
    // Create a demo download entry
    const demoDownload = {
      id: 'demo-download',
      name: 'Demo File.zip',
      url: 'https://example.com/demo.zip',
      status: 'downloading' as const,
      progress: 0,
      size: 100000000, // 100 MB
      downloaded: 0,
      speed: 0,
      createdAt: new Date(),
    };

    addDownload(demoDownload);
    setIsDialogOpen(false);

    // Start the real download
    try {
      await downloadService.startDownload({
        url: demoDownload.url,
        filename: demoDownload.name,
        saveLocation: '~/Downloads/NovaDM',
      });
    } catch (err) {
      console.error('Failed to start download:', err);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <header className="border-b border-border bg-card">
        <div className="px-8 py-4">
          <SectionHeader
            title="Downloads"
            description="Manage your downloads"
            action={
              <button 
                onClick={handleNewDownload}
                className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
              >
                <Plus className="w-4 h-4" />
                New Download
              </button>
            }
          />
        </div>
      </header>

      <main className="flex-1 overflow-auto p-8">
        {downloads.length === 0 ? (
          <EmptyState
            title="No downloads yet"
            description="Start by adding a new download. Your downloads will appear here."
          />
        ) : (
          <div className="space-y-3">
            {downloads.map((download) => (
              <DownloadCard
                key={download.id}
                id={download.id}
                name={download.name}
                url={download.url}
                status={download.status}
                progress={download.progress}
                speed={download.speed}
              />
            ))}
          </div>
        )}
      </main>

      <NewDownloadDialog 
        open={isDialogOpen} 
        onOpenChange={setIsDialogOpen} 
      />
    </div>
  );
}
