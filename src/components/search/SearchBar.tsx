import { useState, useCallback } from "react";
import { useStore } from "../../store";

export function SearchBar() {
  const { searchQuery, setSearchQuery, search } = useStore();
  const [localQuery, setLocalQuery] = useState(searchQuery);

  const handleSearch = useCallback(
    (value: string) => {
      setLocalQuery(value);
      setSearchQuery(value);
      search(value);
    },
    [setSearchQuery, search]
  );

  return (
    <div className="relative" style={{ margin: '10px' }}>
      <input
        type="text"
        value={localQuery}
        onChange={(e) => handleSearch(e.target.value)}
        placeholder="搜索剪贴历史..."
        className="w-full rounded-lg px-4 py-2.5 text-sm text-gray-800 placeholder-gray-400 transition-all duration-200"
        style={{
          background: '#ffffff',
          border: '1px solid #d1d5db',
          padding: '10px',
        }}
      />
      {localQuery && (
        <button
          onClick={() => handleSearch("")}
          className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 transition-colors"
        >
          ✕
        </button>
      )}
    </div>
  );
}
