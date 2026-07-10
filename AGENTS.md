# SunSaltyBoard

Tauri v2 + React 19 + TypeScript + Tailwind CSS v4 + Zustand + SQLite (rusqlite)  
Floating clipboard manager for Windows, macOS, and Linux.

## Quick start

```bash
npm install
npm run tauri dev      # full Tauri dev (Vite on :1420, HMR :1421)
npm run tauri build    # production build (output: src-tauri/target/)
npm run build          # tsc + vite build only (no Tauri)
npm run dev            # Vite-only (no Rust backend)
```

## Architecture

```
src/              — React SPA (Vite, port 1420)
  store/index.ts  — Zustand, single global store
  services/api.ts — invoke() wrappers for every Tauri command
  types/index.ts  — shared TS interfaces mirroring Rust structs
src-tauri/src/    — Rust backend
  lib.rs          — app setup, tray, global shortcut, commands registered
  main.rs         — entrypoint, hides console on Windows release
  commands/       — Tauri command handlers
  database/       — SQLite via rusqlite (bundled), FTS5 full-text search
  clipboard/      — polling-based clipboard monitor (arboard)
  settings/       — Settings struct, persisted in DB
  sync/           — HTTP POST sync to configurable server
  autostart.rs    — Windows/Linux/macOS autostart support
```

## Key quirks

- **Window is invisible by default** (`visible: false`). Shown near active window (Windows) or cursor (Linux/macOS) via global hotkey `Ctrl+Shift+V` or tray icon click.
- **No test framework** installed. No lint/formatter config beyond `tsc --noEmit`.
- **Tailwind v4** — uses `@import "tailwindcss"` (no `postcss.config.js` or `tailwind.config.*`).
- **Dev server on fixed port 1420** (strict). Vite ignores `src-tauri/**` in file watcher.
- **Tauri capabilities** minimal (only `core:default` + `opener:default`). Add permissions here when adding plugins: `src-tauri/capabilities/default.json`
- **Database**: `clipstash.db` in Tauri app data dir. Schema auto-created on first run. FTS5 virtual table for search.
- **Clipboard monitoring** uses polling (500ms idle, 200ms on change), not OS events.
- **Sync** sends POST with JSON payload of clipboard items to `sync_server` URL.
- **Updates** checks `update_server_url` for new versions; version and check button in settings.
- **Cross-platform**: Windows (Win32 API), Linux (enigo), macOS (enigo + CoreGraphics).
- Bundle targets: `nsis` (Windows), `dmg` (macOS), `deb` / `appimage` (Linux).

## Release

```bash
git tag v26.5.x
git push origin v26.5.x
# GitHub Action builds + uploads to release
```

## Verification

```bash
npm run build        # tsc typecheck + vite build (no tests available)
```

No test, lint, or format commands exist. TypeScript strict mode with `noUnusedLocals`/`noUnusedParameters` catches most issues at build time.
