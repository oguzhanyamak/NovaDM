import { cn } from '../../lib/utils';

interface DownloadStatusBadgeProps {
  status: 'pending' | 'downloading' | 'paused' | 'recovered' | 'completed' | 'error' | 'cancelled';
  queuePosition?: number;
  className?: string;
}

export function DownloadStatusBadge({ status, queuePosition, className }: DownloadStatusBadgeProps) {
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
      case 'cancelled':
        return {
          label: 'Cancelled',
          className: 'bg-gray-500/10 text-gray-500 border-gray-500/20'
        };
      case 'recovered':
        return {
          label: 'Recovered',
          className: 'bg-orange-500/10 text-orange-500 border-orange-500/20'
        };
      case 'pending':
      default:
        if (queuePosition) {
          return {
            label: `Queued (#${queuePosition})`,
            className: 'bg-purple-500/10 text-purple-500 border-purple-500/20'
          };
        }
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