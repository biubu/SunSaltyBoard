// Re-export individual stores
export { useClipboardStore } from "./clipboard";
export { useGroupsStore } from "./groups";
export { useSettingsStore } from "./settings";

// Legacy compatibility: unified store that combines all sub-stores
// This maintains backward compatibility with existing components
import { useClipboardStore } from "./clipboard";
import { useGroupsStore } from "./groups";
import { useSettingsStore } from "./settings";
import type { ClipboardItem, Group, Settings } from "../types";

interface AppStore {
  items: ClipboardItem[];
  groups: Group[];
  settings: Settings | null;
  selectedGroup: Group | null;
  searchQuery: string;
  isLoading: boolean;
  error: string | null;

  loadHistory: () => Promise<void>;
  search: (query: string) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  pasteToActive: (item: ClipboardItem) => Promise<void>;
  toggleFavorite: (id: string) => Promise<void>;

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

export function useStore(): AppStore {
  const clipboard = useClipboardStore();
  const groups = useGroupsStore();
  const settings = useSettingsStore();

  return {
    // Clipboard state
    items: clipboard.items,
    isLoading: clipboard.isLoading,
    error: clipboard.error,

    // Groups state
    groups: groups.groups,
    selectedGroup: groups.selectedGroup,

    // Settings state
    settings: settings.settings,

    // Search query (local state in SearchBar, but exposed here for compatibility)
    searchQuery: "",

    // Clipboard actions
    loadHistory: clipboard.loadHistory,
    search: clipboard.search,
    deleteItem: clipboard.deleteItem,
    pasteToActive: clipboard.pasteToActive,
    toggleFavorite: clipboard.toggleFavorite,
    moveItemToGroup: clipboard.moveItemToGroup,
    clearError: clipboard.clearError,

    // Groups actions
    loadGroups: groups.loadGroups,
    createGroup: groups.createGroup,
    deleteGroup: groups.deleteGroup,
    setSelectedGroup: groups.setSelectedGroup,

    // Settings actions
    loadSettings: settings.loadSettings,
    updateSettings: settings.updateSettings,

    // Search query setter (no-op for compatibility)
    setSearchQuery: () => {},
  };
}
