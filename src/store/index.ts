import { create } from "zustand";
import type {
  ClipboardItem,
  Group,
  Settings,
} from "../types";
import * as api from "../services/api";

interface AppStore {
  // State
  items: ClipboardItem[];
  groups: Group[];
  settings: Settings | null;
  selectedGroup: Group | null;
  searchQuery: string;
  isLoading: boolean;
  error: string | null;

  // Actions
  loadHistory: () => Promise<void>;
  search: (query: string) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  pasteItem: (item: ClipboardItem) => Promise<void>;
  pasteToActive: (item: ClipboardItem) => Promise<void>;

  loadGroups: () => Promise<void>;
  createGroup: (name: string, color: string) => Promise<void>;
  deleteGroup: (id: string) => Promise<void>;
  moveItemToGroup: (itemId: string, groupId: string | null) => Promise<void>;

  loadSettings: () => Promise<void>;
  updateSettings: (settings: Settings) => Promise<void>;

  setSelectedGroup: (group: Group | null) => void;
  setSearchQuery: (query: string) => void;

  clearError: () => void;
}

export const useStore = create<AppStore>((set, get) => ({
  // Initial state
  items: [] as ClipboardItem[],
  groups: [],
  settings: null,
  selectedGroup: null,
  searchQuery: "",
  isLoading: false,
  error: null,

  // Clipboard actions
  loadHistory: async () => {
    set({ isLoading: true, error: null });
    try {
      const items = await api.getClipboardHistory(100, 0);
      set({ items, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  search: async (query: string) => {
    if (!query.trim()) {
      return get().loadHistory();
    }
    set({ isLoading: true, error: null });
    try {
      const items = await api.searchClipboard(query, 50);
      set({ items, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  deleteItem: async (id: string) => {
    try {
      await api.deleteItem(id);
      set((state) => ({
        items: state.items.filter((item) => item.id !== id),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  pasteItem: async (item: ClipboardItem) => {
    try {
      await api.pasteItem(item);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  pasteToActive: async (item: ClipboardItem) => {
    try {
      await api.pasteToActive(item);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  // Group actions
  loadGroups: async () => {
    try {
      const groups = await api.getGroups();
      set({ groups });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createGroup: async (name: string, color: string) => {
    try {
      const group = await api.createGroup(name, color);
      set((state) => ({ groups: [...state.groups, group] }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteGroup: async (id: string) => {
    try {
      await api.deleteGroup(id);
      set((state) => ({
        groups: state.groups.filter((g) => g.id !== id),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  moveItemToGroup: async (itemId: string, groupId: string | null) => {
    try {
      await api.moveItemToGroup(itemId, groupId);
      set((state) => ({
        items: state.items.map((item) =>
          item.id === itemId ? { ...item, group_id: groupId } : item
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  // Settings actions
  loadSettings: async () => {
    try {
      const settings = await api.getSettings();
      set({ settings });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateSettings: async (settings: Settings) => {
    try {
      await api.updateSettings(settings);
      set({ settings });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  // UI actions
  setSelectedGroup: (group) => set({ selectedGroup: group }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  clearError: () => set({ error: null }),
}));

// Export shallow selector hook for optimized re-renders
export const useShallowStore = <T>(selector: (state: AppStore) => T) => {
  return useStore(selector);
};
