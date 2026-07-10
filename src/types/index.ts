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

export interface Settings {
  max_history_size: number;
  auto_start: boolean;
  minimize_to_tray: boolean;
  global_shortcut: string;
  sync_enabled: boolean;
  sync_server: string | null;
  theme: string;
  update_server_url: string | null;
}

export interface UpdateInfo {
  latest_version: string | null;
  download_url: string | null;
  release_notes: string | null;
  error: string | null;
}

export interface SyncStatus {
  connected: boolean;
  last_sync: string | null;
  status: string;
}
