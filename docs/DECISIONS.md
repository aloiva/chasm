# Design Decisions

Every major decision was made with measured data and explicit reasoning.

## 1. Why Tauri over Electron?

**Decision**: Tauri v2 (Rust + system WebView2)

| Metric | Tauri | Electron |
|--------|-------|----------|
| Bundle size | ~2 MB (uses system WebView2) | ~150 MB (bundles Chromium) |
| Startup time | < 500ms | 2-5 seconds |
| Memory at rest | ~30 MB | ~100-300 MB |
| CPU at idle | 0% | > 0% (Node.js event loop) |

Electron was rejected because the goal is a utility app that boots instantly and sits in the background with zero CPU. Tauri achieves this natively.

## 2. Why passive data reading over an MCP server?

**Decision**: Read existing files directly. No MCP server in v1.

**Measured**: Full scan of 44 Copilot CLI sessions + 44 workspace.yaml files takes **393ms** (cold, measured on this machine). The session-store.db is only **896 KB**.

An MCP server approach would require:
- Running a sidecar process permanently (CPU/memory cost)
- Modifying every Copilot agent definition to include the MCP server
- Configuring Copilot CLI to always connect to it
- Handling the MCP server crashing or not being available

The passive approach has **zero runtime overhead** — it only reads when the user opens the app or clicks Scan. All the data already exists on disk in well-structured formats (SQLite, YAML, JSONL).

MCP is deferred to v2 for real-time enrichment features (auto-tagging, activity tracking).

## 3. Why cache-based startup?

**Decision**: Scan results saved to `~/.copilot-session-manager/cache/{source}.json`. App boots from cache.

Even though a full scan is ~400ms, we want the app to feel instant:
- First launch: scan runs, results cached (~400ms)
- Subsequent launches: read tiny JSON cache (<50ms)
- User clicks "Scan" to refresh when they want fresh data
- Sources can be scanned individually or all at once
- If a cached session no longer exists on disk → "(deleted)" warning badge, not a crash

## 4. Why no Claude Code adapter in v1?

**Decision**: Stub only, not implemented.

Claude Code is not installed on the development machine. The `SessionSource` trait is designed so adding it later requires only implementing one trait (a single Rust file). The data format is documented in DATA_SOURCES.md.

## 5. Why GitHub Dark theme?

**Decision**: Option B from 4 design mockups.

Four mockups were created and presented:
- A) Minimal/Clean (Linear, Notion style)
- **B) GitHub Dark** — chosen: compact, information-dense, monospace accents, badge labels
- C) macOS/Raycast (gradient, glassmorphism)
- D) Windows 11 Fluent (Mica, Segoe UI)

GitHub Dark was chosen because it's "easily relatable since it's GitHub themed" (direct quote). The entire theme is configurable via CSS custom properties loaded from `theme.json`, so switching themes requires no code changes.

## 6. Why VS Code sessions are read-only (no delete/rename)?

**Decision**: VS Code Copilot Chat data lives in `state.vscdb`, an internal SQLite database used by VS Code for many purposes (not just chat). Modifying it risks corrupting VS Code's workspace state.

Instead:
- **Rename**: stored in app-local metadata (`app-state.json`), not in `state.vscdb`
- **Delete**: not supported — too risky

## 7. Why spawn a real terminal for resume?

**Decision**: `pwsh -NoExit -Command "copilot resume {id}"` spawns a new PowerShell 7 window.

Alternative was clipboard-copy approach (copy command, user opens terminal, pastes, presses enter = 3 steps). Direct terminal spawn is **1 click**.

For VS Code sessions, resume opens VS Code in the workspace: `code --folder-uri {folder}`.

## 8. Why per-source scan (not all-at-once only)?

**Decision**: Each source can be scanned independently.

VS Code workspace scan reads **89 separate SQLite databases** (~1 second). Copilot CLI scan reads **1 database** (~400ms). Users may only want one source. The first-launch source picker lets users choose which tools to scan.

## 9. Why filesystem watcher is optional?

**Decision**: Watcher enabled by default, configurable off in settings.

The watcher uses OS-native APIs (`ReadDirectoryChangesW` on Windows via the `notify` crate). This has near-zero CPU/memory cost — it's a kernel callback, not polling. However, watching 89 VS Code workspace directories could have overhead on some systems, so it's configurable.

## 10. Why Svelte over React/Vue?

**Decision**: Svelte for minimal bundle size and fast startup.

Svelte compiles to vanilla JS at build time — no runtime library shipped. This keeps the WebView2 payload minimal, contributing to the <500ms startup target.
