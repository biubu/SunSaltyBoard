import { useRef, useState, useCallback, memo } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useStore } from "../../store";
import { formatTimeAgo, truncateText } from "../../utils";
import type { ClipboardItem, ContentType } from "../../types";

const ITEM_HEIGHT = 56;

interface ClipboardListProps {
  theme: Record<string, string>;
}

const TYPE_BADGE: Record<
  ContentType,
  { label: string; color: string; bg: string }
> = {
  text: { label: "TEXT", color: "#60a5fa", bg: "rgba(59, 130, 246, 0.15)" },
  html: { label: "HTML", color: "#34d399", bg: "rgba(16, 185, 129, 0.15)" },
  image: { label: "IMG", color: "#c084fc", bg: "rgba(168, 85, 247, 0.15)" },
  file: { label: "FILE", color: "#fbbf24", bg: "rgba(245, 158, 11, 0.15)" },
  rtf: { label: "RTF", color: "#fb923c", bg: "rgba(249, 115, 22, 0.15)" },
  unknown: { label: "UNK", color: "#9ca3af", bg: "rgba(156, 163, 175, 0.15)" },
};

function StarIcon({ filled }: { filled: boolean }) {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill={filled ? "#f59e0b" : "none"}
      stroke={filled ? "#f59e0b" : "currentColor"}
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
      <path d="M10 11v6" />
      <path d="M14 11v6" />
      <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
    </svg>
  );
}

export const ClipboardList = memo(function ClipboardList({ theme }: ClipboardListProps) {
  const { items, isLoading, deleteItem, pasteToActive, toggleFavorite } = useStore();
  const scrollRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ITEM_HEIGHT,
    overscan: 15,
  });

  const handlePaste = useCallback(async (item: ClipboardItem) => {
    await pasteToActive(item);
  }, [pasteToActive]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
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
    },
    [items, focusedIndex, virtualizer, handlePaste, deleteItem]
  );

  const emptyText = theme.emptyText || "#9ca3af";

  if (isLoading && items.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="fs-lg" style={{ color: emptyText }}>加载中...</div>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div
        className="flex flex-col items-center justify-center h-full"
        style={{ color: emptyText }}
      >
        <p className="fs-lg">暂无剪贴历史</p>
        <p className="fs-sm mt-1">复制内容后将自动记录</p>
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
          const isFocused = focusedIndex === virtualRow.index;
          const badge = TYPE_BADGE[item.content_type] ?? TYPE_BADGE.text;

          return (
            <div
              key={item.id}
              data-index={virtualRow.index}
              ref={virtualizer.measureElement}
              role="button"
              tabIndex={0}
              className="group cursor-pointer transition-colors duration-150 relative"
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${virtualRow.start}px)`,
                background: isFocused
                  ? theme.itemFocus
                  : item.is_favorite
                  ? theme.itemFavoriteBg
                  : "transparent",
                borderBottom: `1px solid ${theme.itemBorder}`,
                borderLeft: item.is_favorite
                  ? "3px solid #f59e0b"
                  : "3px solid transparent",
              }}
              onClick={() => handlePaste(item)}
            >
              <div className="flex flex-col gap-1 px-2.5 py-2">
                <div className="flex items-center gap-1.5 min-w-0">
                  {item.content_type !== "text" && (
                    <span
                      className="shrink-0 fs-xs font-bold tracking-wider px-1.5 py-0.5 rounded"
                      style={{ color: badge.color, background: badge.bg }}
                    >
                      {badge.label}
                    </span>
                  )}
                  <p
                    className="flex-1 fs-base leading-snug truncate min-w-0"
                    style={{ color: theme.itemText }}
                  >
                    {truncateText(item.preview, 60)}
                  </p>
                  <div
                    className={`flex items-center gap-0.5 shrink-0 transition-opacity duration-150 ${
                      isFocused
                        ? "opacity-100"
                        : "opacity-0 group-hover:opacity-100"
                    }`}
                  >
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleFavorite(item.id);
                      }}
                      className="w-6 h-6 flex items-center justify-center rounded-md transition-colors hover:bg-black/5"
                      style={{ color: theme.iconText }}
                      title={item.is_favorite ? "取消收藏" : "收藏"}
                      aria-label={item.is_favorite ? "取消收藏" : "收藏"}
                    >
                      <StarIcon filled={item.is_favorite} />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        deleteItem(item.id);
                      }}
                      className="w-6 h-6 flex items-center justify-center rounded-md transition-colors hover:bg-red-500/10"
                      style={{ color: theme.iconText }}
                      title="删除"
                      aria-label="删除"
                      onMouseEnter={(e) =>
                        (e.currentTarget.style.color = "#ef4444")
                      }
                      onMouseLeave={(e) =>
                        (e.currentTarget.style.color = theme.iconText)
                      }
                    >
                      <TrashIcon />
                    </button>
                  </div>
                </div>
                <div
                  className="fs-sm leading-none"
                  style={{ color: theme.itemTime }}
                >
                  {formatTimeAgo(item.created_at)}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
});
