# SunSaltyBoard

A floating clipboard manager for Windows, macOS, and Linux.  
Built with Tauri v2 + React 19 + TypeScript + Tailwind CSS v4.

![SunSaltyBoard](./src-tauri/icons/128x128@2x%20.png)

## Features

- Global hotkey (`Ctrl+Shift+V`) to show clipboard panel
- Click item to paste into active window (original clipboard restored)
- Full-text search through clipboard history
- Favorites / grouping / tagging
- Cloud sync (self-hosted HTTP endpoint)
- Dark & light theme
- Auto-start on boot
- Online update checking
- Virtualized list for performance

## Download

Download the latest installer from the [Releases](https://github.com/biubu/SunSaltyBoard/releases) page:

| Platform | Format |
|----------|--------|
| Windows  | `.exe` (NSIS installer) |
| macOS    | `.dmg` |
| Linux    | `.deb` / `.AppImage` |

## Development

```bash
npm install
npm run tauri dev    # Start Tauri dev environment
npm run tauri build   # Production build
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19, Zustand 5, TanStack Virtual |
| Styling | Tailwind CSS v4 |
| Backend | Rust, Tauri v2 |
| Database | SQLite (rusqlite, bundled) |
| Clipboard | arboard (polling-based monitor) |
| Window/Input | Win32 API (Windows), enigo (Linux/macOS) |
