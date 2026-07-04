import { cn } from '../../lib/utils';

interface DownloadStatusBadgeProps {
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error';
  className?: string;
}

export function DownloadStatusBadge({ status, className }: DownloadStatusBadgeProps) {
  const getStatusConfig = () => {
    switch (status) {
      case 'downloading':
        return {
          label: 'Downloading',
          className: 'bg-blue-500/10 text-blue-500 border-blue-500/20'
        };
      case 'paused':
        return {
          label: 'Paused',
          className: 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20'
        };
      case 'completed':
        return {
          label: 'Completed',
          className: 'bg-green-500/10 text-green-500 border-green-500/20'
        };
      case 'error':
        return {
          label: 'Error',
          className: 'bg-red-500/10 text-red-500 border-red-500/20'
        };
      case 'pending':
      default:
        return {
          label: 'Pending',
          className: 'bg-muted text-muted-foreground border-border'
        };
    }
  };

  const config = getStatusConfig();

  return (
    <span className={cn(
      'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border',
      config.className,
      className
    )}>
      {config.label}
    </span>
  );
}