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
| **Search** | Filter sessions by title, ID, folder, branch, source, or summary. Operators: plain text (contains by default), `startswith=`, `endswith=`, `not=`/`!`. Use `,` for OR, `+` for AND, `()` for grouping |
| **View** | Switch grouping: Source, Folder, Branch, Date |
| **Sort** | Order sessions by modified date, created date, turns, size, title, branch, folder, or source |
| **Filters** | Advanced filtering — title, folder, branch, turn count, checkpoints, status, date range. Filters support the same operators as search |
| **Setups** | Save/load the full config (view, sort, filters, session search, group search) as named presets. See [Setups](#setups) below |
| **Settings** | Configure session paths, DB path, Dobby mode, and theme |

**Keyboard shortcut**: `Ctrl+R` rescans all sessions.

Each feature is described in detail in its own section below: [Search](#search), [Filters](#filters), [Setups](#setups).

## Right-Click Context Menu

Right-click any session for quick actions:
- **Resume** — opens a terminal with the session resumed, ready to continue the conversation
- **Preview** — opens the detail panel for that session
- **Copy ID** — copies the session ID to clipboard; useful for resuming in a separate terminal via `copilot --resume=<id>`, unlinked to chasm (so the session survives if the app closes)
- **Pin / Unpin** — pin a session to the top of its group; pins are stored globally in localStorage and persist across restarts (added in v0.1.0)
- **Delete** — permanently removes the session folder from disk (with confirmation dialog)

## Search

chasm has two search inputs — **session search** and **group search** — each targeting different data.

### Session Search (toolbar)

Searches across all session metadata: title, session ID, folder path, branch, source, and summary. Any detail about a session can be matched.

- Multiple terms separated by `,` (comma) — matches any term (**OR** logic)
- Terms joined by `+` (plus) — all must match (**AND** logic within the group)
- Parentheses `()` override precedence: `(A,B)+C` = (A OR B) AND C
- Without parens, AND binds tighter: `A,B+C` = A OR (B AND C)
- **Search operators** (default is `contains` when no operator is specified):
  - `startswith=value` — matches sessions where any field starts with value
  - `endswith=value` — matches sessions where any field ends with value
  - `contains=value` — explicit contains (same as plain text)
  - `not=value` or `!value` — excludes sessions containing value
  - `plain text` — default contains (substring match)

### Group Search (above session list)

Filters which **groups** are visible based on the group label text in the current view. Only the group name is searched — not the sessions inside it.

- In **Folder view** → searches folder paths
- In **Branch view** → searches branch names
- In **Source view** → searches source names (e.g. "Copilot CLI")
- In **Date view** → searches date bucket labels

Same operators and combinators as session search: `startswith=`, `endswith=`, `contains=`, `not=`/`!`, `,` for OR, `+` for AND, `()` for grouping.

## Setups

Setups are the most powerful feature in chasm. They save the full configuration — view mode, sort, filters, session search, and group search — as a reusable preset for one-click switching between workflows.

### When to use setups

| Use case | Setup config | Why |
|----------|-------------|-----|
| **Focus on one repo across branches** | view: Branch, filter folder: `C:\myrepo` | See all branches for a single repo |
| **One repo with duplicates** | view: Branch, filter folder: `C:\repopath1,C:\duplicaterepopath2` | Merge sessions from duplicate repo paths, grouped by branch |
| **Above, but only release branches** | view: Branch, filter folder: `C:\repopath1,C:\duplicaterepopath2`, group search: `release/` | Same as above, filtered to release branches |
| **Only Copilot CLI sessions** | view: Source, group search: `Copilot CLI` | Hide VS Code and other sources (built-in setup) |
| **Only VS Code sessions** | view: Source, group search: `VS Code Copilot` | Hide Copilot CLI sessions (built-in setup) |
| **Dobby agent sessions** | view: Folder, group search: `startswith=C:\dobby\agents+endswith=_agent-cli` | Dobby agent folders only (built-in setup) |
| **Active meaningful work** | view: Source, filter branch: `main,dev`, filter min turns: 3 | Find real work on key branches |
| **Recent long sessions** | view: Date, filter min turns: 10, filter date: last 7 days | Review substantial recent sessions |

### Setup details

| Setup name | View | Sort | Filter | Session Search | Group Search |
|-----------|------|------|--------|----------------|--------------|
| `Copilot CLI Sessions` (built-in) | Source | — | — | — | `Copilot CLI` |
| `VS Code Chat Sessions` (built-in) | Source | — | — | — | `VS Code Copilot` |
| `Dobby` (built-in) | Folder | Modified | — | — | `startswith=C:\dobby\agents+endswith=_agent-cli` |

### How to use

1. Configure your view, sort, filters, session search, and group search the way you want
2. Open the **Setups** dropdown and click **Save current as setup**
3. Give it a name — it appears in the dropdown for one-click switching

**Remove All** — the Setups dropdown includes a button to delete all custom setups at once.

## Filters

The filter panel lets you narrow down sessions before grouping:

- **Folder** — supports operators (`startswith=`, `endswith=`, `not=`/`!`, or plain text); `,` for OR, `+` for AND
- **Branch** — same operators and combinators as folder
- **Min/Max turns** — filter by conversation length
- **Checkpoints** — show only sessions with/without checkpoints
- **Status** — Active (on disk) or Deleted
- **Date range** — filter by creation date
- **Hide deleted / Hide empty** — clean up the list

**About deleted sessions**: sessions that were deleted from disk still appear as "Deleted" until the session store database is reset. You can trigger a reset via **Settings → Reindex Sessions** (experimental). This rebuilds the Copilot CLI session index, but the cleanup may not always reflect immediately — a restart or `Ctrl+R` rescan may be needed.

Filters specifically narrow sessions by the group field. Session search does the same but matches across all session metadata. Group search narrows which groups are visible, not which sessions.

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

## Future Plans

- **Checkpoint browsing** — browse and navigate session checkpoints directly within chasm
- **AgentViz integration** — integrate [AgentViz](https://github.com/jayparikh/agentviz) for visual agent session exploration

## Documentation

- [docs/FEATURES.md](docs/FEATURES.md) — full feature list and capabilities
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — source adapter plugin system
- [docs/DECISIONS.md](docs/DECISIONS.md) — design decisions with measured reasoning
- [docs/DATA_SOURCES.md](docs/DATA_SOURCES.md) — data format reference
