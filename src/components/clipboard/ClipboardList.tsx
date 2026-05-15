import { useRef, useState, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useStore } from "../../store";
import { formatTimeAgo, truncateText } from "../../utils";
import { ContextMenu } from "./ContextMenu";
import type { ClipboardItem } from "../../types";
import * as api from "../../services/api";

const ITEM_HEIGHT = 44;

interface ClipboardListProps {
  theme: Record<string, string>;
}

export function ClipboardList({ theme }: ClipboardListProps) {
  const { items, isLoading, deleteItem, toggleFavorite } = useStore();
  const scrollRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; item: ClipboardItem } | null>(null);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ITEM_HEIGHT,
    overscan: 15,
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

  const emptyText = theme.emptyText || "#9ca3af";

  if (isLoading && items.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-sm" style={{ color: emptyText }}>加载中...</div>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full" style={{ color: emptyText }}>
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
              className="cursor-pointer transition-colors duration-150 relative hover:bg-gray-50"
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${virtualRow.start}px)`,
                background: focusedIndex === virtualRow.index ? theme.itemFocus : item.is_favorite ? (theme.isDark ? "#2a2518" : "#fffdf5") : "transparent",
                borderBottom: `1px solid ${theme.itemBorder}`,
                borderLeft: item.is_favorite ? "3px solid #f59e0b" : "3px solid transparent",
                padding: "8px 12px",
              }}
              onClick={() => handlePaste(item)}
              onContextMenu={(e) => {
                e.preventDefault();
                setCtxMenu({ x: e.clientX, y: e.clientY, item });
              }}
            >
              <div className="flex items-center gap-2 min-w-0">
                <p className="flex-1 text-[13px] leading-snug truncate" style={{ color: theme.itemText }}>
                  {truncateText(item.preview, 80)}
                </p>
                <span className="shrink-0 text-[11px] whitespace-nowrap" style={{ color: theme.itemTime }}>{formatTimeAgo(item.created_at)}</span>
              </div>
            </div>
          );
        })}
      </div>

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
