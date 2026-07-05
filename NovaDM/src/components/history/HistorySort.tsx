import { HistorySort } from '../../types';
import { cn } from '../../lib/utils';

interface HistorySortProps {
  sort: HistorySort;
  onSortChange: (sort: HistorySort) => void;
  className?: string;
}

const SORT_OPTIONS: { value: HistorySort; label: string }[] = [
  { value: 'newest', label: 'Newest' },
  { value: 'oldest', label: 'Oldest' },
  { value: 'largest', label: 'Largest' },
  { value: 'smallest', label: 'Smallest' },
  { value: 'alphabetical', label: 'A-Z' },
];

export function HistorySortSelect({ sort, onSortChange, className }: HistorySortProps) {
  return (
    <select
      value={sort}
      onChange={(e) => onSortChange(e.target.value as HistorySort)}
      className={cn(
        'px-3 py-1.5 text-sm bg-background border border-border rounded-md text-foreground',
        'focus:outline-none focus:ring-2 focus:ring-primary',
        className
      )}
    >
      {SORT_OPTIONS.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}