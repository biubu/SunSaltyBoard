import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "./store";
import { SearchBar } from "./components/search/SearchBar";
import { ClipboardList } from "./components/clipboard/ClipboardList";
import { SettingsPanel } from "./components/settings/SettingsPanel";
import type { ClipboardItem } from "./types";

const GROUP_COLORS = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4"];

function App() {
  const { loadHistory, loadSettings, loadGroups, settings, updateSettings, error, clearError, groups, selectedGroup, setSelectedGroup, createGroup, deleteGroup } = useStore();
  const [showSettings, setShowSettings] = useState(false);
  const [syncStatus, setSyncStatus] = useState<string>("");
  const [syncing, setSyncing] = useState(false);

  useEffect(() => {
    loadHistory();
    loadSettings();
    loadGroups();

    let unlistenFn: (() => void) | undefined;

    listen<ClipboardItem>("clipboard-changed", () => {
      loadHistory();
    }).then((unlisten) => {
      unlistenFn = unlisten;
    });

    return () => {
      unlistenFn?.();
    };
  }, []);

  const hideWindow = async () => {
    await invoke("hide_window");
  };

  const handleCreateGroup = useCallback(() => {
    const name = prompt("分组名称:");
    if (name?.trim()) {
      createGroup(name.trim(), GROUP_COLORS[groups.length % GROUP_COLORS.length]);
    }
  }, [createGroup, groups.length]);

  const handleSync = useCallback(async () => {
    if (!settings?.sync_enabled || !settings.sync_server) return;
    setSyncing(true);
    setSyncStatus("同步中...");
    try {
      const status = await invoke("trigger_sync") as { status: string };
      setSyncStatus(status.status === "synced" ? "同步成功" : status.status);
    } catch {
      setSyncStatus("同步失败");
    }
    setSyncing(false);
  }, [settings]);

  const isDark = settings?.theme === "dark";

  const theme = {
    bg: isDark ? "#111827" : "#f3f4f6",
    headerBg: isDark ? "#1f2937" : "#ffffff",
    headerBorder: isDark ? "#374151" : "#e5e7eb",
    titleText: isDark ? "#f9fafb" : "#374151",
    iconText: isDark ? "#d1d5db" : "#6b7280",
    iconHover: isDark ? "#f9fafb" : "#374151",
    searchBg: isDark ? "#1f2937" : "#ffffff",
    searchBorder: isDark ? "#4b5563" : "#d1d5db",
    searchText: isDark ? "#f9fafb" : "#1f2937",
    searchPlaceholder: isDark ? "#6b7280" : "#9ca3af",
    itemBg: isDark ? "#1f2937" : "#ffffff",
    itemBorder: isDark ? "#374151" : "#f3f4f6",
    itemText: isDark ? "#f3f4f6" : "#1f2937",
    itemTime: isDark ? "#9ca3af" : "#9ca3af",
    itemHover: isDark ? "#374151" : "#f9fafb",
    itemFocus: isDark ? "#1e3a5f" : "#f0f4ff",
    settingsBg: isDark ? "#1f2937" : "#ffffff",
    settingsBorder: isDark ? "#374151" : "#e5e7eb",
    settingsTitle: isDark ? "#f9fafb" : "#374151",
    settingsLabel: isDark ? "#d1d5db" : "#4b5563",
    settingsInputBg: isDark ? "#111827" : "#ffffff",
    settingsInputBorder: isDark ? "#4b5563" : "#d1d5db",
    settingsInputText: isDark ? "#f9fafb" : "#1f2937",
    groupBg: isDark ? "#1f2937" : "#ffffff",
    groupBorder: isDark ? "#374151" : "#e5e7eb",
    groupText: isDark ? "#d1d5db" : "#4b5563",
    groupActiveBg: isDark ? "#374151" : "#eff6ff",
    groupActiveText: isDark ? "#f9fafb" : "#1d4ed8",
    emptyText: isDark ? "#6b7280" : "#9ca3af",
  };


  return (
    <div className="h-screen flex flex-col overflow-hidden" style={{ background: theme.bg }}>
      {/* Title Bar */}
      <header
        className="flex items-center justify-between px-4 py-3 border-b"
        data-tauri-drag-region
        style={{ background: theme.headerBg, borderColor: theme.headerBorder }}
      >
        <div className="flex items-center gap-2" data-tauri-drag-region>
          <span className="text-sm font-semibold tracking-wide" data-tauri-drag-region style={{ color: theme.titleText }}>SunSaltyBoard</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowSettings(!showSettings)}
            className="w-7 h-7 flex items-center justify-center rounded transition-colors"
            title="设置"
            style={{ color: theme.iconText }}
            onMouseEnter={(e) => (e.currentTarget.style.color = theme.iconHover)}
            onMouseLeave={(e) => (e.currentTarget.style.color = theme.iconText)}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          </button>
          <button
            onClick={hideWindow}
            className="w-7 h-7 flex items-center justify-center rounded transition-colors"
            style={{ color: theme.iconText }}
            onMouseEnter={(e) => (e.currentTarget.style.color = theme.iconHover)}
            onMouseLeave={(e) => (e.currentTarget.style.color = theme.iconText)}
          >
            ✕
          </button>
        </div>
      </header>

      {/* Error toast */}
      {error && (
        <div className="mx-4 mt-2 px-3 py-2 rounded-lg text-xs text-red-700 bg-red-50 border border-red-200 flex items-center justify-between shrink-0">
          <span>{error}</span>
          <button onClick={clearError} className="ml-2 text-red-400 hover:text-red-600 font-bold">✕</button>
        </div>
      )}

      {/* Settings Panel */}
      {showSettings && settings && (
        <SettingsPanel
          settings={settings}
          onUpdate={updateSettings}
          onSync={handleSync}
          syncStatus={syncStatus}
          syncing={syncing}
          theme={theme}
        />
      )}

      {/* Search Bar */}
      <div style={{ margin: isDark ? "8px 10px 4px" : "10px" }}>
        <SearchBar theme={theme} />
      </div>

      {/* Group Filter */}
      {groups.length > 0 && (
        <div className="flex items-center gap-1 px-3 py-1 overflow-x-auto shrink-0" style={{ background: theme.groupBg }}>
          <button
            onClick={() => setSelectedGroup(null)}
            className="px-2 py-0.5 rounded-full text-xs whitespace-nowrap transition-colors"
            style={{
              background: !selectedGroup ? (isDark ? "#374151" : "#eff6ff") : "transparent",
              color: !selectedGroup ? (isDark ? "#f9fafb" : "#1d4ed8") : theme.groupText,
              border: `1px solid ${!selectedGroup ? (isDark ? "#4b5563" : "#bfdbfe") : theme.groupBorder}`,
            }}
          >
            全部
          </button>
          {groups.map((g) => (
            <button
              key={g.id}
              onClick={() => setSelectedGroup(selectedGroup?.id === g.id ? null : g)}
              onContextMenu={(e) => {
                e.preventDefault();
                if (confirm(`删除分组 "${g.name}"？`)) deleteGroup(g.id);
              }}
              className="px-2 py-0.5 rounded-full text-xs whitespace-nowrap transition-colors flex items-center gap-1"
              style={{
                background: selectedGroup?.id === g.id ? (isDark ? "#374151" : "#eff6ff") : "transparent",
                color: selectedGroup?.id === g.id ? (isDark ? "#f9fafb" : "#1d4ed8") : theme.groupText,
                border: `1px solid ${selectedGroup?.id === g.id ? (isDark ? "#4b5563" : "#bfdbfe") : theme.groupBorder}`,
              }}
            >
              <span className="inline-block w-1.5 h-1.5 rounded-full shrink-0" style={{ background: g.color }} />
              {g.name}
            </button>
          ))}
          <button
            onClick={handleCreateGroup}
            className="w-5 h-5 flex items-center justify-center rounded-full text-xs shrink-0 transition-colors"
            style={{ color: theme.groupText }}
            title="新建分组"
          >
            +
          </button>
        </div>
      )}

      {/* Clipboard List */}
      <div className="flex-1 overflow-hidden">
        <ClipboardList theme={theme} />
      </div>
    </div>
  );
}

export default App;
