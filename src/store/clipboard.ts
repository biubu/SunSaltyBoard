import { create } from "zustand";
import type { ClipboardItem } from "../types";
import * as api from "../services/api";

function sortClipboardItems(items: ClipboardItem[]): ClipboardItem[] {
  return [...items].sort((a, b) => {
    if (a.is_favorite !== b.is_favorite) return a.is_favorite ? -1 : 1;
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
  });
}

interface ClipboardStore {
  items: ClipboardItem[];
  isLoading: boolean;
  error: string | null;

  loadHistory: () => Promise<void>;
  search: (query: string) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  pasteToActive: (item: ClipboardItem) => Promise<void>;
  toggleFavorite: (id: string) => Promise<void>;
  moveItemToGroup: (itemId: string, groupId: string | null) => Promise<void>;
  clearError: () => void;
}

export const useClipboardStore = create<ClipboardStore>((set, get) => ({
  items: [],
  isLoading: false,
  error: null,

  loadHistory: async () => {
    set({ isLoading: true, error: null });
    try {
      const items = await api.getClipboardHistory(200, 0);
      set({ items: sortClipboardItems(items), isLoading: false });
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
      set({ items: sortClipboardItems(items), isLoading: false });
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

  pasteToActive: async (item: ClipboardItem) => {
    try {
      await api.pasteToActive(item);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  toggleFavorite: async (id: string) => {
    try {
      const isFav = await api.toggleFavorite(id);
      set((state) => ({
        items: state.items.map((item) =>
          item.id === id ? { ...item, is_favorite: isFav } : item
        ),
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

  clearError: () => set({ error: null }),
}));
