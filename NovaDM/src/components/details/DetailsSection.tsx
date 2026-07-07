import { ReactNode } from 'react';
import { cn } from '../../lib/utils';

interface DetailsSectionProps {
  title: string;
  children: ReactNode;
  className?: string;
}

export function DetailsSection({ title, children, className }: DetailsSectionProps) {
  return (
    <div className={cn('space-y-2', className)}>
      <h3 className="text-sm font-semibold text-foreground">{title}</h3>
      <div className="space-y-1">{children}</div>
    </div>
  );
}