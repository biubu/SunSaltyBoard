import { useRef, useState, useCallback, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useStore } from "../../store";
import { formatTimeAgo, truncateText, getContentTypeIcon } from "../../utils";
import { ContextMenu } from "./ContextMenu";
import type { ClipboardItem } from "../../types";
import * as api from "../../services/api";

const ITEM_HEIGHT = 80;

export function ClipboardList() {
  const { items, isLoading, deleteItem, toggleFavorite, error, clearError } = useStore();
  const scrollRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; item: ClipboardItem } | null>(null);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ITEM_HEIGHT,
    overscan: 5,
  });

  const handlePaste = useCallback(async (item: ClipboardItem) => {
    try {
      await api.pasteToActive(item);
    } catch (e) {
      console.error("Paste failed:", e);
    }
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (items.length === 0) return;
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setFocusedIndex((prev) => {
          const next = prev + 1 >= items.length ? 0 : prev + 1;
          virtualizer.scrollToIndex(next, { align: "auto" });
          return next;
        });
        break;
      case "ArrowUp":
        e.preventDefault();
        setFocusedIndex((prev) => {
          const next = prev - 1 < 0 ? items.length - 1 : prev - 1;
          virtualizer.scrollToIndex(next, { align: "auto" });
          return next;
        });
        break;
      case "Enter":
        e.preventDefault();
        if (focusedIndex >= 0 && focusedIndex < items.length) {
          handlePaste(items[focusedIndex]);
        }
        break;
      case "Delete":
      case "Backspace":
        e.preventDefault();
        if (focusedIndex >= 0 && focusedIndex < items.length) {
          deleteItem(items[focusedIndex].id);
          setFocusedIndex(-1);
        }
        break;
      case "Escape":
        setFocusedIndex(-1);
        break;
    }
  }, [items, focusedIndex, virtualizer, handlePaste, deleteItem]);

  // Auto-dismiss error after 5 seconds
  useEffect(() => {
    if (error) {
      const t = setTimeout(clearError, 5000);
      return () => clearTimeout(t);
    }
  }, [error, clearError]);

  if (isLoading && items.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-400 text-sm">加载中...</div>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400">
        <p className="text-sm">暂无剪贴历史</p>
        <p className="text-xs mt-1">复制内容后将自动记录</p>
      </div>
    );
  }

  return (
    <div
      ref={scrollRef}
      className="h-full overflow-y-auto outline-none"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onBlur={() => setFocusedIndex(-1)}
    >
      {/* Error toast */}
      {error && (
        <div className="mx-2 mt-2 px-3 py-2 rounded-lg text-xs text-red-700 bg-red-50 border border-red-200 flex items-center justify-between">
          <span>{error}</span>
          <button onClick={clearError} className="ml-2 text-red-400 hover:text-red-600">✕</button>
        </div>
      )}

      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const item = items[virtualRow.index];
          return (
            <div
              key={item.id}
              data-index={virtualRow.index}
              ref={virtualizer.measureElement}
              className="rounded-lg cursor-pointer transition-all duration-200 hover:bg-gray-100 relative"
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${virtualRow.start}px)`,
                background: focusedIndex === virtualRow.index ? "#f0f4ff" : "#ffffff",
                border: focusedIndex === virtualRow.index ? "2px solid #93c5fd" : "1px solid #e5e7eb",
                marginBottom: "8px",
                padding: "12px",
              }}
              onClick={() => handlePaste(item)}
              onContextMenu={(e) => {
                e.preventDefault();
                setCtxMenu({ x: e.clientX, y: e.clientY, item });
              }}
            >
              <div className="flex items-start gap-3 pr-6">
                <span className="text-xl shrink-0">{getContentTypeIcon(item.content_type)}</span>
                <div className="flex-1 min-w-0">
                  <p className="text-gray-800 text-sm leading-relaxed line-clamp-2">
                    {truncateText(item.preview, 120)}
                  </p>
                  <div className="flex items-center gap-3 mt-1.5">
                    <span className="text-xs text-gray-400">{formatTimeAgo(item.created_at)}</span>
                  </div>
                </div>
              </div>

              {/* Favorite star */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  toggleFavorite(item.id);
                }}
                className="absolute right-1 top-1 w-6 h-6 flex items-center justify-center text-lg transition-colors"
                title={item.is_favorite ? "取消收藏" : "收藏"}
              >
                {item.is_favorite ? "⭐" : "☆"}
              </button>

              {/* Delete button */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  deleteItem(item.id);
                }}
                className="absolute right-1 bottom-1 w-6 h-6 flex items-center justify-center text-gray-300 hover:text-red-500 transition-colors text-lg font-bold"
                title="删除"
              >
                ×
              </button>
            </div>
          );
        })}
      </div>

      {/* Context Menu */}
      {ctxMenu && (
        <ContextMenu
          x={ctxMenu.x}
          y={ctxMenu.y}
          onClose={() => setCtxMenu(null)}
          items={[
            {
              label: "粘贴到活动窗口",
              onClick: () => handlePaste(ctxMenu.item),
            },
            {
              label: ctxMenu.item.is_favorite ? "取消收藏" : "收藏",
              onClick: () => toggleFavorite(ctxMenu.item.id),
            },
            {
              label: "删除",
              onClick: () => deleteItem(ctxMenu.item.id),
              danger: true,
            },
          ]}
        />
      )}
    </div>
  );
}
