import { HistorySearch } from './HistorySearch';
import { HistoryFilters } from './HistoryFilters';
import { HistorySortSelect } from './HistorySort';
import { HistoryFilter, HistorySort } from '../../types';
import { Trash2 } from 'lucide-react';
import { ConfirmationDialog } from '../common/ConfirmationDialog';
import { useState } from 'react';

interface HistoryToolbarProps {
  filter: HistoryFilter;
  sort: HistorySort;
  searchQuery: string;
  selectedCount: number;
  onFilterChange: (filter: HistoryFilter) => void;
  onSortChange: (sort: HistorySort) => void;
  onSearchChange: (query: string) => void;
  onClearHistory: () => void;
  onDeleteSelected: () => void;
}

export function HistoryToolbar({
  filter,
  sort,
  searchQuery,
  selectedCount,
  onFilterChange,
  onSortChange,
  onSearchChange,
  onClearHistory,
  onDeleteSelected,
}: HistoryToolbarProps) {
  const [showClearDialog, setShowClearDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  return (
    <div className="flex items-center justify-between gap-4 p-4 border-b border-border bg-card">
      <div className="flex-1 max-w-md">
        <HistorySearch value={searchQuery} onChange={onSearchChange} />
      </div>
      
      <div className="flex items-center gap-4">
        <HistoryFilters filter={filter} onFilterChange={onFilterChange} />
        
        <HistorySortSelect sort={sort} onSortChange={onSortChange} />
        
        {selectedCount > 0 && (
          <button
            onClick={() => setShowDeleteDialog(true)}
            className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            Delete ({selectedCount})
          </button>
        )}
        
        <button
          onClick={() => setShowClearDialog(true)}
          className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        >
          <Trash2 className="w-4 h-4" />
          Clear All
        </button>
      </div>

      <ConfirmationDialog
        open={showClearDialog}
        onOpenChange={setShowClearDialog}
        title="Clear all history?"
        description="This will remove all download history entries. This action cannot be undone."
        confirmLabel="Clear All"
        cancelLabel="Cancel"
        variant="destructive"
        onConfirm={onClearHistory}
      />

      <ConfirmationDialog
        open={showDeleteDialog}
        onOpenChange={setShowDeleteDialog}
        title={`Delete ${selectedCount} selected entries?`}
        description="This action cannot be undone."
        confirmLabel="Delete"
        cancelLabel="Cancel"
        variant="destructive"
        onConfirm={onDeleteSelected}
      />
    </div>
  );
}