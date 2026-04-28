import { invoke } from "@tauri-apps/api/core";
import type {
  ClipboardItem,
  Group,
  Tag,
  Hotkey,
  Plugin,
  Settings,
  SyncStatus,
} from "../types";

// Clipboard
export async function getClipboardHistory(
  limit?: number,
  offset?: number
): Promise<ClipboardItem[]> {
  return invoke("get_clipboard_history", { limit, offset });
}

export async function searchClipboard(
  query: string,
  limit?: number
): Promise<ClipboardItem[]> {
  return invoke("search_clipboard", { query, limit });
}

export async function pasteItem(item: ClipboardItem): Promise<void> {
  return invoke("paste_item", { item });
}

export async function pasteToActive(item: ClipboardItem): Promise<void> {
  return invoke("paste_to_active", { item });
}

export async function deleteItem(id: string): Promise<void> {
  return invoke("delete_item", { id });
}

export async function toggleFavorite(id: string): Promise<boolean> {
  return invoke("toggle_favorite", { id });
}

// Groups
export async function getGroups(): Promise<Group[]> {
  return invoke("get_groups");
}

export async function createGroup(name: string, color: string): Promise<Group> {
  return invoke("create_group", { name, color });
}

export async function deleteGroup(id: string): Promise<void> {
  return invoke("delete_group", { id });
}

export async function moveItemToGroup(
  itemId: string,
  groupId: string | null
): Promise<void> {
  return invoke("move_item_to_group", { item_id: itemId, group_id: groupId });
}

// Tags
export async function getTags(): Promise<Tag[]> {
  return invoke("get_tags");
}

export async function createTag(name: string, color: string): Promise<Tag> {
  return invoke("create_tag", { name, color });
}

export async function deleteTag(id: string): Promise<void> {
  return invoke("delete_tag", { id });
}

export async function addTagToItem(itemId: string, tagId: string): Promise<void> {
  return invoke("add_tag_to_item", { item_id: itemId, tag_id: tagId });
}

export async function removeTagFromItem(
  itemId: string,
  tagId: string
): Promise<void> {
  return invoke("remove_tag_from_item", { item_id: itemId, tag_id: tagId });
}

// Hotkeys
export async function getHotkeys(): Promise<Hotkey[]> {
  return invoke("get_hotkeys");
}

export async function updateHotkey(
  action: string,
  keyCombination: string
): Promise<void> {
  return invoke("update_hotkey", { action, key_combination: keyCombination });
}

// Plugins
export async function getPlugins(): Promise<Plugin[]> {
  return invoke("get_plugins");
}

export async function togglePlugin(id: string, enabled: boolean): Promise<void> {
  return invoke("toggle_plugin", { id, enabled });
}

// Settings
export async function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: Settings): Promise<void> {
  return invoke("update_settings", { settings });
}

// Sync
export async function triggerSync(): Promise<SyncStatus> {
  return invoke("trigger_sync");
}

export async function getSyncStatus(): Promise<SyncStatus> {
  return invoke("get_sync_status");
}

// Window
export async function showWindow(): Promise<void> {
  return invoke("show_window");
}

export async function hideWindow(): Promise<void> {
  return invoke("hide_window");
}
