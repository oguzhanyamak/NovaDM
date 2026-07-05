import { History as HistoryIcon } from 'lucide-react';
import { useEffect } from 'react';
import { SectionHeader } from '../components/common/SectionHeader';
import { EmptyState } from '../components/common/EmptyState';
import { HistoryCard } from '../components/history/HistoryCard';
import { HistoryToolbar } from '../components/history/HistoryToolbar';
import { useHistoryStore } from '../store/history';

export function History() {
  const {
    filteredHistory,
    filter,
    sort,
    searchQuery,
    selectedIds,
    isLoading,
    loadHistory,
    setFilter,
    setSort,
    setSearchQuery,
    toggleSelection,
    clearHistory,
    deleteSelected,
  } = useHistoryStore();

  // Load history on mount
  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  return (
    <div className="flex flex-col h-full">
      <header className="border-b border-border bg-card">
        <div className="px-8 py-4">
          <SectionHeader
            title="History"
            description="View your download history"
          />
        </div>
      </header>

      <HistoryToolbar
        filter={filter}
        sort={sort}
        searchQuery={searchQuery}
        selectedCount={selectedIds.size}
        onFilterChange={setFilter}
        onSortChange={setSort}
        onSearchChange={setSearchQuery}
        onClearHistory={clearHistory}
        onDeleteSelected={deleteSelected}
      />

      <main className="flex-1 overflow-auto p-4">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-muted-foreground">Loading history...</p>
          </div>
        ) : filteredHistory.length === 0 ? (
          <EmptyState
            icon={
              <div className="w-24 h-24 rounded-full bg-secondary flex items-center justify-center mb-6">
                <HistoryIcon className="w-12 h-12 text-muted-foreground" />
              </div>
            }
            title="No history yet"
            description="Your completed, failed, and cancelled downloads will appear here."
          />
        ) : (
          <div className="space-y-3">
            {filteredHistory.map((entry) => (
              <HistoryCard
                key={entry.id}
                entry={entry}
                isSelected={selectedIds.has(entry.id)}
                onSelect={toggleSelection}
              />
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
