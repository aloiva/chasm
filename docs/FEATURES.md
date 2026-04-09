# chasm — features

**c**entralised **h**ub for **a**gent **s**ession **m**anagement

A desktop app that aggregates AI coding sessions from multiple tools into a single, searchable interface.

---

## supported sources

| Source | Data location | Resume | Rename | Delete |
|---|---|---|---|---|
| **Copilot CLI** | `~/.copilot/session-store.db` + `session-state/` folders | ✅ Opens pwsh with `copilot --resume` | ✅ Edits `workspace.yaml` | ✅ Removes session folder |
| **VS Code Copilot Chat** | `%APPDATA%/Code/User/workspaceStorage/*/state.vscdb` | ✅ Opens VS Code to workspace folder | ❌ Read-only storage | ❌ Read-only storage |

---

## session management

### context menu (right-click)

Right-click any session card to access:

- **Preview** — open the session detail panel
- **Pin / Unpin** — pin the session to the top of the list (stored globally in localStorage, persists across restarts; added in v0.1.0)
- **Copy ID** — copy the session ID to clipboard; useful for resuming in a separate terminal via `copilot --resume=<id>`, unlinked to chasm (so the session survives if the app closes)
- **Open Folder** — open the session's directory in file explorer
- **Resume** — opens a terminal with the session resumed, ready to continue the conversation
- **Rename** — change the session display name (Copilot CLI only)
- **Delete** — permanently removes the session folder from disk (with confirmation dialog; Copilot CLI only)

### resume behaviour

- **Copilot CLI**: opens a PowerShell terminal with the session resumed via `copilot --resume=<id>`. The terminal is fully interactive.
- **VS Code Copilot Chat**: opens VS Code to the associated workspace folder via `Start-Process code <path>`.

### pinned sessions

- Pin any session from the context menu or by clicking the 📌 icon
- Pinned sessions always appear at the top of the list, regardless of sort order
- Pins are stored globally in localStorage and persist across app restarts
- Added in v0.1.0

---

## browsing & filtering

### session search

- Real-time text search across all session metadata: title, session ID, folder path, branch, source, and summary. Any detail about a session can be matched.
- Supports multiple comma-separated search terms (e.g., `repo1,title1,title2`)
- **Search operators** (default is `contains` when no operator is specified):
  - `startswith=value` — matches sessions where any field starts with value
  - `endswith=value` — matches sessions where any field ends with value
  - `contains=value` — explicit contains (same as plain text)
  - `not=value` or `!value` — excludes sessions containing value
  - `/regex/flags` — regex matching (e.g. `/feat-\d+/i`). To test: type `/pattern/` in any search input. Invalid regex falls back to plain text.
  - `plain text` — default contains (substring match)
- Debounced input to avoid excessive re-renders

### sort options

- Sort by **date** (newest/oldest first) or **name** (A-Z / Z-A)
- Sort is applied after pin priority (pinned sessions always on top)

### source filter

- Source is a **view mode** — select "Source" in the View dropdown to see sessions grouped by their origin (Copilot CLI, VS Code Copilot, etc.)
- Use the **group filter** to narrow down to a specific source (e.g. type `Copilot CLI` to show only that group)
- Built-in setups "Copilot CLI Sessions" and "VS Code Chat Sessions" apply this automatically

### group search

- Filters which **groups** are visible based on the group label text in the current view — only the group name is searched, not sessions inside it
  - In **Folder view** → searches folder paths
  - In **Branch view** → searches branch names
  - In **Source view** → searches source names (e.g. "Copilot CLI")
  - In **Date view** → searches date bucket labels
- **Search operators** (same as session search): `startswith=`, `endswith=`, `contains=`, `not=`/`!`, `/regex/`
- Default (no operator) is `contains` — case-insensitive substring match
- **Multiple patterns**: use commas to search for multiple terms (e.g., `startswith=feat,!test` shows groups starting with "feat" but not containing "test")
- **Regex support**: wrap a pattern in `/regex/flags` for regex matching (e.g., `/^C:\\projects/i`)
- Invalid regex gracefully falls back to plain text search
- Combines with view mode — e.g., filter folder groups while in folder view

### grouping

- Sessions can be grouped by **source** (default), **folder**, **branch**, or **date**
- Source view is the default — shows sessions grouped by their adapter origin
- Each group has a collapsible header with session count
- Group headers have their own context menu with bulk actions

---

## detail panel

Clicking a session shows the detail panel on the right side with:

- **Session title** and source badge
- **Session ID** (with copy button)
- **Turn count** — actual number of conversation turns
- **Created / updated timestamps**
- **Conversation timeline** — full turn-by-turn view (lazy-loaded on click)

The panel auto-refreshes when:
- A different session is selected
- Ctrl+R is pressed to refresh all data
- The file watcher detects changes

---

## keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+R` | Refresh all sessions and detail panel |

---

## file watcher

