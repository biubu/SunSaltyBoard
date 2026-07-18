import { invoke } from "@tauri-apps/api/core";
import type {
  ClipboardItem,
  Group,
  Settings,
  UpdateInfo,
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

// Settings
export async function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: Settings): Promise<void> {
  return invoke("update_settings", { settings });
}

// Accessibility (macOS)
export async function checkAccessibilityPermission(): Promise<boolean> {
  return invoke("check_accessibility_permission");
}

export async function openAccessibilitySettings(): Promise<void> {
  return invoke("open_accessibility_settings");
}

// Updates
export async function getAppVersion(): Promise<string> {
  return invoke("get_app_version");
}

export async function checkUpdate(): Promise<UpdateInfo> {
  return invoke("check_update");
}
