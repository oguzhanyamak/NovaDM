import { create } from 'zustand';
import { HistoryEntry, HistoryFilter, HistorySort } from '../types';
import { historyService } from '../services/history';

interface HistoryState {
  // Raw history data from backend
  history: HistoryEntry[];
  // Filtered and sorted history
  filteredHistory: HistoryEntry[];
  // Current filter
  filter: HistoryFilter;
  // Current sort
  sort: HistorySort;
  // Search query
  searchQuery: string;
  // Selected entry IDs
  selectedIds: Set<string>;
  // Loading state
  isLoading: boolean;
  // Error state
  error: string | null;

  // Actions
  loadHistory: () => Promise<void>;
  deleteEntry: (id: string) => Promise<void>;
  deleteSelected: () => Promise<void>;
  clearHistory: () => Promise<void>;
  setFilter: (filter: HistoryFilter) => void;
  setSort: (sort: HistorySort) => void;
  setSearchQuery: (query: string) => void;
  toggleSelection: (id: string) => void;
  selectAll: () => void;
  clearSelection: () => void;
  isSelected: (id: string) => boolean;
  // Internal
  applyFiltersAndSort: () => void;
}

export const useHistoryStore = create<HistoryState>((set, get) => ({
  history: [],
  filteredHistory: [],
  filter: 'all',
  sort: 'newest',
  searchQuery: '',
  selectedIds: new Set(),
  isLoading: false,
  error: null,

  loadHistory: async () => {
    set({ isLoading: true, error: null });
    try {
      const history = await historyService.getHistory();
      set({
        history,
        isLoading: false,
      });
      get().applyFiltersAndSort();
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to load history',
        isLoading: false 
      });
    }
  },

  deleteEntry: async (id: string) => {
    try {
      await historyService.deleteEntry(id);
      set((state) => ({
        history: state.history.filter((h) => h.id !== id),
        selectedIds: new Set([...state.selectedIds].filter((i) => i !== id)),
      }));
      get().applyFiltersAndSort();
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to delete entry' 
      });
    }
  },

  deleteSelected: async () => {
    const { selectedIds } = get();
    const ids = Array.from(selectedIds);
    
    if (ids.length === 0) return;

    try {
      await historyService.deleteEntries(ids);
      set((state) => ({
        history: state.history.filter((h) => !state.selectedIds.has(h.id)),
        selectedIds: new Set(),
      }));
      get().applyFiltersAndSort();
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to delete entries' 
      });
    }
  },

  clearHistory: async () => {
    try {
      await historyService.clearHistory();
      set({ history: [], selectedIds: new Set() });
      get().applyFiltersAndSort();
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to clear history' 
      });
    }
  },

  setFilter: (filter: HistoryFilter) => {
    set({ filter });
    get().applyFiltersAndSort();
  },

  setSort: (sort: HistorySort) => {
    set({ sort });
    get().applyFiltersAndSort();
  },

  setSearchQuery: (searchQuery: string) => {
    set({ searchQuery });
    get().applyFiltersAndSort();
  },

  toggleSelection: (id: string) => {
    set((state) => {
      const newSelected = new Set(state.selectedIds);
      if (newSelected.has(id)) {
        newSelected.delete(id);
      } else {
        newSelected.add(id);
      }
      return { selectedIds: newSelected };
    });
  },

  selectAll: () => {
    set((state) => ({
      selectedIds: new Set(state.filteredHistory.map((h) => h.id)),
    }));
  },

  clearSelection: () => {
    set({ selectedIds: new Set() });
  },

  isSelected: (id: string) => {
    return get().selectedIds.has(id);
  },

  // Internal: Apply filters and sorting
  applyFiltersAndSort: () => {
    const { history, filter, sort, searchQuery } = get();
    
    // Apply filter
    let filtered = history;
    if (filter !== 'all') {
      filtered = history.filter((h) => h.status === filter);
    }

    // Apply search (case insensitive)
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (h) =>
          h.filename.toLowerCase().includes(query) ||
          h.url.toLowerCase().includes(query)
      );
    }

    // Apply sort
    filtered.sort((a, b) => {
      switch (sort) {
        case 'newest':
          return b.completed_at - a.completed_at;
        case 'oldest':
          return a.completed_at - b.completed_at;
        case 'largest':
          return b.file_size - a.file_size;
        case 'smallest':
          return a.file_size - b.file_size;
        case 'alphabetical':
          return a.filename.localeCompare(b.filename);
        default:
          return 0;
      }
    });

    set({ filteredHistory: filtered });
  },
}));