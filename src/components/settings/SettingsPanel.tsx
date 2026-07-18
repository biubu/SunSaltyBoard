import { useState, useEffect, useRef, useCallback } from "react";
import type { Settings } from "../../types";
import { HotkeyRecorder } from "./HotkeyRecorder";
import { getAppVersion } from "../../services/api";

interface SettingsPanelProps {
  settings: Settings;
  onUpdate: (settings: Settings) => void;
  theme: Record<string, string>;
  onClose: () => void;
}

const DEBOUNCE_MS = 400;

function Toggle({
  checked,
  onChange,
  theme,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  theme: Record<string, string>;
}) {
  const trackBg = checked
    ? theme.accent ?? "#3b82f6"
    : theme.toggleTrack ?? "#d1d5db";
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-1"
      style={{
        background: trackBg,
        boxShadow: "inset 0 1px 2px rgba(0,0,0,0.15)",
      }}
    >
      <span
        className="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200"
        style={{
          transform: checked ? "translateX(18px)" : "translateX(2px)",
          marginTop: "2px",
          boxShadow: "0 1px 3px rgba(0,0,0,0.3)",
        }}
      />
    </button>
  );
}

function Icon({ name }: { name: string }) {
  const common = {
    xmlns: "http://www.w3.org/2000/svg",
    width: 14,
    height: 14,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 2,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
  };
  switch (name) {
    case "general":
      return (
        <svg {...common}>
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      );
    case "behavior":
      return (
        <svg {...common}>
          <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
        </svg>
      );
    case "appearance":
      return (
        <svg {...common}>
          <circle cx="13.5" cy="6.5" r=".5" />
          <circle cx="17.5" cy="10.5" r=".5" />
          <circle cx="8.5" cy="7.5" r=".5" />
          <circle cx="6.5" cy="12.5" r=".5" />
          <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z" />
        </svg>
      );
    case "back":
      return (
        <svg {...common}>
          <line x1="19" y1="12" x2="5" y2="12" />
          <polyline points="12 19 5 12 12 5" />
        </svg>
      );
    case "about":
      return (
        <svg {...common}>
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="16" x2="12" y2="12" />
          <line x1="12" y1="8" x2="12.01" y2="8" />
        </svg>
      );
    default:
      return null;
  }
}

function Card({
  title,
  subtitle,
  iconName,
  theme,
  children,
}: {
  title: string;
  subtitle?: string;
  iconName: string;
  theme: Record<string, string>;
  children: React.ReactNode;
}) {
  return (
    <section
      className="rounded-xl border overflow-hidden flex-shrink-0"
      style={{
        background: theme.settingsBg,
        borderColor: theme.settingsBorder,
        boxShadow: theme.settingsCardShadow,
      }}
    >
      <header
        className="px-4 py-2.5 border-b"
        style={{
          background: theme.settingsCardHeaderBg,
          borderColor: theme.settingsBorder,
        }}
      >
        <div className="flex items-center gap-2">
          <span style={{ color: theme.accent }}>
            <Icon name={iconName} />
          </span>
          <span
            className="fs-sm font-semibold tracking-wider uppercase"
            style={{ color: theme.settingsTitle }}
          >
            {title}
          </span>
        </div>
        {subtitle && (
          <div
            className="fs-sm mt-0.5 leading-relaxed"
            style={{ color: theme.settingsHint }}
          >
            {subtitle}
          </div>
        )}
      </header>
      <div className="px-4 py-3 flex flex-col gap-3">{children}</div>
    </section>
  );
}

function Field({
  label,
  description,
  theme,
  children,
}: {
  label: string;
  description?: string;
  theme: Record<string, string>;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div
        className="fs-base font-medium leading-snug"
        style={{ color: theme.settingsLabel }}
      >
        {label}
      </div>
      {description && (
        <div
          className="fs-sm mt-1 leading-relaxed"
          style={{ color: theme.settingsHint }}
        >
          {description}
        </div>
      )}
      <div className="mt-2 flex justify-end">{children}</div>
    </div>
  );
}

