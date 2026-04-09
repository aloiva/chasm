# CHASM

**C**entralised **H**ub for **A**gent **S**ession **M**anager

A lightweight desktop app for managing AI coding sessions across multiple tools.

## Supported Tools
- **Copilot CLI** — reads `~/.copilot/session-store.db` and `session-state/` folders
- **VS Code Copilot Chat** — reads `state.vscdb` from VS Code workspace storage

## Tech Stack
- **Backend**: Tauri v2 (Rust) — reads SQLite, YAML, JSON
- **Frontend**: Svelte + TypeScript
- **Design**: GitHub Dark theme, fully configurable via `theme.json`

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the source adapter plugin system.
See [docs/DECISIONS.md](docs/DECISIONS.md) for design decisions with measured reasoning.
See [docs/DATA_SOURCES.md](docs/DATA_SOURCES.md) for data format reference.
