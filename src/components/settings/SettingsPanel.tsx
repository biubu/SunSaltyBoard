import type { Settings } from "../../types";

interface SettingsPanelProps {
  settings: Settings;
  onUpdate: (settings: Settings) => void;
}

export function SettingsPanel({ settings, onUpdate }: SettingsPanelProps) {
  return (
    <div className="mx-4 mt-2 p-4 rounded-lg border border-gray-200 bg-white shrink-0">
      <h3 className="text-sm font-semibold text-gray-700 mb-3">设置</h3>
      <div className="space-y-3">
        <label className="flex items-center justify-between text-sm text-gray-600">
          历史记录上限
          <input
            type="number"
            value={settings.max_history_size}
            onChange={(e) => onUpdate({ ...settings, max_history_size: Number(e.target.value) })}
            className="w-20 px-2 py-1 rounded border border-gray-300 text-sm text-right"
            min={50}
            max={5000}
          />
        </label>
        <label className="flex items-center justify-between text-sm text-gray-600">
          开机自启
          <input
            type="checkbox"
            checked={settings.auto_start}
            onChange={(e) => onUpdate({ ...settings, auto_start: e.target.checked })}
            className="rounded"
          />
        </label>
        <label className="flex items-center justify-between text-sm text-gray-600">
          最小化到托盘
          <input
            type="checkbox"
            checked={settings.minimize_to_tray}
            onChange={(e) => onUpdate({ ...settings, minimize_to_tray: e.target.checked })}
            className="rounded"
          />
        </label>
        <label className="flex items-center justify-between text-sm text-gray-600">
          全局快捷键
          <input
            type="text"
            value={settings.global_shortcut}
            onChange={(e) => onUpdate({ ...settings, global_shortcut: e.target.value })}
            className="w-32 px-2 py-1 rounded border border-gray-300 text-sm text-right"
          />
        </label>
        <label className="flex items-center justify-between text-sm text-gray-600">
          主题
          <select
            value={settings.theme}
            onChange={(e) => onUpdate({ ...settings, theme: e.target.value })}
            className="px-2 py-1 rounded border border-gray-300 text-sm"
          >
            <option value="light">浅色</option>
            <option value="dark">深色</option>
          </select>
        </label>
      </div>
    </div>
  );
}
