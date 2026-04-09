# chasm

**C**entralised **H**ub for **A**gent **S**ession **M**anagement

A lightweight desktop app for managing AI coding sessions across multiple tools.

## Supported Tools
- **Copilot CLI** — reads `~/.copilot/session-store.db` and `session-state/` folders (fully supported: resume, rename, delete, preview)
- **VS Code Copilot Chat** — reads `state.vscdb` from VS Code workspace storage (read-only)

Currently, only the Copilot CLI adapter has full functionality (resume, rename, delete). Other agents or tools will need their own adapter implementing the required traits for each feature. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details on the adapter plugin system.

## Tech Stack
- **Backend**: Tauri v2 (Rust) — reads SQLite, YAML, JSON
- **Frontend**: Svelte + TypeScript
- **Design**: GitHub Dark theme, fully configurable via `theme.json`

## Toolbar

| Button | What it does |
|--------|-------------|
| **Search** | Filter sessions by text (matches title, summary, branch, folder) |
| **View** | Switch grouping: Source, Folder, Branch, Date |
| **Sort** | Order sessions by modified date, created date, turns, size, title, branch, folder, or source |
| **Filters** | Advanced filtering — folder, branch, turn count, checkpoints, status, date range |
| **Setups** | Save/load filter + view combinations as named presets |
| **Settings** | Configure session paths, DB path, Dobby mode, and theme |

**Keyboard shortcut**: `Ctrl+R` rescans all sessions.

## Right-Click Context Menu

Right-click any session for quick actions:
- **Resume** — opens a new terminal in the session's working directory
- **Preview** — opens the detail panel for that session
- **Copy ID** — copies the session ID to clipboard
- **Pin / Unpin** — pin a session to the top of its group
- **Delete** — removes the session from disk

## Filters

The filter panel lets you narrow down sessions before grouping:

- **Folder** — comma-separated folder names; matches any session whose working directory contains one of the values (e.g. `myrepo,project-x`)
- **Branch** — comma-separated branch names; matches any session on a matching branch (e.g. `main,dev,feature`)
- **Min/Max turns** — filter by conversation length
- **Checkpoints** — show only sessions with/without checkpoints
- **Status** — Active (on disk) or Deleted
- **Date range** — filter by creation date
- **Hide deleted / Hide empty** — clean up the list

## Setups

Setups save your current view, sort, and filter configuration as a reusable preset. Use them to quickly switch between different workflows.

### How to use

1. Configure your view, sort, and filters the way you want
2. Open the **Setups** dropdown and click **Save current as setup**
3. Give it a name — it appears in the dropdown for one-click switching

### Use cases

| Setup name | View | Filters / Group Filter | When to use |
|-----------|------|------------------------|-------------|
| `Copilot CLI Sessions` | Source | group filter: `Copilot CLI` | Show only Copilot CLI sessions (built-in) |
| `VS Code Chat Sessions` | Source | group filter: `VS Code Copilot` | Show only VS Code Chat sessions (built-in) |
| `my-project` | Branch | folder: `C:\code\my-project` | Focus on one repo, grouped by branch |
| `active-work` | Source | branch: `main,dev`; min turns: 3 | Find meaningful sessions on key branches |
| `recent-long` | Date | min turns: 10; date: last 7 days | Review substantial recent sessions |
| `dobby` | Folder | *(built-in, auto-configured)* | When Dobby is enabled in settings, this setup appears automatically |

**Remove All** — the Setups dropdown includes a button to delete all custom setups at once.

## Group Search

When viewing sessions grouped by folder, branch, or date, the group search bar (above the session list) filters which groups are visible.

- Separate multiple patterns with `,` (comma) — matches any pattern (OR logic)
- Prefix a pattern with `/` to use regex (e.g. `/feature-.*`)
- Matching is case-insensitive

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Configuring Session Paths

chasm has two configurable paths in **Settings** for Copilot CLI:

| Setting | Default | Description |
|---------|---------|-------------|
| **Copilot CLI Sessions Path** | `~/.copilot/session-state` | Directory containing session folders (workspace.yaml, events.jsonl) |
| **Session Store DB Path** | `~/.copilot/session-store.db` | SQLite database with sessions, turns, checkpoints, files |

If your sessions are stored elsewhere (e.g. older Copilot CLI versions used `~/.copilot/history-session-state/`), you can configure the paths individually.

Multiple paths can be specified **comma-separated** for both settings:

```
C:\Users\you\.copilot\session-state, C:\Users\you\.copilot-old\session-state
```

All valid paths are scanned and sessions are merged (deduplicated by session ID). This is useful when migrating between Copilot CLI versions or when session data lives in multiple locations.

## Documentation

- [docs/FEATURES.md](docs/FEATURES.md) — full feature list and capabilities
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — source adapter plugin system
- [docs/DECISIONS.md](docs/DECISIONS.md) — design decisions with measured reasoning
- [docs/DATA_SOURCES.md](docs/DATA_SOURCES.md) — data format reference
