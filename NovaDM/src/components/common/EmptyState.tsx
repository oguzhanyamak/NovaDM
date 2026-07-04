import { cn } from '../../lib/utils';
import { Download } from 'lucide-react';

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description: string;
  action?: React.ReactNode;
  className?: string;
}

export function EmptyState({
  icon,
  title,
  description,
  action,
  className
}: EmptyStateProps) {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center h-full',
      className
    )}>
      {icon || (
        <div className="w-24 h-24 rounded-full bg-secondary flex items-center justify-center mb-6">
          <Download className="w-12 h-12 text-muted-foreground" />
        </div>
      )}
      <h3 className="text-xl font-semibold text-foreground mb-2">
        {title}
      </h3>
      <p className="text-muted-foreground text-center max-w-md mb-6">
        {description}
      </p>
      {action}
    </div>
  );
}