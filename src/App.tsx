import { useEffect, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "./store";
import { SearchBar } from "./components/search/SearchBar";
import { ClipboardList } from "./components/clipboard/ClipboardList";
import { SettingsPanel } from "./components/settings/SettingsPanel";
import type { ClipboardItem } from "./types";

const GROUP_COLORS = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4"];
const CLIPBOARD_RELOAD_DEBOUNCE_MS = 150;

function App() {
  const { loadHistory, loadSettings, loadGroups, settings, updateSettings, error, clearError, groups, selectedGroup, setSelectedGroup, createGroup, deleteGroup, searchQuery } = useStore();
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    loadHistory();
    loadSettings();
    loadGroups();

    let unlistenFn: UnlistenFn | undefined;
    let debounceTimer: number | undefined;
    let cancelled = false;

    const scheduleReload = () => {
      if (debounceTimer !== undefined) {
        window.clearTimeout(debounceTimer);
      }
      // Skip reload if user is actively searching; the search query will
      // re-run via the debounced SearchBar.
      if (searchQuery.trim()) return;
      debounceTimer = window.setTimeout(() => {
        loadHistory();
      }, CLIPBOARD_RELOAD_DEBOUNCE_MS);
    };

    listen<ClipboardItem>("clipboard-changed", scheduleReload)
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          unlistenFn = unlisten;
        }
      })
      .catch((e) => {
        console.error("Failed to listen for clipboard-changed:", e);
      });

    return () => {
      cancelled = true;
      if (debounceTimer !== undefined) window.clearTimeout(debounceTimer);
      unlistenFn?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
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

  const isDark = settings?.theme === "dark";

  const theme = {
    bg: isDark ? "#111827" : "#f3f4f6",
    headerBg: isDark ? "#1f2937" : "#ffffff",
    headerBorder: isDark ? "#374151" : "#e5e7eb",
    titleText: isDark ? "#f9fafb" : "#374151",
    iconText: isDark ? "#d1d5db" : "#4b5563",
    iconHover: isDark ? "#f9fafb" : "#1f2937",
    accent: "#3b82f6",
    searchBg: isDark ? "#1f2937" : "#ffffff",
    searchBorder: isDark ? "#4b5563" : "#d1d5db",
    searchText: isDark ? "#f9fafb" : "#1f2937",
    searchPlaceholder: isDark ? "#9ca3af" : "#9ca3af",
    itemBg: isDark ? "#1f2937" : "#ffffff",
    itemBorder: isDark ? "#374151" : "#e5e7eb",
    itemText: isDark ? "#f3f4f6" : "#1f2937",
    itemTime: isDark ? "#9ca3af" : "#9ca3af",
    itemFavoriteBg: isDark ? "rgba(245, 158, 11, 0.18)" : "#fff3cd",
    itemFavoriteText: isDark ? "#fcd34d" : "#92400e",
    itemHover: isDark ? "#374151" : "#f3f4f6",
    itemFocus: isDark ? "#1e3a5f" : "#f0f4ff",
    settingsBg: isDark ? "#1f2937" : "#ffffff",
    settingsCardHeaderBg: isDark ? "#161e2b" : "#f9fafb",
    settingsCardShadow: isDark
      ? "0 1px 3px rgba(0,0,0,0.4), 0 4px 12px rgba(0,0,0,0.3)"
      : "0 1px 2px rgba(0,0,0,0.04), 0 4px 16px rgba(0,0,0,0.04)",
    settingsBorder: isDark ? "#374151" : "#e5e7eb",
    settingsTitle: isDark ? "#f9fafb" : "#374151",
    settingsLabel: isDark ? "#d1d5db" : "#4b5563",
    settingsInputBg: isDark ? "#0b1220" : "#f9fafb",
    settingsInputBorder: isDark ? "#4b5563" : "#d1d5db",
    settingsInputText: isDark ? "#f9fafb" : "#1f2937",
    settingsHint: isDark ? "#9ca3af" : "#6b7280",
    toggleTrack: isDark ? "#4b5563" : "#d1d5db",
    groupBg: isDark ? "#1f2937" : "#ffffff",
    groupBorder: isDark ? "#374151" : "#e5e7eb",
    groupText: isDark ? "#d1d5db" : "#4b5563",
    groupActiveBg: isDark ? "#1e3a5f" : "#eff6ff",
    groupActiveText: isDark ? "#93c5fd" : "#1d4ed8",
    groupActiveBorder: isDark ? "#3b82f6" : "#bfdbfe",
    emptyText: isDark ? "#9ca3af" : "#9ca3af",
  };


  return (
    <div className="h-screen flex flex-col overflow-hidden" data-tauri-drag-region="deep" style={{ background: theme.bg }}>
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
      {showSettings && settings ? (
        <SettingsPanel
          settings={settings}
          onUpdate={updateSettings}
          theme={theme}
          onClose={() => setShowSettings(false)}
        />
      ) : (
        <>
          {/* Search Bar */}
          <div style={{ margin: isDark ? "8px 10px 4px" : "10px" }}>
            <SearchBar theme={theme} />
          </div>

          {/* Group Filter */}
          {groups.length > 0 && (
            <div
              className="flex items-center gap-1 px-3 py-1.5 overflow-x-auto shrink-0 border-b"
              style={{ background: theme.groupBg, borderColor: theme.headerBorder }}
            >
              <button
                onClick={() => setSelectedGroup(null)}
                className="px-2.5 py-0.5 rounded-full text-xs whitespace-nowrap transition-colors"
                style={{
                  background: !selectedGroup ? theme.groupActiveBg : "transparent",
                  color: !selectedGroup ? theme.groupActiveText : theme.groupText,
                  border: `1px solid ${!selectedGroup ? theme.groupActiveBorder : "transparent"}`,
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
                  className="px-2.5 py-0.5 rounded-full text-xs whitespace-nowrap transition-colors flex items-center gap-1"
                  style={{
                    background:
                      selectedGroup?.id === g.id ? theme.groupActiveBg : "transparent",
                    color:
                      selectedGroup?.id === g.id ? theme.groupActiveText : theme.groupText,
                    border: `1px solid ${
                      selectedGroup?.id === g.id ? theme.groupActiveBorder : "transparent"
                    }`,
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
        </>
      )}
    </div>
  );
}

export default App;