- Watches source directories for filesystem changes using the `notify` crate
- **Copilot CLI**: watches both `~/.copilot/session-state/` (folder changes) and `~/.copilot/session-store.db` (turn count updates)
- **VS Code Copilot**: watches `%APPDATA%/Code/User/workspaceStorage/` for new/changed sessions
- 2-second debounce to batch rapid changes
- Auto-refreshes the session list when changes are detected

---

## settings

Access settings via the ⚙️ gear icon in the toolbar:

- **Enable Dobby** — toggle Dobby adapter (experimental, for Dobby agent sessions)
- **Copilot CLI Sessions Path** — override the default `~/.copilot/session-state` directory; supports multiple comma-separated paths (all are scanned and sessions are merged); validates that each path exists before applying; persists across restarts
- **Session Store DB Path** — override the default `~/.copilot/session-store.db` file path; supports multiple comma-separated paths (all DBs are queried and sessions are deduplicated by ID); validates that each path exists before applying; persists across restarts
- **Reindex** (experimental) — triggers `/chronicle reindex` to rebuild the Copilot CLI session index, useful when deleted sessions still appear

---

## custom setups

Setups are the most powerful feature in chasm. They save the full configuration — view mode, sort, filters, session search, and group search — as a reusable preset for one-click switching.

### when to use setups

| Use case | View | Filter | Session Search | Group Search |
|----------|------|--------|----------------|--------------|
| Focus on one repo across branches | Branch | folder: `C:\myrepo` | — | — |
| One repo with duplicates, per branch | Branch | folder: `C:\repopath1,C:\duplicaterepopath2` | — | — |
| Above, only release branches | Branch | folder: `C:\repopath1,C:\duplicaterepopath2` | — | `release/` |
| Only Copilot CLI sessions | Source | — | — | `Copilot CLI` |
| Only VS Code sessions | Source | — | — | `VS Code Copilot` |
| Dobby agent sessions | Folder | — | — | `startswith=C:\dobby\agents,endswith=_agent-cli` |
| Active meaningful work | Source | branch: `main,dev`, min turns: 3 | — | — |
| Recent long sessions | Date | min turns: 10, date: last 7 days | — | — |

### built-in setups

| Setup | View | Sort | Filter | Session Search | Group Search |
|-------|------|------|--------|----------------|--------------|
| Copilot CLI Sessions | Source | — | — | — | `Copilot CLI` |
| VS Code Chat Sessions | Source | — | — | — | `VS Code Copilot` |
| Dobby | Folder | Modified | — | — | `startswith=C:\dobby\agents,endswith=_agent-cli` |

### how to create

- Save the current view configuration (view mode, session search, group search, sort, filters) as a named setup
- Apply any setup with one click to restore the full view state
- Delete user-created setups from the setup menu
- Active setup is visually highlighted

---

## theming

- All colors use CSS custom properties defined in `:root`
- GitHub Dark theme by default
- Theme is swappable by modifying CSS variables — no component changes needed
- Cross-platform scrollbar styling (thin, dark scrollbars matching the theme)

---

## layout

- **Resizable sidebar** — drag the divider between session list and detail panel
- **Collapsible groups** — click group headers to expand/collapse
- **Responsive** — detail panel fills available space

---

## build & release

### development

```bash
npm install
npm run tauri dev
```

### production build

```bash
npm run tauri build
```

Produces three artifacts in `src-tauri/target/release/`:
- `chasm.exe` — standalone executable
- `bundle/nsis/chasm_<version>_x64-setup.exe` — NSIS installer with Start Menu entry
- `bundle/msi/chasm_<version>_x64_en-US.msi` — MSI installer for enterprise deployment

### github releases

Push a version tag to trigger automated builds:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The GitHub Actions workflow builds on `windows-latest` and attaches installers to a GitHub Release.

---

## tech stack

| Layer | Technology |
|---|---|
| Backend | Tauri v2 (Rust) |
| Frontend | SvelteKit + TypeScript |
| Data | SQLite (rusqlite), YAML (serde_yaml), JSON |
| File watching | notify crate |
| Icons | Custom logo (green lightning V) |
| Build | Vite (frontend), Cargo (Rust), NSIS/WiX (installers) |

---

## known limitations

- **Terminal lifecycle**: terminals spawned by chasm are killed when the app exits (Windows Job Object behaviour). Use **Copy ID** to resume in a separate terminal unlinked to the app.
- **VS Code resume**: cannot open a specific Copilot Chat session — VS Code Copilot Chat doesn't expose a CLI/URI for individual sessions. Resume opens the workspace folder only.
- **VS Code sessions are read-only**: `state.vscdb` is owned by VS Code. Rename and delete are not supported for VS Code Copilot sessions.
- **Deleted sessions persist in DB**: sessions deleted from disk still appear as "Deleted" until the session store database is reset. Use **Settings → Reindex Sessions** (experimental) to rebuild the index. A restart or `Ctrl+R` may be needed after reindexing.
- **macOS icons**: `.icns` generation requires macOS-specific tooling. Current build targets Windows only.
