import clsx from "clsx";

interface ContextMenuProps {
  x: number;
  y: number;
  onClose: () => void;
  items: {
    label: string;
    onClick: () => void;
    danger?: boolean;
    disabled?: boolean;
  }[];
}

export function ContextMenu({ x, y, onClose, items }: ContextMenuProps) {
  return (
    <>
      <div
        className="fixed inset-0 z-50"
        onClick={onClose}
        onContextMenu={(e) => { e.preventDefault(); onClose(); }}
      />
      <div
        className="fixed z-50 rounded-lg py-1 shadow-lg border border-gray-200 min-w-[140px]"
        style={{
          left: x,
          top: y,
          background: "#ffffff",
        }}
      >
        {items.map((item, i) => (
          <button
            key={i}
            onClick={() => { item.onClick(); onClose(); }}
            disabled={item.disabled}
            className={clsx(
              "w-full text-left px-3 py-1.5 text-sm transition-colors",
              item.danger
                ? "text-red-600 hover:bg-red-50"
                : "text-gray-700 hover:bg-gray-100",
              item.disabled && "opacity-40 cursor-default"
            )}
          >
            {item.label}
          </button>
        ))}
      </div>
    </>
  );
}
