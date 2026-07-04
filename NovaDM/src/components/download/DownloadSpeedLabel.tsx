import { cn } from '../../lib/utils';

interface DownloadSpeedLabelProps {
  speed: number; // bytes per second
  className?: string;
}

export function DownloadSpeedLabel({ speed, className }: DownloadSpeedLabelProps) {
  const formatSpeed = (bytesPerSecond: number): string => {
    if (bytesPerSecond === 0) return '0 B/s';
    
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    let size = bytesPerSecond;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  return (
    <span className={cn('text-xs text-muted-foreground', className)}>
      {formatSpeed(speed)}
    </span>
  );
}