import { useStore } from "../../store";
import { formatTimeAgo, truncateText, getContentTypeIcon } from "../../utils";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import type { ClipboardItem } from "../../types";
import * as api from "../../services/api";

function ClipboardItemCard({ item, onDelete }: { item: ClipboardItem; onDelete: (id: string) => void }) {
  const handlePaste = async (item: ClipboardItem) => {
    try {
      await api.pasteToActive(item);
    } catch (e) {
      console.error("Paste failed:", e);
    }
  };

  return (
    <div
      className="rounded-lg cursor-pointer transition-all duration-200 hover:bg-gray-100 relative"
      style={{
        background: '#ffffff',
        border: '1px solid #e5e7eb',
        padding: '15px',
      }}
      onClick={() => handlePaste(item)}
    >
      <div className="flex items-start gap-3 pr-8">
        <span className="text-xl shrink-0">{getContentTypeIcon(item.content_type)}</span>
        <div className="flex-1 min-w-0">
          <p className="text-gray-800 text-sm leading-relaxed line-clamp-2">{truncateText(item.preview, 120)}</p>
          <div className="flex items-center gap-3 mt-1.5">
            <span className="text-xs text-gray-400">{formatTimeAgo(item.created_at)}</span>
          </div>
        </div>
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onDelete(item.id);
        }}
        className="absolute right-3 top-1/2 -translate-y-1/2 w-6 h-6 flex items-center justify-center text-gray-300 hover:text-red-500 transition-colors text-lg font-bold"
        title="删除"
      >
        ×
      </button>
    </div>
  );
}

export function ClipboardList() {
  const { items, isLoading, deleteItem } = useStore();
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 100,
    overscan: 5,
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-400">加载中...</div>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400">
        <div className="text-6xl mb-4">📋</div>
        <p className="text-sm font-medium">暂无剪贴历史</p>
        <p className="text-xs mt-2 text-gray-500">复制内容后将自动记录</p>
      </div>
    );
  }

  return (
    <div
      ref={parentRef}
      className="h-full overflow-auto"
      style={{ margin: '0 -16px -16px' }}
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const item = items[virtualRow.index];
          return (
            <div
              key={item.id}
              data-index={virtualRow.index}
              ref={virtualizer.measureElement}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                transform: `translateY(${virtualRow.start}px)`,
                padding: '0 16px 8px',
              }}
            >
              <ClipboardItemCard item={item} onDelete={deleteItem} />
            </div>
          );
        })}
      </div>
    </div>
  );
}
