import { Search } from 'lucide-react';
import { useDebounce } from '../../hooks/use-debounce';
import { useState, useEffect } from 'react';

interface HistorySearchProps {
  value: string;
  onChange: (value: string) => void;
  className?: string;
}

export function HistorySearch({ value, onChange, className }: HistorySearchProps) {
  const [localValue, setLocalValue] = useState(value);
  const debouncedValue = useDebounce(localValue, 300);

  useEffect(() => {
    onChange(debouncedValue);
  }, [debouncedValue, onChange]);

  return (
    <div className={cn('relative', className)}>
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
      <input
        type="text"
        placeholder="Search by filename or URL..."
        value={localValue}
        onChange={(e) => setLocalValue(e.target.value)}
        className="w-full pl-10 pr-4 py-2 bg-background border border-border rounded-lg text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
      />
    </div>
  );
}

function cn(...inputs: (string | undefined)[]) {
  return inputs.filter(Boolean).join(' ');
}