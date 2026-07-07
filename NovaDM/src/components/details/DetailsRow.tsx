import { cn } from '../../lib/utils';

interface DetailsRowProps {
  label: string;
  value: string | number | undefined;
  className?: string;
}

export function DetailsRow({ label, value, className }: DetailsRowProps) {
  return (
    <div className={cn('flex justify-between py-2 border-b border-border/50 last:border-0', className)}>
      <span className="text-sm text-muted-foreground">{label}</span>
      <span className="text-sm font-medium text-foreground">
        {value ?? '—'}
      </span>
    </div>
  );
}