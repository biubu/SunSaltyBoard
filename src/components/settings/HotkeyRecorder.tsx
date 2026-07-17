import { useEffect, useRef, useState } from "react";

const MODIFIER_CODES = new Set(["Control", "Shift", "Alt", "Meta"]);

function codeToToken(code: string): string | null {
  const letter = /^Key([A-Z])$/.exec(code);
  if (letter) return letter[1];

  const digit = /^Digit([0-9])$/.exec(code);
  if (digit) return digit[1];

  const fn = /^F(1[0-9]|2[0-4]|[1-9])$/.exec(code);
  if (fn) return `F${fn[1]}`;

  const numpad = /^Numpad([0-9])$/.exec(code);
  if (numpad) return `Numpad${numpad[1]}`;

  const special: Record<string, string> = {
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
    Backspace: "Backspace",
    Delete: "Delete",
    Insert: "Insert",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Backslash: "\\",
    Semicolon: ";",
    Quote: "'",
    Comma: ",",
    Period: ".",
    Slash: "/",
    Backquote: "`",
  };

  return special[code] ?? null;
}

function buildCombo(e: KeyboardEvent): string | null {
  const main = codeToToken(e.code);
  if (!main) return null;

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  parts.push(main);

  return parts.join("+");
}

function modifierPreview(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  return parts.join("+");
}

interface HotkeyRecorderProps {
  value: string;
  onChange: (v: string) => void;
  theme: Record<string, string>;
}

export function HotkeyRecorder({ value, onChange, theme }: HotkeyRecorderProps) {
  const [recording, setRecording] = useState(false);
  const [preview, setPreview] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const exitTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const onChangeRef = useRef(onChange);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    return () => {
      if (exitTimer.current) clearTimeout(exitTimer.current);
    };
  }, []);

  useEffect(() => {
    if (!recording) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape") {
        setRecording(false);
        setPreview("");
        return;
      }

      if (
        e.code === "Backspace" &&
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        !e.shiftKey
      ) {
        onChangeRef.current("");
        setPreview("已清空");
        if (exitTimer.current) clearTimeout(exitTimer.current);
        exitTimer.current = setTimeout(() => {
          setRecording(false);
          setPreview("");
        }, 400);
        return;
      }

      const combo = buildCombo(e);
      if (combo) {
        onChangeRef.current(combo);
        setPreview(combo);
        if (exitTimer.current) clearTimeout(exitTimer.current);
        exitTimer.current = setTimeout(() => {
          setRecording(false);
          setPreview("");
        }, 280);
        return;
      }

      if (MODIFIER_CODES.has(e.code)) {
        const mod = modifierPreview(e);
        setPreview(mod ? `${mod}+…` : "");
      } else {
        setPreview("不支持的按键");
      }
    };

    const handleBlur = () => {
      setRecording(false);
      setPreview("");
    };

    window.addEventListener("keydown", handleKeyDown, true);
    window.addEventListener("blur", handleBlur);
    return () => {
      window.removeEventListener("keydown", handleKeyDown, true);
      window.removeEventListener("blur", handleBlur);
    };
  }, [recording]);

  useEffect(() => {
    if (!recording) return;

    const handleClick = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setRecording(false);
        setPreview("");
      }
    };

    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [recording]);

  const showPlaceholder = recording && !preview;
  const showLivePreview = recording && !!preview;
  const display = recording ? (preview || "请按下快捷键…(Esc 取消)") : value;

  return (
    <div ref={containerRef} className="flex gap-1.5 w-full">
      <div
        className="flex-1 min-w-0 px-2.5 py-1.5 rounded-md fs-lg font-mono transition-colors flex items-center"
        style={{
          background: recording ? `${theme.accent}14` : theme.settingsInputBg,
          color: showLivePreview
            ? theme.accent
            : showPlaceholder
              ? theme.settingsHint
              : theme.settingsInputText,
          border: `1px solid ${recording ? theme.accent : theme.settingsInputBorder}`,
        }}
      >
        <span className="truncate">{display}</span>
      </div>
      <button
        type="button"
        onClick={() => {
          if (exitTimer.current) clearTimeout(exitTimer.current);
          setRecording((r) => !r);
          setPreview("");
        }}
        className="px-2.5 py-1.5 rounded-md fs-sm transition-colors shrink-0 focus:outline-none focus:ring-2 focus:ring-blue-500/30"
        style={{
          background: recording ? theme.accent : theme.settingsInputBg,
          color: recording ? "#ffffff" : theme.settingsInputText,
          border: `1px solid ${recording ? theme.accent : theme.settingsInputBorder}`,
        }}
      >
        {recording ? "取消" : "录制"}
      </button>
    </div>
  );
}
