import { useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "./store";
import { SearchBar } from "./components/search/SearchBar";
import { ClipboardList } from "./components/clipboard/ClipboardList";
import type { ClipboardItem } from "./types";

function App() {
  const { loadHistory, loadSettings, items } = useStore();

  useEffect(() => {
    loadHistory();
    loadSettings();

    const unlistenPromise = listen<ClipboardItem>("clipboard-changed", (event) => {
      console.log("Clipboard changed:", event.payload);
      loadHistory();
    });

    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, []); // Empty dependency array - loadHistory and loadSettings are stable

  const hideWindow = useCallback(async () => {
    await invoke("hide_window");
  }, []);

  return (
    <div className="h-screen text-text flex flex-col overflow-hidden" style={{ background: '#f3f4f6', overflow: 'hidden' }}>
      {/* Title Bar */}
      <header
        className="flex items-center justify-between px-4 py-3 border-b border-gray-200"
        data-tauri-drag-region
      >
        <div className="flex items-center gap-2" data-tauri-drag-region>
          <span className="text-xl" data-tauri-drag-region>📋</span>
          <span className="text-sm font-semibold text-gray-700 tracking-wide" data-tauri-drag-region>SunSaltyBoard</span>
        </div>
        <button
          onClick={hideWindow}
          className="w-7 h-7 flex items-center justify-center rounded text-gray-500 hover:text-gray-700 transition-colors"
        >
          ✕
        </button>
      </header>

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
        <div className="flex-1 overflow-y-auto px-4 pb-4">
          <ClipboardList />
        </div>
      </main>
    </div>
  );
}

export default App;
