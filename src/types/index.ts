export interface ClipboardItem {
  id: string;
  content_type: string;
  content: string;
  preview: string;
  group_id: string | null;
  created_at: string;
  is_favorite: boolean;
  metadata: string | null;
}

export interface Group {
  id: string;
  name: string;
  color: string;
  created_at: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string;
}

export interface Hotkey {
  id: string;
  action: string;
  key_combination: string;
  enabled: boolean;
}

export interface Plugin {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
  config: string | null;
}

export interface Settings {
  max_history_size: number;
  auto_start: boolean;
  minimize_to_tray: boolean;
  global_shortcut: string;
  sync_enabled: boolean;
  sync_server: string | null;
  theme: string;
  sensitive_filter: boolean;
  encrypt_sensitive: boolean;
}

export interface SyncStatus {
  connected: boolean;
  last_sync: string | null;
  status: string;
}

export interface SearchResult {
  items: ClipboardItem[];
  total: number;
}
