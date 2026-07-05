import { useState, useEffect } from 'react';
import { Plus } from 'lucide-react';
import { SectionHeader } from '../components/common/SectionHeader';
import { EmptyState } from '../components/common/EmptyState';
import { DownloadCard } from '../components/download/DownloadCard';
import { NewDownloadDialog } from '../components/NewDownloadDialog';
import { useDownloadsStore } from '../store/downloads';
import { eventService } from '../services/event';
import type { DownloadProgressData, DownloadCompletedData, DownloadErrorData, DownloadCancelledData, DownloadQueuedData, DownloadStartedData, DownloadRetryData } from '../services/event';

export function Downloads() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const downloads = useDownloadsStore((state) => state.downloads);
  const updateDownloadProgress = useDownloadsStore((state) => state.updateDownloadProgress);
  const completeDownload = useDownloadsStore((state) => state.completeDownload);
  const updateDownload = useDownloadsStore((state) => state.updateDownload);
  const markAsCancelled = useDownloadsStore((state) => state.markAsCancelled);
  const queueDownload = useDownloadsStore((state) => state.queueDownload);
  const startQueuedDownload = useDownloadsStore((state) => state.startQueuedDownload);
  const retryDownload = useDownloadsStore((state) => state.retryDownload);

  // Register event listeners
  useEffect(() => {
    const unlistenProgress = eventService.registerProgressListener(
      (data: DownloadProgressData) => {
        const progress = data.progress ?? 0;
        updateDownloadProgress(data.id, progress, data.downloaded_bytes, data.total_bytes, data.speed);
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

    const unlistenCancelled = eventService.registerCancelledListener(
      (data: DownloadCancelledData) => {
        markAsCancelled(data.id);
      }
    );

    const unlistenQueued = eventService.registerQueuedListener(
      (data: DownloadQueuedData) => {
        queueDownload(data.id, data.position);
      }
    );

    const unlistenStarted = eventService.registerStartedListener(
      (data: DownloadStartedData) => {
        startQueuedDownload(data.id);
      }
    );

    const unlistenRetry = eventService.registerRetryListener(
      (data: DownloadRetryData) => {
        // The retry creates a new download with a new ID
        // We need to handle the state transition
        retryDownload(data.id);
      }
    );

    return () => {
      unlistenProgress();
      unlistenCompleted();
      unlistenError();
      unlistenCancelled();
      unlistenQueued();
      unlistenStarted();
      unlistenRetry();
    };
  }, [updateDownloadProgress, completeDownload, updateDownload, markAsCancelled, queueDownload, startQueuedDownload, retryDownload]);

  const handleOpenDialog = () => {
    setIsDialogOpen(true);
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
                onClick={handleOpenDialog}
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
              queuePosition={download.queuePosition}
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