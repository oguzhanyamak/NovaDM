import { useMemo } from 'react';
import { Copy, FolderOpen, RefreshCw, Pause, Play, X, Trash2, FileText } from 'lucide-react';
import { useDownloadsStore } from '../../store/downloads';
import { useHistoryStore } from '../../store/history';
import { useDetailsStore } from '../../store/details';
import { useSettingsStore } from '../../store/settings';
import { detailsService } from '../../services/details';
import { DetailsSection } from './DetailsSection';
import { DetailsRow } from './DetailsRow';
import { cn } from '../../lib/utils';

interface DetailsPanelProps {
  className?: string;
}

export function DetailsPanel({ className }: DetailsPanelProps) {
  const { selection, isCollapsed, setCollapsed } = useDetailsStore();
  const { downloads } = useDownloadsStore();
  const { history } = useHistoryStore();
  const { settings } = useSettingsStore();

  // Build details when selection changes
  const displayDetails = useMemo(() => {
    if (!selection.downloadId) return null;

    if (selection.source === 'downloads') {
      const download = downloads.find((d) => d.id === selection.downloadId);
      if (download) {
        return detailsService.buildFromDownload(download, settings.bandwidth_limit_kb * 1024);
      }
    } else if (selection.source === 'history') {
      const entry = history.find((h) => h.id === selection.downloadId);
      if (entry) {
        return detailsService.buildFromHistory(entry, settings.bandwidth_limit_kb * 1024);
      }
    }
    return null;
  }, [selection, downloads, history, settings.bandwidth_limit_kb]);

  // Get available actions based on status
  const availableActions = useMemo(() => {
    if (!displayDetails) return [];

    const actions: string[] = ['open_file', 'show_folder', 'copy_url'];

    if (displayDetails.status === 'downloading' || displayDetails.status === 'recovered') {
      actions.push('pause');
    } else if (displayDetails.status === 'paused') {
      actions.push('resume');
    } else if (displayDetails.status === 'error' || displayDetails.status === 'pending') {
      actions.push('retry');
    }

    if (selection.source === 'history') {
      actions.push('delete');
    }

    return actions;
  }, [displayDetails, selection.source]);

  const handleAction = async (action: string) => {
    if (!displayDetails) return;

    switch (action) {
      case 'open_file':
        // Would call Tauri open_file command
        break;
      case 'show_folder':
        // Would call Tauri show_in_folder command
        break;
      case 'copy_url':
        await navigator.clipboard.writeText(displayDetails.url);
        break;
      case 'pause':
        // Would call Tauri pause_download command
        break;
      case 'resume':
        // Would call Tauri resume_download command
        break;
      case 'retry':
        // Would call Tauri retry_download command
        break;
      case 'delete':
        // Would call Tauri delete_history_entry command
        break;
    }
  };

  if (isCollapsed) {
    return (
      <div className={cn('w-12 border-l border-border bg-card flex flex-col items-center justify-start pt-4', className)}>
        <button
          onClick={() => setCollapsed(false)}
          className="p-2 text-muted-foreground hover:text-foreground"
          title="Expand details"
        >
          <FileText className="w-5 h-5" />
        </button>
      </div>
    );
  }

  if (!displayDetails) {
    return (
      <div className={cn('w-80 border-l border-border bg-card flex flex-col', className)}>
        <div className="flex items-center justify-between p-3 border-b border-border">
          <h2 className="text-sm font-semibold text-foreground">Properties</h2>
        </div>
        <div className="flex-1 flex items-center justify-center p-4">
          <p className="text-sm text-muted-foreground">Select a download to view details</p>
        </div>
      </div>
    );
  }

  return (
    <div className={cn('w-80 border-l border-border bg-card flex flex-col', className)}>
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b border-border">
        <h2 className="text-sm font-semibold text-foreground">Properties</h2>
        <button
          onClick={() => setCollapsed(true)}
          className="p-1 text-muted-foreground hover:text-foreground"
          title="Collapse"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Content - Two column layout like IDM/FDM */}
      <div className="flex-1 overflow-auto p-3 space-y-4">
        {/* General Section */}
        <DetailsSection title="General">
          <div className="grid grid-cols-2 gap-x-2 gap-y-1">
            <DetailsRow label="File name" value={displayDetails.filename} />
            <DetailsRow label="File size" value={detailsService.formatBytes(displayDetails.file_size)} />
            <DetailsRow label="Downloaded" value={detailsService.formatBytes(displayDetails.downloaded_bytes)} />
            <DetailsRow label="Remaining" value={detailsService.formatBytes(displayDetails.remaining_bytes)} />
            <DetailsRow label="Progress" value={`${displayDetails.progress}%`} />
            <DetailsRow label="Status" value={displayDetails.status} />
            <DetailsRow label="URL" value={displayDetails.url} />
            <DetailsRow label="Output folder" value={displayDetails.output_folder} />
            <DetailsRow label="Output file" value={displayDetails.output_file} />
            <DetailsRow label="Resume" value={displayDetails.resume_supported ? 'Yes' : 'No'} />
            <DetailsRow label="Checksum" value={displayDetails.checksum} />
          </div>
        </DetailsSection>

        {/* Performance Section */}
        <DetailsSection title="Performance">
          <div className="grid grid-cols-2 gap-x-2 gap-y-1">
            <DetailsRow label="Current speed" value={detailsService.formatSpeed(displayDetails.performance.current_speed)} />
            <DetailsRow label="Average speed" value={detailsService.formatSpeed(displayDetails.performance.average_speed)} />
            <DetailsRow label="Peak speed" value={detailsService.formatSpeed(displayDetails.performance.peak_speed)} />
            <DetailsRow 
              label="Time remaining" 
              value={displayDetails.performance.estimated_time_remaining 
                ? detailsService.formatDuration(displayDetails.performance.estimated_time_remaining) 
                : undefined} 
            />
            <DetailsRow label="Elapsed time" value={detailsService.formatDuration(displayDetails.performance.elapsed_time)} />
            <DetailsRow label="Connections" value={displayDetails.technical.connection_count} />
            <DetailsRow label="Bandwidth limit" value={detailsService.formatSpeed(displayDetails.performance.bandwidth_limit)} />
          </div>
        </DetailsSection>

        {/* Timeline Section */}
        <DetailsSection title="Timeline">
          <div className="grid grid-cols-2 gap-x-2 gap-y-1">
            {displayDetails.timeline.map((event, index) => (
              <DetailsRow 
                key={index} 
                label={event.type.charAt(0).toUpperCase() + event.type.slice(1)} 
                value={new Date(event.timestamp).toLocaleTimeString()} 
              />
            ))}
          </div>
        </DetailsSection>

        {/* Technical Section */}
        <DetailsSection title="Technical">
          <div className="grid grid-cols-2 gap-x-2 gap-y-1">
            <DetailsRow label="HTTP status" value={displayDetails.technical.http_status} />
            <DetailsRow label="Server" value={displayDetails.technical.server} />
            <DetailsRow label="Content type" value={displayDetails.technical.content_type} />
            <DetailsRow label="ETag" value={displayDetails.technical.etag} />
            <DetailsRow label="Last modified" value={displayDetails.technical.last_modified} />
            <DetailsRow label="Accept ranges" value={displayDetails.technical.accept_ranges} />
          </div>
        </DetailsSection>
      </div>

      {/* Actions - Bottom bar like IDM/FDM */}
      <div className="p-3 border-t border-border bg-secondary/30">
        <div className="flex flex-wrap gap-1.5">
          {availableActions.includes('open_file') && (
            <button
              onClick={() => handleAction('open_file')}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-secondary text-secondary-foreground rounded hover:bg-secondary/80"
              title="Open file"
            >
              <FolderOpen className="w-3 h-3" />
              Open
            </button>
          )}
          {availableActions.includes('show_folder') && (
            <button
              onClick={() => handleAction('show_folder')}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-secondary text-secondary-foreground rounded hover:bg-secondary/80"
              title="Show in folder"
            >
              <FolderOpen className="w-3 h-3" />
              Folder
            </button>
          )}
          {availableActions.includes('copy_url') && (
            <button
              onClick={() => handleAction('copy_url')}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-secondary text-secondary-foreground rounded hover:bg-secondary/80"
              title="Copy URL"
            >
              <Copy className="w-3 h-3" />
              URL
            </button>
          )}
          {availableActions.includes('retry') && (
            <button
              onClick={() => handleAction('retry')}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/80"
              title="Retry"
            >
              <RefreshCw className="w-3 h-3" />
              Retry
            </button>
          )}
          {availableActions.includes('pause') && (
            <button
              onClick={() => handleAction('pause')}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-secondary text-secondary-foreground rounded hover:bg-secondary/80"
              title="Pause"
            >
              <Pause className="w-3 h-3" />
              Pause
            </button>
          )}
          {availableActions.includes('resume') && (
            <button
              onClick={() => handleAction('resume')}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/80"
              title="Resume"
            >
              <Play className="w-3 h-3" />
              Resume
            </button>
          )}
          {availableActions.includes('delete') && (
            <button
              onClick={() => handleAction('delete')}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-destructive text-destructive-foreground rounded hover:bg-destructive/80"
              title="Delete"
            >
              <Trash2 className="w-3 h-3" />
              Delete
            </button>
          )}
        </div>
      </div>
    </div>
  );
}