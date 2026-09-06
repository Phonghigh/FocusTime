# FocusTime

A Tauri v2 desktop app (React + TypeScript frontend, Rust + SQLite backend) that
tracks how you spend time across **Work**, **Entertainment**, and
**Distraction** categories by watching the active foreground window — and, for
browsers, the active tab's domain.

## How it works

- A background poller reads the active foreground window (process name) every
  ~1s and classifies it via per-profile app rules (`app_rules` table).
- When the tracked foreground app is a browser (`chrome.exe`, `msedge.exe`,
  `firefox.exe`), a companion browser extension reports the active tab's
  domain over native messaging, and classification switches to per-domain
  rules (`domain_rules` table) instead — see [`browser-extension/`](./browser-extension)
  for setup.
- A session engine aggregates time per category live and produces a report
  (totals, timeline, interruption count) when the session ends.

## Project layout

- `src/` — React frontend (setup / live-session / report screens)
- `src-tauri/` — Rust backend: SQLite schema + migrations, active-window
  tracker, session engine, Tauri commands, native-messaging host
  (`focustime.exe --native-host`) and TCP bridge for browser domain updates
- `browser-extension/` — Manifest V3 extension + native messaging host
  manifest for reporting the active tab's domain

## Getting started

```bash
npm install
npm run tauri dev
```

Requires a Rust toolchain (see `src-tauri/Cargo.toml`) and Node.js. On
Windows without MSVC Build Tools, the GNU toolchain
(`x86_64-pc-windows-gnu`) plus a MinGW-w64 linker on `PATH` also works.

For browser domain tracking, follow the one-time setup in
[`browser-extension/README.md`](./browser-extension/README.md).

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
