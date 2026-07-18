import { create } from "zustand";
import type { Settings } from "../types";
import * as api from "../services/api";

interface SettingsStore {
  settings: Settings | null;

  loadSettings: () => Promise<void>;
  updateSettings: (settings: Settings) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: null,

  loadSettings: async () => {
    try {
      const settings = await api.getSettings();
      set({ settings });
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  },

  updateSettings: async (settings: Settings) => {
    try {
      await api.updateSettings(settings);
      set({ settings });
    } catch (e) {
      console.error("Failed to update settings:", e);
    }
  },
}));
