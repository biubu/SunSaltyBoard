import { useState, useCallback, useEffect, useRef } from "react";
import { useStore } from "../../store";

export function SearchBar() {
  const { searchQuery, setSearchQuery, search } = useStore();
  const [localQuery, setLocalQuery] = useState(searchQuery);
  
  // Debounce search to avoid excessive API calls
  const debounceRef = useRef<NodeJS.Timeout | null>(null);
  
  const handleSearch = useCallback(
    (value: string) => {
      setLocalQuery(value);
      setSearchQuery(value);
      
      // Clear previous timeout
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      
      // Debounce search by 300ms
      debounceRef.current = setTimeout(() => {
        search(value);
      }, 300);
    },
    [setSearchQuery, search]
  );
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

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
