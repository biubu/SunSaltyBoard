import { useStore } from "../../store";
import { formatTimeAgo, truncateText, getContentTypeIcon } from "../../utils";
import type { ClipboardItem } from "../../types";
import * as api from "../../services/api";

export function ClipboardList() {
  const { items, isLoading, deleteItem } = useStore();

  const handlePaste = async (item: ClipboardItem) => {
    try {
      await api.pasteToActive(item);
    } catch (e) {
      console.error("Paste failed:", e);
    }
  };

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
        <p className="text-sm">暂无剪贴历史</p>
        <p className="text-xs mt-1">复制内容后将自动记录</p>
      </div>
    );
  }

  return (
    <div className="space-y-3 pt-2">
      {items.map((item) => (
        <div
          key={item.id}
          className="rounded-lg p-4 cursor-pointer transition-all duration-200 hover:bg-gray-100 relative"
          style={{
            background: '#ffffff',
            border: '1px solid #e5e7eb',
            margin: '10px',
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
              deleteItem(item.id);
            }}
            className="absolute right-3 top-1/2 -translate-y-1/2 w-6 h-6 flex items-center justify-center text-gray-300 hover:text-red-500 transition-colors text-lg font-bold"
            title="删除"
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
