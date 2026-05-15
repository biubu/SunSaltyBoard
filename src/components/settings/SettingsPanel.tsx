import type { Settings } from "../../types";

interface SettingsPanelProps {
  settings: Settings;
  onUpdate: (settings: Settings) => void;
  onSync: () => void;
  syncStatus: string;
  syncing: boolean;
  theme: Record<string, string>;
}

export function SettingsPanel({ settings, onUpdate, onSync, syncStatus, syncing, theme }: SettingsPanelProps) {
  const inputStyle: React.CSSProperties = {
    background: theme.settingsInputBg,
    color: theme.settingsInputText,
    border: `1px solid ${theme.settingsInputBorder}`,
  };

  return (
    <div className="mx-3 mt-1 p-4 rounded-lg border shrink-0" style={{ background: theme.settingsBg, borderColor: theme.settingsBorder }}>
      <h3 className="text-sm font-semibold mb-3" style={{ color: theme.settingsTitle }}>设置</h3>
      <div className="space-y-2.5">
        <label className="flex items-center justify-between text-sm" style={{ color: theme.settingsLabel }}>
          历史记录上限
          <input
            type="number"
            value={settings.max_history_size}
            onChange={(e) => onUpdate({ ...settings, max_history_size: Number(e.target.value) })}
            className="w-20 px-2 py-1 rounded text-sm text-right"
            style={inputStyle}
            min={50}
            max={5000}
          />
        </label>
        <label className="flex items-center justify-between text-sm" style={{ color: theme.settingsLabel }}>
          开机自启
          <input
            type="checkbox"
            checked={settings.auto_start}
            onChange={(e) => onUpdate({ ...settings, auto_start: e.target.checked })}
          />
        </label>
        <label className="flex items-center justify-between text-sm" style={{ color: theme.settingsLabel }}>
          最小化到托盘
          <input
            type="checkbox"
            checked={settings.minimize_to_tray}
            onChange={(e) => onUpdate({ ...settings, minimize_to_tray: e.target.checked })}
          />
        </label>
        <label className="flex items-center justify-between text-sm" style={{ color: theme.settingsLabel }}>
          全局快捷键
          <input
            type="text"
            value={settings.global_shortcut}
            onChange={(e) => onUpdate({ ...settings, global_shortcut: e.target.value })}
            className="w-32 px-2 py-1 rounded text-sm text-right"
            style={inputStyle}
          />
        </label>
        <label className="flex items-center justify-between text-sm" style={{ color: theme.settingsLabel }}>
          主题
          <select
            value={settings.theme}
            onChange={(e) => onUpdate({ ...settings, theme: e.target.value })}
            className="px-2 py-1 rounded text-sm"
            style={inputStyle}
          >
            <option value="light">浅色</option>
            <option value="dark">深色</option>
          </select>
        </label>

        {/* Cloud Sync */}
        <div className="pt-2 mt-2" style={{ borderTop: `1px solid ${theme.settingsBorder}` }}>
          <label className="flex items-center justify-between text-sm" style={{ color: theme.settingsLabel }}>
            启用云同步
            <input
              type="checkbox"
              checked={settings.sync_enabled}
              onChange={(e) => onUpdate({ ...settings, sync_enabled: e.target.checked })}
            />
          </label>
          {settings.sync_enabled && (
            <>
              <input
                type="text"
                placeholder="服务器地址"
                value={settings.sync_server || ""}
                onChange={(e) => onUpdate({ ...settings, sync_server: e.target.value || null })}
                className="w-full mt-2 px-2 py-1 rounded text-sm"
                style={inputStyle}
              />
              <div className="flex items-center gap-2 mt-2">
                <button
                  onClick={onSync}
                  disabled={syncing || !settings.sync_server}
                  className="px-3 py-1 rounded text-xs text-white disabled:opacity-40"
                  style={{ background: "#3b82f6" }}
                >
                  {syncing ? "同步中..." : "立即同步"}
                </button>
                {syncStatus && <span className="text-xs" style={{ color: theme.settingsLabel }}>{syncStatus}</span>}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
