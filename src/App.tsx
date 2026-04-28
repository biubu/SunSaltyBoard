import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "./store";
import { SearchBar } from "./components/search/SearchBar";
import { ClipboardList } from "./components/clipboard/ClipboardList";
import type { ClipboardItem } from "./types";

function App() {
  const { loadHistory, loadSettings, settings, updateSettings, error, clearError, items } = useStore();
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    loadHistory();
    loadSettings();

    const unlisten = listen<ClipboardItem>("clipboard-changed", (event) => {
      console.log("Clipboard changed:", event.payload);
      loadHistory();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const hideWindow = async () => {
    await invoke("hide_window");
  };

  return (
    <div className="h-screen text-text flex flex-col overflow-hidden" style={{ background: "#f3f4f6", overflow: "hidden" }}>
      {/* Title Bar */}
      <header
        className="flex items-center justify-between px-4 py-3 border-b border-gray-200"
        data-tauri-drag-region
      >
        <div className="flex items-center gap-2" data-tauri-drag-region>
          <span className="text-xl" data-tauri-drag-region>📋</span>
          <span className="text-sm font-semibold text-gray-700 tracking-wide" data-tauri-drag-region>SunSaltyBoard</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowSettings(!showSettings)}
            className="w-7 h-7 flex items-center justify-center rounded text-gray-500 hover:text-gray-700 transition-colors text-sm"
            title="设置"
          >
            ⚙
          </button>
          <button
            onClick={hideWindow}
            className="w-7 h-7 flex items-center justify-center rounded text-gray-500 hover:text-gray-700 transition-colors"
          >
            ✕
          </button>
        </div>
      </header>

      {/* Global error toast */}
      {error && (
        <div className="mx-4 mt-2 px-3 py-2 rounded-lg text-xs text-red-700 bg-red-50 border border-red-200 flex items-center justify-between shrink-0">
          <span>{error}</span>
          <button onClick={clearError} className="ml-2 text-red-400 hover:text-red-600 font-bold">✕</button>
        </div>
      )}

      {/* Settings Panel */}
      {showSettings && settings && (
        <div className="mx-4 mt-2 p-4 rounded-lg border border-gray-200 bg-white shrink-0">
          <h3 className="text-sm font-semibold text-gray-700 mb-3">设置</h3>
          <div className="space-y-3">
            <label className="flex items-center justify-between text-sm text-gray-600">
              历史记录上限
              <input
                type="number"
                value={settings.max_history_size}
                onChange={(e) => updateSettings({ ...settings, max_history_size: Number(e.target.value) })}
                className="w-20 px-2 py-1 rounded border border-gray-300 text-sm text-right"
                min={50}
                max={5000}
              />
            </label>
            <label className="flex items-center justify-between text-sm text-gray-600">
              开机自启
              <input
                type="checkbox"
                checked={settings.auto_start}
                onChange={(e) => updateSettings({ ...settings, auto_start: e.target.checked })}
                className="rounded"
              />
            </label>
            <label className="flex items-center justify-between text-sm text-gray-600">
              最小化到托盘
              <input
                type="checkbox"
                checked={settings.minimize_to_tray}
                onChange={(e) => updateSettings({ ...settings, minimize_to_tray: e.target.checked })}
                className="rounded"
              />
            </label>
            <label className="flex items-center justify-between text-sm text-gray-600">
              全局快捷键
              <input
                type="text"
                value={settings.global_shortcut}
                onChange={(e) => updateSettings({ ...settings, global_shortcut: e.target.value })}
                className="w-32 px-2 py-1 rounded border border-gray-300 text-sm text-right"
              />
            </label>
            <label className="flex items-center justify-between text-sm text-gray-600">
              主题
              <select
                value={settings.theme}
                onChange={(e) => updateSettings({ ...settings, theme: e.target.value })}
                className="px-2 py-1 rounded border border-gray-300 text-sm"
              >
                <option value="light">浅色</option>
                <option value="dark">深色</option>
              </select>
            </label>
          </div>
        </div>
      )}

      {/* Search Bar */}
      <div className="px-4 py-3 border-b border-gray-200">
        <SearchBar />
      </div>

      {/* Main Content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        <div className="list-head px-4 py-2.5 flex items-center justify-between">
          <h2 className="text-sm font-medium text-gray-700">剪贴历史</h2>
          <span className="text-gray-400 text-xs">{items.length} 条</span>
        </div>
        <div className="flex-1 overflow-hidden px-4 pb-4">
          <ClipboardList />
        </div>
      </main>
    </div>
  );
}

export default App;
