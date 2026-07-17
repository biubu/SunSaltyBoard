import { create } from "zustand";
import type { Group } from "../types";
import * as api from "../services/api";

interface GroupsStore {
  groups: Group[];
  selectedGroup: Group | null;

  loadGroups: () => Promise<void>;
  createGroup: (name: string, color: string) => Promise<void>;
  deleteGroup: (id: string) => Promise<void>;
  setSelectedGroup: (group: Group | null) => void;
}

export const useGroupsStore = create<GroupsStore>((set) => ({
  groups: [],
  selectedGroup: null,

  loadGroups: async () => {
    try {
      const groups = await api.getGroups();
      set({ groups });
    } catch (e) {
      console.error("Failed to load groups:", e);
    }
  },

  createGroup: async (name: string, color: string) => {
    try {
      const group = await api.createGroup(name, color);
      set((state) => ({ groups: [...state.groups, group] }));
    } catch (e) {
      console.error("Failed to create group:", e);
    }
  },

  deleteGroup: async (id: string) => {
    try {
      await api.deleteGroup(id);
      set((state) => ({
        groups: state.groups.filter((g) => g.id !== id),
      }));
    } catch (e) {
      console.error("Failed to delete group:", e);
    }
  },

  setSelectedGroup: (group) => set({ selectedGroup: group }),
}));
