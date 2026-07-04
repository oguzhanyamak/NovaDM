import { cn } from '../../lib/utils';

interface DownloadProgressProps {
  progress: number;
  className?: string;
}

export function DownloadProgress({ progress, className }: DownloadProgressProps) {
  const clampedProgress = Math.min(100, Math.max(0, progress));
  
  return (
    <div className={cn('w-full', className)}>
      <div className="flex items-center justify-between mb-1">
        <span className="text-xs font-medium text-foreground">
          {clampedProgress.toFixed(0)}%
        </span>
      </div>
      <div className="h-2 w-full bg-secondary rounded-full overflow-hidden">
        <div
          className="h-full bg-primary transition-all duration-300 ease-in-out"
          style={{ width: `${clampedProgress}%` }}
        />
      </div>
    </div>
  );
}