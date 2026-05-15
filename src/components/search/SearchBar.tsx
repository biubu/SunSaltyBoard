import { useState, useCallback, useRef } from "react";
import { useStore } from "../../store";

interface SearchBarProps {
  theme: Record<string, string>;
}

export function SearchBar({ theme }: SearchBarProps) {
  const { searchQuery, setSearchQuery, search } = useStore();
  const [localQuery, setLocalQuery] = useState(searchQuery);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const handleSearch = useCallback(
    (value: string) => {
      setLocalQuery(value);
      setSearchQuery(value);
      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(() => search(value), 300);
    },
    [setSearchQuery, search]
  );

  return (
    <div className="relative">
      <input
        type="text"
        value={localQuery}
        onChange={(e) => handleSearch(e.target.value)}
        placeholder="搜索剪贴历史..."
        className="w-full rounded-lg px-4 py-2 text-sm transition-all duration-200"
        style={{
          background: theme.searchBg,
          color: theme.searchText,
          border: `1px solid ${theme.searchBorder}`,
        }}
      />
      {localQuery && (
        <button
          onClick={() => handleSearch("")}
          className="absolute right-3 top-1/2 -translate-y-1/2 transition-colors"
          style={{ color: theme.searchPlaceholder }}
        >
          ✕
        </button>
      )}
    </div>
  );
}
