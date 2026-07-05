import { HistoryFilter } from '../../types';
import { cn } from '../../lib/utils';

interface HistoryFiltersProps {
  filter: HistoryFilter;
  onFilterChange: (filter: HistoryFilter) => void;
  className?: string;
}

const FILTER_OPTIONS: { value: HistoryFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'completed', label: 'Completed' },
  { value: 'failed', label: 'Failed' },
  { value: 'cancelled', label: 'Cancelled' },
];

export function HistoryFilters({ filter, onFilterChange, className }: HistoryFiltersProps) {
  return (
    <div className={cn('flex items-center gap-1', className)}>
      {FILTER_OPTIONS.map((option) => (
        <button
          key={option.value}
          onClick={() => onFilterChange(option.value)}
          className={cn(
            'px-3 py-1.5 text-sm font-medium rounded-md transition-colors',
            filter === option.value
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}