import { X } from 'lucide-react';
import { downloadService } from '../../services/download';
import { DownloadFileIcon } from './DownloadFileIcon';
import { DownloadStatusBadge } from './DownloadStatusBadge';
import { DownloadProgress } from './DownloadProgress';
import { DownloadSpeedLabel } from './DownloadSpeedLabel';
import { cn } from '../../lib/utils';

interface DownloadCardProps {
  id: string;
  name: string;
  url: string;
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error' | 'cancelled';
  progress: number;
  speed: number;
  queuePosition?: number;
  className?: string;
}

export function DownloadCard({
  id,
  name,
  url,
  status,
  progress,
  speed,
  queuePosition,
  className
}: DownloadCardProps) {
  const handleCancel = async () => {
    try {
      await downloadService.cancelDownload(id);
    } catch (err) {
      console.error('Failed to cancel download:', err);
    }
  };

  return (
    <div 
      data-testid={`download-card-${id}`}
      className={cn(
        'bg-card border border-border rounded-lg p-4 hover:border-primary/50 transition-colors',
        className
      )}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <DownloadFileIcon filename={name} />
          <div className="flex-1 min-w-0">
            <h4 className="font-medium text-foreground truncate">
              {name}
            </h4>
            <p className="text-sm text-muted-foreground truncate">
              {url}
            </p>
            {(status === 'downloading' || status === 'paused') && (
              <div className="mt-2">
                <DownloadProgress progress={progress} />
              </div>
            )}
          </div>
        </div>
        <div className="flex items-center gap-4 ml-4">
          {status === 'downloading' && (
            <DownloadSpeedLabel speed={speed} />
          )}
          <DownloadStatusBadge status={status} queuePosition={queuePosition} />
          {(status === 'downloading' || status === 'pending') && (
            <button
              onClick={handleCancel}
              className="p-1 rounded hover:bg-accent hover:text-accent-foreground"
              title="Cancel download"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}