export function SettingsPanel({
  settings,
  onUpdate,
  theme,
  onClose,
}: SettingsPanelProps) {
  const [draft, setDraft] = useState<Settings>(settings);
  const [appVersion, setAppVersion] = useState<string>("");
  const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined
  );

  useEffect(() => {
    getAppVersion().then(setAppVersion).catch(() => {});
  }, []);

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

  const applyImmediate = useCallback(
    (next: Settings) => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      setDraft(next);
      onUpdate(next);
    },
    [onUpdate]
  );

  return (
    <div
      className="flex-1 flex flex-col overflow-hidden"
      style={{ background: theme.bg }}
    >
      {/* Settings Header */}
      <header
        className="flex items-center justify-between px-4 py-3 border-b shrink-0"
        style={{ background: theme.headerBg, borderColor: theme.settingsBorder }}
      >
        <div className="flex items-center gap-2">
          <button
            onClick={onClose}
            className="w-7 h-7 flex items-center justify-center rounded-md transition-colors hover:bg-black/5"
            style={{ color: theme.iconText }}
            title="返回"
            aria-label="返回"
          >
            <Icon name="back" />
          </button>
          <span
            className="fs-lg font-semibold tracking-wide"
            style={{ color: theme.titleText }}
          >
            设置
          </span>
        </div>
        <button
          onClick={onClose}
          className="w-7 h-7 flex items-center justify-center rounded-md transition-colors hover:bg-black/5"
          style={{ color: theme.iconText }}
          title="关闭"
          aria-label="关闭"
        >
          ✕
        </button>
      </header>

      {/* Settings Body (scrollable) */}
      <div className="flex-1 overflow-y-auto p-2 flex flex-col gap-2">
        {/* General */}
        <Card
          title="通用"
          subtitle="历史容量与全局快捷键"
          iconName="general"
          theme={theme}
        >
          <Field
            label="历史记录上限"
            description="最多保留多少条剪贴记录(50–5000)"
            theme={theme}
          >
            <input
              type="number"
              value={draft.max_history_size}
              min={50}
              max={5000}
              onChange={(e) =>
                updateDraft({ max_history_size: Number(e.target.value) || 0 })
              }
              onBlur={() => commitDraft(draft)}
              className="w-24 px-2.5 py-1.5 rounded-md fs-lg outline-none text-right transition-shadow focus:ring-2 focus:ring-blue-500/30"
              style={{
                background: theme.settingsInputBg,
                color: theme.settingsInputText,
                border: `1px solid ${theme.settingsInputBorder}`,
              }}
            />
          </Field>
          <Field
            label="全局快捷键"
            description="随时唤出剪贴板管理面板 · 点击右侧录制并按下组合键"
            theme={theme}
          >
            <HotkeyRecorder
              value={draft.global_shortcut}
              onChange={(v) => updateDraft({ global_shortcut: v })}
              theme={theme}
            />
          </Field>
        </Card>

        {/* Behavior */}
        <Card
          title="行为"
          subtitle="启动与窗口行为"
          iconName="behavior"
          theme={theme}
        >
          <Field
            label="开机自启"
            description="登录系统后自动启动 SunSaltyBoard"
            theme={theme}
          >
            <Toggle
              checked={draft.auto_start}
              onChange={(v) => applyImmediate({ ...draft, auto_start: v })}
              theme={theme}
            />
          </Field>
          <Field
            label="最小化到托盘"
            description="关闭主窗口时不退出程序"
            theme={theme}
          >
            <Toggle
              checked={draft.minimize_to_tray}
              onChange={(v) =>
                applyImmediate({ ...draft, minimize_to_tray: v })
              }
              theme={theme}
            />
          </Field>
        </Card>

        {/* Clipboard Monitor */}
        <Card
          title="剪贴板监控"
          subtitle="剪贴板轮询与远程桌面兼容"
          iconName="behavior"
          theme={theme}
        >
          <Field
            label="启用监控"
            description="关闭后不读取系统剪贴板"
            theme={theme}
          >
            <Toggle
              checked={draft.clipboard_monitor_enabled}
              onChange={(v) =>
                applyImmediate({ ...draft, clipboard_monitor_enabled: v })
              }
              theme={theme}
            />
          </Field>
          <Field
            label="轮询间隔(ms)"
            description="每隔多少毫秒检查一次剪贴板变化 (200–10000)"
            theme={theme}
          >
            <input
              type="number"
              value={draft.clipboard_poll_interval_ms}
              min={200}
              max={10000}
              step={100}
              onChange={(e) =>
                updateDraft({
                  clipboard_poll_interval_ms: Number(e.target.value) || 2000,
                })
              }
              onBlur={() => commitDraft(draft)}
              className="w-24 px-2.5 py-1.5 rounded-md fs-lg outline-none text-right transition-shadow focus:ring-2 focus:ring-blue-500/30"
              style={{
                background: theme.settingsInputBg,
                color: theme.settingsInputText,
                border: `1px solid ${theme.settingsInputBorder}`,
              }}
            />
          </Field>
          <Field
            label="监控模式"
            description="自适应: 远程桌面下暂停读取 · 始终轮询: 传统模式"
            theme={theme}
          >
            <div
              className="inline-flex rounded-md overflow-hidden border fs-sm"
              style={{ borderColor: theme.settingsInputBorder }}
            >
              {[
                { v: "adaptive", label: "自适应" },
                { v: "poll", label: "始终轮询" },
              ].map((opt, idx) => {
                const active = draft.clipboard_monitor_mode === opt.v;
                return (
                  <button
                    key={opt.v}
                    onClick={() =>
                      applyImmediate({
                        ...draft,
                        clipboard_monitor_mode: opt.v,
                      })
                    }
                    className="px-3 py-1.5 transition-colors"
                    style={{
                      background: active
                        ? theme.accent
                        : theme.settingsInputBg,
                      color: active ? "#ffffff" : theme.settingsInputText,
                      borderLeft:
                        idx > 0
                          ? `1px solid ${theme.settingsInputBorder}`
                          : "none",
                    }}
                  >
                    {opt.label}
                  </button>
                );
              })}
            </div>
          </Field>
        </Card>

        {/* Appearance */}
        <Card
          title="外观"
          subtitle="主题配色"
          iconName="appearance"
          theme={theme}
        >
          {/* Font Size */}
          <div className="mb-1">
            <div className="fs-base font-medium leading-snug" style={{ color: theme.settingsLabel }}>
              字体大小
            </div>
            <div className="mt-2 px-1">
              <div className="flex items-center gap-2 mb-1">
                <span className="leading-none" style={{ color: theme.settingsHint, fontSize: 10 }}>A</span>
                <input
                  type="range"
                  min={1}
                  max={5}
                  step={1}
                  value={draft.font_size}
                  onChange={(e) => {
                    updateDraft({ font_size: Number(e.target.value) });
                  }}
                  className="flex-1 h-1.5 rounded-full appearance-none cursor-pointer"
                  style={{
                    background: `linear-gradient(to right, ${theme.accent} 0%, ${theme.accent} ${(draft.font_size - 1) * 25}%, ${theme.settingsInputBorder} ${(draft.font_size - 1) * 25}%, ${theme.settingsInputBorder} 100%)`,
                    WebkitAppearance: "none",
                    accentColor: theme.accent,
                  }}
                />
                <span className="leading-none" style={{ color: theme.settingsHint, fontSize: 18 }}>A</span>
              </div>
              <div className="flex justify-between px-0.5">
                {["极小", "小", "默认", "大", "极大"].map((label, i) => (
                  <span
                    key={label}
                    className="fs-xs"
                    style={{
                      color: draft.font_size === i + 1 ? theme.accent : theme.settingsHint,
                    }}
                  >
                    {label}
                  </span>
                ))}
              </div>
            </div>
          </div>
          <Field label="主题" theme={theme}>
            <div
              className="inline-flex rounded-md overflow-hidden border fs-sm"
              style={{ borderColor: theme.settingsInputBorder }}
            >
              {[
                { v: "light", label: "浅色" },
                { v: "dark", label: "深色" },
              ].map((opt, idx) => {
                const active = draft.theme === opt.v;
                return (
                  <button
                    key={opt.v}
                    onClick={() => applyImmediate({ ...draft, theme: opt.v })}
                    className="px-3 py-1.5 transition-colors"
                    style={{
                      background: active
                        ? theme.accent
                        : theme.settingsInputBg,
                      color: active ? "#ffffff" : theme.settingsInputText,
                      borderLeft:
                        idx > 0
                          ? `1px solid ${theme.settingsInputBorder}`
                          : "none",
                    }}
                  >
                    {opt.label}
                  </button>
                );
              })}
            </div>
          </Field>
        </Card>

        {/* About */}
        <Card
          title="关于"
          subtitle=""
          iconName="about"
          theme={theme}
        >
          <div className="fs-sm leading-relaxed" style={{ color: theme.settingsLabel }}>
            SunSaltyBoard v{appVersion || "..."}
          </div>
          <div className="fs-sm leading-relaxed" style={{ color: theme.settingsHint }}>
            A floating clipboard manager for Windows, macOS, and Linux.
          </div>
        </Card>
      </div>
    </div>
  );
}

