import { FileText, FolderOpen, Copy, RefreshCw, Trash2 } from 'lucide-react';
import { HistoryEntry } from '../../types';
import { downloadService } from '../../services/download';
import { historyService } from '../../services/history';
import { DownloadFileIcon } from '../download/DownloadFileIcon';
import { DownloadStatusBadge } from '../download/DownloadStatusBadge';
import { formatFileSize, formatSpeed, formatDuration, formatDate } from '../../lib/utils';
import { cn } from '../../lib/utils';

interface HistoryCardProps {
  entry: HistoryEntry;
  isSelected: boolean;
  onSelect: (id: string) => void;
  className?: string;
}

export function HistoryCard({ entry, isSelected, onSelect, className }: HistoryCardProps) {
  const handleOpenFile = async () => {
    try {
      await downloadService.openFile(entry.output_path);
    } catch (err) {
      console.error('Failed to open file:', err);
    }
  };

  const handleShowInFolder = async () => {
    try {
      await downloadService.showInFolder(entry.output_path);
    } catch (err) {
      console.error('Failed to show in folder:', err);
    }
  };

  const handleCopyUrl = async () => {
    try {
      await navigator.clipboard.writeText(entry.url);
    } catch (err) {
      console.error('Failed to copy URL:', err);
    }
  };

  const handleDownloadAgain = async () => {
    try {
      // Extract directory from output_path
      const saveLocation = entry.output_path.substring(0, entry.output_path.lastIndexOf('/'));
      await downloadService.startDownload({
        url: entry.url,
        filename: entry.filename,
        saveLocation,
      });
    } catch (err) {
      console.error('Failed to download again:', err);
    }
  };

  const handleDelete = async () => {
    try {
      await historyService.deleteEntry(entry.id);
    } catch (err) {
      console.error('Failed to delete entry:', err);
    }
  };

  return (
    <div
      data-testid={`history-card-${entry.id}`}
      className={cn(
        'bg-card border border-border rounded-lg p-4 hover:border-primary/50 transition-colors',
        isSelected && 'border-primary bg-primary/5',
        className
      )}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <input
            type="checkbox"
            checked={isSelected}
            onChange={() => onSelect(entry.id)}
            className="w-4 h-4 rounded border-border"
            onClick={(e) => e.stopPropagation()}
          />
          <DownloadFileIcon filename={entry.filename} />
          <div className="flex-1 min-w-0">
            <h4 className="font-medium text-foreground truncate">
              {entry.filename}
            </h4>
            <p className="text-sm text-muted-foreground truncate">
              {entry.url}
            </p>
            <div className="flex items-center gap-4 mt-1 text-xs text-muted-foreground">
              <span>{formatFileSize(entry.file_size)}</span>
              <span>{formatDate(new Date(entry.completed_at * 1000))}</span>
              <span>{formatDuration(entry.duration)}</span>
              <span>{formatSpeed(entry.average_speed)} avg</span>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2 ml-4">
          <DownloadStatusBadge 
            status={entry.status as 'pending' | 'downloading' | 'paused' | 'recovered' | 'completed' | 'error' | 'cancelled'} 
          />
          <div className="flex items-center gap-1">
            <button
              onClick={handleOpenFile}
              className="p-1 rounded hover:bg-accent hover:text-accent-foreground"
              title="Open file"
            >
              <FileText className="w-4 h-4" />
            </button>
            <button
              onClick={handleShowInFolder}
              className="p-1 rounded hover:bg-accent hover:text-accent-foreground"
              title="Show in folder"
            >
              <FolderOpen className="w-4 h-4" />
            </button>
            <button
              onClick={handleCopyUrl}
              className="p-1 rounded hover:bg-accent hover:text-accent-foreground"
              title="Copy URL"
            >
              <Copy className="w-4 h-4" />
            </button>
            <button
              onClick={handleDownloadAgain}
              className="p-1 rounded hover:bg-accent hover:text-accent-foreground"
              title="Download again"
            >
              <RefreshCw className="w-4 h-4" />
            </button>
            <button
              onClick={handleDelete}
              className="p-1 rounded hover:bg-destructive hover:text-destructive-foreground"
              title="Delete from history"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}