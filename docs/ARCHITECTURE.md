# Architecture

## Overview

chasm uses a **source adapter pattern** to unify session data from multiple AI coding tools into one UI. Each tool is a Rust struct implementing the `SessionSource` trait.

## Source Adapter System

```
┌─────────────────────────────────────────────────────┐
│                    Frontend (Svelte)                  │
│  SessionList ← SessionCard, SourceBadge, SearchBar  │
│  SessionDetail ← ConversationTimeline, Toolbar      │
└──────────────────────┬──────────────────────────────┘
                       │ Tauri IPC (invoke)
┌──────────────────────▼──────────────────────────────┐
│               Tauri Command Layer (lib.rs)           │
│  list_sessions, get_detail, rename, delete, resume  │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│              SourceRegistry                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │CopilotCli    │  │VsCodeCopilot │  │ Future... │ │
│  │Source         │  │Source         │  │           │ │
│  └──────┬───────┘  └──────┬───────┘  └───────────┘ │
└─────────┼──────────────────┼────────────────────────┘
          │                  │
          ▼                  ▼
  ~/.copilot/           %APPDATA%/Code/
  session-store.db      workspaceStorage/
  session-state/        */state.vscdb
```

## SessionSource Trait

Every adapter implements:
- `name()` / `display_name()` / `icon()` — identity
- `is_available()` — check if tool is installed
- `scan()` → `Vec<SessionSummary>` — list all sessions (never panic)
- `load_detail(id)` → `SessionDetail` — full conversation (on demand)
- `rename(id, name)` / `delete(id)` — mutations (may return SourceError::Warning)
- `resume(id)` → `ResumeAction` — how to resume (spawn terminal or open app)
- `watch_paths()` — directories for filesystem watcher

## Error Handling

Two error levels:
- `SourceError::Warning` — skip this item, continue scanning (missing file, locked DB)
- `SourceError::Fatal` — disable this source entirely (directory doesn't exist)

The app never crashes on missing data. Deleted sessions show a "(deleted)" badge.

## Cache Model

- First scan populates `~/.chasm/cache/{source}.json`
- App boots from cache (instant <50ms)
- Manual "Scan" button re-reads source files and updates cache
- Filesystem watcher (optional, configurable) can trigger auto-refresh

## Component Tree (Svelte)

```
src/
  lib/
    components/
      SessionCard.svelte       — single session card
      SessionList.svelte        — virtual scroll list
      SessionDetail.svelte      — conversation timeline
      SourceBadge.svelte        — "CLI" / "VSC" badge
      SourceFilter.svelte       — multi-select source toggle pills
      ContextMenu.svelte        — right-click context menu (Resume/Delete/Preview/Rename)
      SearchBar.svelte          — debounced search input
      Toolbar.svelte            — sort/filter/scan
      SourcePicker.svelte       — first-launch source selector
      ConfirmDialog.svelte      — delete confirmation
    stores/
      sessions.ts               — Svelte store for session data (multi-select, folder grouping)
      theme.ts                  — CSS custom property injector
      sources.ts                — enabled source state
    types/
      session.ts                — TypeScript types matching Rust
    utils/
      format.ts                 — date formatting, truncation
```

## Theming

All visual properties come from CSS custom properties loaded from `~/.chasm/theme.json`. Changing the JSON changes the entire look without rebuilding.
