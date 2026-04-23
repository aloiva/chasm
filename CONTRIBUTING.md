# Contributing Rules

These rules are **mandatory** for all contributors (including AI agents).

## 1. Abstraction First

- **Every data source must implement the `SessionSource` trait.** No source-specific logic in the Tauri command layer or frontend.
- **All types flowing to the frontend use unified types** (`SessionSummary`, `SessionDetail`, `ConversationTurn`). Never expose source-internal types to the UI.
- When adding a new AI tool adapter, only a single new file under `src-tauri/src/adapters/` should be needed. If you're touching `lib.rs` beyond registering the adapter, you're doing it wrong.

## 2. Error Handling

- **Never panic on missing/corrupt data.** Use `SourceError::Warning` and skip. The app must always launch.
- **Never crash if a source directory doesn't exist.** Return `SourceError::Fatal` to disable that source gracefully.
- **Log warnings, don't show error dialogs** for missing sessions, locked databases, or deleted files.

## 3. Separation of Concerns

- **Rust backend**: data access, filesystem, SQLite. No UI logic, no HTML, no CSS.
- **Svelte frontend**: presentation only. No direct filesystem access, no SQLite queries. All data comes through Tauri `invoke()`.
- **Components are single-responsibility.** `SessionCard` renders one card. `SessionList` renders many cards. `Toolbar` handles search/filter. Don't merge them.
- **Stores hold state, components render it.** Business logic (filtering, sorting, grouping) lives in stores, not components.

## 4. Theming & Styling

- **All colors, fonts, and sizes must use CSS custom properties** (e.g., `var(--bg-primary)`). No hardcoded hex values in components.
- **The theme must be swappable** by changing `theme.json` without modifying any component code.
- When adding new visual elements, add corresponding CSS variables to the theme system.

## 5. No Source Pollution

- **The app never writes to AI tool directories** except:
  - Copilot CLI: `workspace.yaml` (rename only)
  - Copilot CLI: folder deletion (delete only, with user confirmation)
- **App state lives in `~/.chasm/`** only. Pins, tags, cache, config — all in our own directory.
- **VS Code `state.vscdb` is read-only.** Do not write to it under any circumstances.

## 6. Performance

- **No polling.** Use filesystem watchers (`notify` crate) or manual scan. CPU at idle must be 0%.
- **Lazy-load conversation content.** Session list shows summaries only. Full turn data loads on click.
- **Cache session summaries.** The app must boot from cache in <50ms. Full scan is on-demand.

## 7. Future-Proofing

- **`SessionSummary.extra: HashMap<String, String>`** exists for source-specific metadata. Use it instead of adding source-specific fields to the unified types.
- **New adapters should not require frontend changes.** The UI reads `source`, `display_name`, and `icon` from the adapter — the frontend doesn't hardcode source names (except for badge colors, which use a fallback).
- **Document every design decision** in `docs/DECISIONS.md` with measured reasoning. "It felt right" is not a valid reason.

## 8. Code Quality

- `cargo check` must pass with **zero errors and zero warnings** before committing Rust code.
- `npm run build` must succeed before committing frontend code.
- New Rust adapters must include at least one integration test verifying `scan()` returns results (or an appropriate error) on the developer's machine.
- Keep dependencies minimal. Every new crate or npm package must justify its existence.

## 9. Documentation

- Update `docs/FEATURES.md` when adding or changing user-facing functionality.
- Update `docs/DATA_SOURCES.md` when adding a new AI tool adapter.
- Update `docs/ARCHITECTURE.md` when changing the adapter trait or data flow.
- Update `docs/DECISIONS.md` when making a non-trivial design choice.
- The `README.md` must always list all supported tools.
