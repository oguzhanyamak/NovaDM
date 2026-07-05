import { useEffect, useState } from 'react';
import { cn } from '../../lib/utils';

interface SettingsInputProps {
  value: string | number;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: 'text' | 'number';
  min?: number;
  className?: string;
}

export function SettingsInput({
  value,
  onChange,
  placeholder,
  type = 'text',
  min,
  className,
}: SettingsInputProps) {
  const [localValue, setLocalValue] = useState<string | number>(value);

  // Sync local value with prop changes (e.g. from store load/reset)
  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  const handleBlur = () => {
    onChange(localValue.toString());
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      onChange(localValue.toString());
      e.currentTarget.blur();
    }
  };

  return (
    <input
      type={type}
      value={localValue}
      onChange={(e) => setLocalValue(e.target.value)}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      placeholder={placeholder}
      min={min}
      className={cn(
        'w-full px-3 py-2 bg-background border border-border rounded-md text-foreground',
        'focus:outline-none focus:ring-2 focus:ring-primary',
        className
      )}
    />
  );
}