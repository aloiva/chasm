# chasm — features

**c**entralised **h**ub for **a**gent **s**ession **m**anager

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
- **Pin / Unpin** — pin the session to the top of the list
- **Copy ID** — copy the session ID to clipboard
- **Open Folder** — open the session's directory in file explorer
- **Resume** — continue the session in its native tool
- **Rename** — change the session display name (Copilot CLI only)
- **Delete** — remove the session with confirmation dialog (Copilot CLI only)

### resume behaviour

- **Copilot CLI**: spawns a new PowerShell terminal window with `copilot --resume=<id>`. Uses `Start-Process` to ensure the terminal is fully interactive with user input support.
- **VS Code Copilot Chat**: opens VS Code to the associated workspace folder via `Start-Process code <path>`.

### pinned sessions

- Pin any session from the context menu or by clicking the 📌 icon
- Pinned sessions always appear at the top of the list, regardless of sort order
- Pins persist across app restarts (stored in localStorage)

---

## browsing & filtering

### search

- Real-time text search filters sessions by name, ID, or summary
- Debounced input to avoid excessive re-renders

### sort options

- Sort by **date** (newest/oldest first) or **name** (A-Z / Z-A)
- Sort is applied after pin priority (pinned sessions always on top)

### source filter

- Toggle which sources are visible (e.g. show only Copilot CLI sessions)
- Filter pills in the toolbar for quick toggling

### grouping

- Sessions are grouped by **source** (Copilot CLI, VS Code Copilot)
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

- **Enable Dobby** — toggle Dobby adapter (experimental, for `c:\dobby\agents` paths)
- **Copilot CLI Path** — override the default `~/.copilot` directory with a custom path; validates that the path exists before applying; persists across restarts
- **Reindex** (experimental) — triggers `/chronicle reindex` to rebuild the Copilot CLI session index, useful when deleted sessions still appear

---

## custom setups

- Define custom session sources with a name, path, and command
- Custom setups appear as additional source options
- Managed through the settings panel

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

- **Terminal lifecycle**: terminals spawned by chasm are killed when the app exits (Windows Job Object behaviour). This is a platform limitation — `CREATE_BREAKAWAY_FROM_JOB` requires elevated privileges.
- **VS Code resume**: cannot open a specific Copilot Chat session — VS Code Copilot Chat doesn't expose a CLI/URI for individual sessions. Resume opens the workspace folder only.
- **VS Code sessions are read-only**: `state.vscdb` is owned by VS Code. Rename and delete are not supported for VS Code Copilot sessions.
- **macOS icons**: `.icns` generation requires macOS-specific tooling. Current build targets Windows only.
