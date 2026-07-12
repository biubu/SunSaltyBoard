import { useState, useEffect, useRef, useCallback } from "react";
import type { Settings, UpdateInfo } from "../../types";
import { getAppVersion, checkUpdate } from "../../services/api";

interface SettingsPanelProps {
  settings: Settings;
  onUpdate: (settings: Settings) => void;
  onSync: () => void;
  syncStatus: string;
  syncing: boolean;
  theme: Record<string, string>;
}

function SettingRow({
  label,
  theme,
  children,
}: {
  label: string;
  theme: Record<string, string>;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1">
      <div className="text-sm" style={{ color: theme.settingsLabel }}>
        {label}
      </div>
      <div>{children}</div>
    </div>
  );
}

function SectionTitle({ title, theme }: { title: string; theme: Record<string, string> }) {
  return (
    <div className="text-xs font-semibold uppercase tracking-wider mb-2 mt-1 first:mt-0" style={{ color: theme.settingsTitle }}>
      {title}
    </div>
  );
}

const DEBOUNCE_MS = 400;

export function SettingsPanel({ settings, onUpdate, onSync, syncStatus, syncing, theme }: SettingsPanelProps) {
  const inputStyle: React.CSSProperties = {
    background: theme.settingsInputBg,
    color: theme.settingsInputText,
    border: `1px solid ${theme.settingsInputBorder}`,
  };

  const [appVersion, setAppVersion] = useState("");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);

  const [draft, setDraft] = useState<Settings>(settings);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, []);

  const commitDraft = useCallback(
    (next: Settings) => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(() => {
        onUpdate(next);
      }, DEBOUNCE_MS);
    },
    [onUpdate]
  );

  const updateDraft = useCallback(
    (patch: Partial<Settings>) => {
      setDraft((prev) => {
        const next = { ...prev, ...patch };
        commitDraft(next);
        return next;
      });
    },
    [commitDraft]
  );

  // Apply settings immediately (no debounce) - used for checkboxes and selects
  // where a single click represents the user's intent.
  const applyImmediate = useCallback(
    (next: Settings) => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      setDraft(next);
      onUpdate(next);
    },
    [onUpdate]
  );

  useEffect(() => {
    getAppVersion().then(setAppVersion).catch(() => setAppVersion("?"));
  }, []);

  const handleCheckUpdate = async () => {
    setCheckingUpdate(true);
    setUpdateInfo(null);
    try {
      const info = await checkUpdate();
      setUpdateInfo(info);
    } catch {
      setUpdateInfo({ latest_version: null, download_url: null, release_notes: null, error: "检查更新失败" });
    }
    setCheckingUpdate(false);
  };

  return (
    <div className="mx-3 mt-1 p-3 rounded-lg border shrink-0 space-y-3" style={{ background: theme.settingsBg, borderColor: theme.settingsBorder }}>
      {/* General */}
      <div>
        <SectionTitle title="通用" theme={theme} />
        <SettingRow label="历史记录上限" theme={theme}>
          <input
            type="number"
            value={draft.max_history_size}
            onChange={(e) => updateDraft({ max_history_size: Number(e.target.value) })}
            onBlur={() => commitDraft(draft)}
            className="w-full px-2 py-1 rounded text-sm"
            style={inputStyle}
            min={50}
            max={5000}
          />
        </SettingRow>
        <SettingRow label="全局快捷键" theme={theme}>
          <input
            type="text"
            value={draft.global_shortcut}
            onChange={(e) => updateDraft({ global_shortcut: e.target.value })}
            onBlur={() => commitDraft(draft)}
            className="w-full px-2 py-1 rounded text-sm font-mono"
            style={inputStyle}
          />
        </SettingRow>
      </div>

      {/* Behavior */}
      <div>
        <SectionTitle title="行为" theme={theme} />
        <SettingRow label="开机自启" theme={theme}>
          <input
            type="checkbox"
            checked={draft.auto_start}
            onChange={(e) => applyImmediate({ ...draft, auto_start: e.target.checked })}
            className="h-4 w-4 accent-blue-500"
          />
        </SettingRow>
        <SettingRow label="最小化到托盘" theme={theme}>
          <input
            type="checkbox"
            checked={draft.minimize_to_tray}
            onChange={(e) => applyImmediate({ ...draft, minimize_to_tray: e.target.checked })}
            className="h-4 w-4 accent-blue-500"
          />
        </SettingRow>
      </div>

      {/* Appearance */}
      <div>
        <SectionTitle title="外观" theme={theme} />
        <SettingRow label="主题" theme={theme}>
          <select
            value={draft.theme}
            onChange={(e) => applyImmediate({ ...draft, theme: e.target.value })}
            className="w-full px-2 py-1 rounded text-sm"
            style={inputStyle}
          >
            <option value="light">浅色</option>
            <option value="dark">深色</option>
          </select>
        </SettingRow>
      </div>

      {/* Cloud Sync */}
      <div>
        <SectionTitle title="云同步" theme={theme} />
        <SettingRow label="启用同步" theme={theme}>
          <input
            type="checkbox"
            checked={draft.sync_enabled}
            onChange={(e) => applyImmediate({ ...draft, sync_enabled: e.target.checked })}
            className="h-4 w-4 accent-blue-500"
          />
        </SettingRow>
        {draft.sync_enabled && (
          <div className="mt-2 space-y-2">
            <input
              type="text"
              placeholder="服务器地址"
              value={draft.sync_server || ""}
              onChange={(e) => updateDraft({ sync_server: e.target.value || null })}
              onBlur={() => commitDraft(draft)}
              className="w-full px-2 py-1 rounded text-sm"
              style={inputStyle}
            />
            <div className="flex items-center gap-2">
              <button
                onClick={onSync}
                disabled={syncing || !draft.sync_server}
                className="px-3 py-1 rounded text-xs text-white disabled:opacity-40 transition-opacity"
                style={{ background: "#3b82f6" }}
              >
                {syncing ? "同步中..." : "立即同步"}
              </button>
              {syncStatus && (
                <span className="text-xs" style={{ color: theme.settingsLabel }}>{syncStatus}</span>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Updates */}
      <div>
        <SectionTitle title="更新" theme={theme} />
        <div className="text-xs space-y-1.5" style={{ color: theme.settingsLabel }}>
          <div className="flex items-center justify-between">
            <span>当前版本</span>
            <span className="font-mono font-semibold" style={{ color: theme.settingsTitle }}>
              {appVersion || "..."}
            </span>
          </div>
          <SettingRow label="更新服务器" theme={theme}>
            <input
              type="text"
              placeholder="https://example.com/updates"
              value={draft.update_server_url || ""}
              onChange={(e) => updateDraft({ update_server_url: e.target.value || null })}
              onBlur={() => commitDraft(draft)}
              className="w-full px-2 py-1 rounded text-sm"
              style={inputStyle}
            />
          </SettingRow>
          <div className="flex items-center gap-2 pt-1">
            <button
              onClick={handleCheckUpdate}
              disabled={checkingUpdate || !draft.update_server_url}
              className="px-3 py-1 rounded text-xs text-white disabled:opacity-40 transition-opacity"
              style={{ background: "#8b5cf6" }}
            >
              {checkingUpdate ? "检查中..." : "检查更新"}
            </button>
          </div>
          {updateInfo && (
            <div className="mt-1 text-xs" style={{ color: theme.settingsLabel }}>
              {updateInfo.error ? (
                <span style={{ color: "#ef4444" }}>{updateInfo.error}</span>
              ) : updateInfo.latest_version ? (
                <div className="space-y-1">
                  {updateInfo.latest_version !== appVersion ? (
                    <>
                      <span style={{ color: "#10b981" }}>
                        新版本可用: v{updateInfo.latest_version}
                      </span>
                      {updateInfo.download_url && (
                        <div>
                          <a
                            href={updateInfo.download_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="underline"
                            style={{ color: "#3b82f6" }}
                          >
                            下载更新
                          </a>
                        </div>
                      )}
                    </>
                  ) : (
                    <span style={{ color: "#10b981" }}>已是最新版本</span>
                  )}
                </div>
              ) : (
                <span>未能获取版本信息</span>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}