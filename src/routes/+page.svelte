<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import SessionList from '$lib/components/SessionList.svelte';
  import SessionDetail from '$lib/components/SessionDetail.svelte';
  import GroupDetail from '$lib/components/GroupDetail.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import GroupContextMenu from '$lib/components/GroupContextMenu.svelte';
  import { sessions, loading, selectedSessionId, selectedGroupKey, selectSession, selectGroup, refreshCounter, togglePin, searchQuery, messageMatchIds } from '$lib/stores/sessions';
  import { extractSearchKeywords, parseSearchExpr, resolveExprWithSets } from '$lib/utils/search';
  import { settings } from '$lib/stores/settings';
  import type { SessionSummary } from '$lib/types/session';

  let contextMenu = $state<{ session: SessionSummary; x: number; y: number } | null>(null);
  let groupContextMenu = $state<{ key: string; x: number; y: number } | null>(null);
  let renameTarget = $state<SessionSummary | null>(null);
  let renameValue = $state('');
  let deleteConfirm = $state<SessionSummary | null>(null);
  let deleteWorkspaceCount = $state(0);
  let deleteMode = $state<'session' | 'workspace'>('session');
  let unlisten: (() => void) | null = null;
  let msgSearchTimer: ReturnType<typeof setTimeout> | null = null;
  let unsubSearch: (() => void) | null = null;

  // Resizable sidebar
  let sidebarWidth = $state(400);
  let isResizing = $state(false);

  function onResizeStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    function onMouseMove(ev: MouseEvent) {
      const newWidth = startWidth + (ev.clientX - startX);
      sidebarWidth = Math.max(200, Math.min(newWidth, window.innerWidth - 300));
    }

    function onMouseUp() {
      isResizing = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  async function scanSessions() {
    const isViewing = !!get(selectedSessionId);
    // Don't show loading spinner if user is reading a session
    if (!isViewing) loading.set(true);
    try {
      // Phase 1: Show cached data instantly (no filesystem walk)
      const cached = await invoke('list_sessions_cached');
      sessions.set(cached as any[]);
      if (!isViewing) {
        loading.set(false);
        refreshCounter.update(n => n + 1);
      }

      // Phase 2: Full scan in background, silently update the list
      const fresh = await invoke('list_sessions');
      sessions.set(fresh as any[]);
      if (!isViewing) refreshCounter.update(n => n + 1);
    } catch (e) {
      console.error('Scan failed:', e);
      if (!isViewing) loading.set(false);
    }
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'r') {
      e.preventDefault();
      scanSessions();
    }
  }

  function handleRescan() {
    scanSessions();
  }

  import { get } from 'svelte/store';

  onMount(async () => {
    // Apply saved custom paths before first scan (comma-separated, sent as-is to backend)
    const savedCliPath = get(settings).copilotCliPath;
    if (savedCliPath) {
      try {
        await invoke('set_copilot_cli_path', { path: savedCliPath });
      } catch {
        // Paths may no longer exist — backend will reject
      }
    }
    const savedDbPath = get(settings).copilotDbPath;
    if (savedDbPath) {
      try {
        await invoke('set_copilot_db_path', { path: savedDbPath });
      } catch {
        // Path may no longer exist
      }
    }

    await scanSessions();

    // Listen for file watcher events to auto-refresh
    unlisten = await listen('sessions-changed', () => {
      scanSessions();
    });

    window.addEventListener('keydown', handleGlobalKeydown);
    window.addEventListener('chasm-rescan', handleRescan);

    // Debounced message search: query the backend for turn content matches.
    // Each keyword is searched individually, then combined with AND/OR logic.
    unsubSearch = searchQuery.subscribe((q) => {
      if (msgSearchTimer) clearTimeout(msgSearchTimer);
      const trimmed = q.trim();
      if (!trimmed) {
        messageMatchIds.set(new Set());
        return;
      }
      msgSearchTimer = setTimeout(async () => {
        try {
          const keywords = extractSearchKeywords(trimmed);
          if (keywords.length === 0) {
            messageMatchIds.set(new Set());
            return;
          }
          // Query each keyword in parallel
          const results = await Promise.all(
            keywords.map(async (kw) => {
              const ids: string[] = await invoke('search_messages', { query: kw });
              return [kw, new Set(ids)] as [string, Set<string>];
            })
          );
          const keywordResults = new Map(results);

          // Combine using the parsed expression tree (AND = intersect, OR = union)
          const expr = parseSearchExpr(trimmed);
          if (expr) {
            messageMatchIds.set(resolveExprWithSets(expr, keywordResults));
          } else {
            // Fallback: union all
            const all = new Set<string>();
            for (const [, ids] of keywordResults) {
              for (const id of ids) all.add(id);
            }
            messageMatchIds.set(all);
          }
        } catch {
          messageMatchIds.set(new Set());
        }
      }, 300);
    });
  });

  onDestroy(() => {
    unlisten?.();
    unsubSearch?.();
    if (msgSearchTimer) clearTimeout(msgSearchTimer);
    window.removeEventListener('keydown', handleGlobalKeydown);
    window.removeEventListener('chasm-rescan', handleRescan);
  });

  const showDetail = $derived($selectedSessionId !== null || $selectedGroupKey !== null);

  function openContextMenu(e: MouseEvent, session: SessionSummary) {
    contextMenu = { session, x: e.clientX, y: e.clientY };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function openGroupContextMenu(e: MouseEvent, key: string) {
    groupContextMenu = { key, x: e.clientX, y: e.clientY };
  }

  function closeGroupContextMenu() {
    groupContextMenu = null;
  }

  function handlePreview(session: SessionSummary) {
    selectSession(session.id + ':' + session.source);
  }

  function handlePin(session: SessionSummary) {
    togglePin(session.id + ':' + session.source);
  }

  async function handleCopyId(session: SessionSummary) {
    try {
      await navigator.clipboard.writeText(session.id);
    } catch (e: any) {
      console.error('Copy ID failed:', e);
    }
  }

  async function handleResume(session: SessionSummary) {
    try {
      await invoke('resume_session', { source: session.source, id: session.id });
    } catch (e: any) {
      console.error('Resume failed:', e);
    }
  }

  async function handleOpenFiles(session: SessionSummary) {
    if (!session.cwd) return;
    try {
      await invoke('open_folder', { path: session.cwd });
    } catch (e: any) {
      console.error('Open folder failed:', e);
    }
  }

  async function handleOpenAgentviz(session: SessionSummary) {
    try {
      await invoke('open_agentviz', {
        agentvizPath: $settings.agentvizPath,
        source: session.source,
        id: session.id,
        maxSessions: $settings.agentvizMaxSessions,
      });
    } catch (e: any) {
      console.error('agentviz failed:', e);
    }
  }

  function handleRenameStart(session: SessionSummary) {
    renameTarget = session;
    renameValue = session.title ?? '';
  }

  async function handleRenameSubmit() {
    if (!renameTarget || !renameValue.trim()) return;
    try {
      await invoke('rename_session', {
        source: renameTarget.source,
        id: renameTarget.id,
        name: renameValue.trim(),
      });
      // Update local state
      sessions.update(all =>
        all.map(s =>
          s.id === renameTarget!.id && s.source === renameTarget!.source
            ? { ...s, title: renameValue.trim() }
            : s
        )
      );
    } catch (e: any) {
      console.error('Rename failed:', e);
    } finally {
      renameTarget = null;
      renameValue = '';
    }
  }

  function handleRenameCancel() {
    renameTarget = null;
    renameValue = '';
  }

  async function handleDeleteStart(session: SessionSummary) {
    deleteConfirm = session;
    deleteMode = 'session';
    deleteWorkspaceCount = 0;
    // For VS Code sessions with a workspace, check how many sessions are in the workspace
    if (session.source === 'vscode-copilot' && session.extra?.workspace_hash) {
      try {
        const count = await invoke<number>('count_workspace_sessions', {
          workspaceHash: session.extra.workspace_hash,
        });
        deleteWorkspaceCount = count;
      } catch {
        deleteWorkspaceCount = 0;
      }
    }
  }

  async function handleDeleteConfirm() {
    if (!deleteConfirm) return;
    try {
      if (deleteMode === 'workspace' && deleteConfirm.extra?.workspace_hash) {
        // Workspace-level delete: removes entire workspace folder
        await invoke('delete_workspace', {
          workspaceHash: deleteConfirm.extra.workspace_hash,
        });
        // Remove all sessions from this workspace from local state
        const wsHash = deleteConfirm.extra.workspace_hash;
        sessions.update(all =>
          all.filter(s => !(s.source === 'vscode-copilot' && s.extra?.workspace_hash === wsHash))
        );
        if ($selectedSessionId) {
          const sel = $sessions.find(s => s.id + ':' + s.source === $selectedSessionId);
          if (!sel) selectedSessionId.set(null);
        }
      } else {
        // Session-level delete
        const errors: string[] = await invoke('delete_sessions', {
          source: deleteConfirm.source,
          ids: [deleteConfirm.id],
        });
        if (errors.length > 0) {
          console.error('Delete errors:', errors);
        } else {
          const deleted = deleteConfirm;
          sessions.update(all =>
            all.filter(s => !(s.id === deleted.id && s.source === deleted.source))
          );
          if ($selectedSessionId === deleted.id + ':' + deleted.source) {
            selectedSessionId.set(null);
          }
        }
      }
    } catch (e: any) {
      console.error('Delete failed:', e);
    } finally {
      deleteConfirm = null;
      deleteWorkspaceCount = 0;
      deleteMode = 'session';
    }
  }

  function handleDeleteCancel() {
    deleteConfirm = null;
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleRenameSubmit();
    if (e.key === 'Escape') handleRenameCancel();
  }
</script>

<div class="app">
  <Toolbar />
  <div class="content" class:resizing={isResizing}>
    <div class="sidebar" class:collapsed={showDetail} style={showDetail ? `width:${sidebarWidth}px;min-width:${sidebarWidth}px` : ''}>
      <SessionList oncontextmenu={openContextMenu} ongroupcontextmenu={openGroupContextMenu} />
    </div>
    {#if showDetail}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle" onmousedown={onResizeStart}></div>
      <div class="detail-panel">
        {#if $selectedGroupKey}
          <GroupDetail />
        {:else}
          <SessionDetail />
        {/if}
      </div>
    {/if}
  </div>
</div>

{#if contextMenu}
  <ContextMenu
    session={contextMenu.session}
    x={contextMenu.x}
    y={contextMenu.y}
    onclose={closeContextMenu}
    onpreview={handlePreview}
    onpin={handlePin}
    oncopyid={handleCopyId}
    onopenfiles={handleOpenFiles}
    onresume={handleResume}
    onrename={handleRenameStart}
    ondelete={handleDeleteStart}
    onagentviz={handleOpenAgentviz}
  />
{/if}

{#if groupContextMenu}
  <GroupContextMenu
    groupKey={groupContextMenu.key}
    x={groupContextMenu.x}
    y={groupContextMenu.y}
    onclose={closeGroupContextMenu}
    onview={(key) => { selectGroup(key); closeGroupContextMenu(); }}
  />
{/if}

{#if renameTarget}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-backdrop" onclick={handleRenameCancel}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal" onclick={(e: MouseEvent) => e.stopPropagation()}>
      <div class="modal-title">Rename Session</div>
      <input
        class="modal-input"
        type="text"
        bind:value={renameValue}
        onkeydown={handleRenameKeydown}
        placeholder="Session name"
      />
      <div class="modal-actions">
        <button class="modal-btn cancel" onclick={handleRenameCancel}>Cancel</button>
        <button class="modal-btn confirm" onclick={handleRenameSubmit} disabled={!renameValue.trim()}>Rename</button>
      </div>
    </div>
  </div>
{/if}

{#if deleteConfirm}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-backdrop" onclick={handleDeleteCancel}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal" onclick={(e: MouseEvent) => e.stopPropagation()}>
      <div class="modal-title">Delete Session</div>
      {#if deleteConfirm.source === 'vscode-copilot' && deleteWorkspaceCount > 1}
        <div class="modal-text">
          This workspace contains <strong>{deleteWorkspaceCount}</strong> session(s). Choose what to delete:
        </div>
        <div class="delete-options">
          <label class="delete-option">
            <input type="radio" bind:group={deleteMode} value="session" />
            <span>Delete this session only</span>
          </label>
          <label class="delete-option">
            <input type="radio" bind:group={deleteMode} value="workspace" />
            <span>Delete entire workspace ({deleteWorkspaceCount} sessions)</span>
          </label>
        </div>
      {:else}
        <div class="modal-text">
          This will permanently delete the session from disk. This action cannot be undone.
        </div>
      {/if}
      <div class="modal-actions">
        <button class="modal-btn cancel" onclick={handleDeleteCancel}>Cancel</button>
        <button class="modal-btn danger" onclick={handleDeleteConfirm}>
          {deleteMode === 'workspace' ? 'Delete Workspace' : 'Delete'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  :root {
    /* GitHub Dark theme — configurable */
    --bg-primary: #0d1117;
    --bg-secondary: #161b22;
    --bg-tertiary: #21262d;
    --border: #30363d;
    --text-primary: #e6edf3;
    --text-secondary: #8b949e;
    --text-muted: #484f58;
    --accent: #58a6ff;
    --accent-green: #3fb950;
    --accent-yellow: #d29922;
    --accent-red: #f85149;
    --font-mono: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', monospace;
    --font-ui: system-ui, -apple-system, sans-serif;
    --font-size-base: 13px;
    --font-size-small: 11px;
    --font-size-title: 14px;
    --radius: 6px;
    --card-padding: 8px 12px;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-base);
    overflow: hidden;
  }

  :global(*) { box-sizing: border-box; }

  :global(*::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }
  :global(*::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(*::-webkit-scrollbar-thumb) {
    background: var(--border);
    border-radius: 4px;
  }
  :global(*::-webkit-scrollbar-thumb:hover) {
    background: var(--text-muted);
  }
  :global(*::-webkit-scrollbar-corner) {
    background: transparent;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar {
    width: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-right: 1px solid var(--border);
  }

  .sidebar.collapsed {
    flex-shrink: 0;
  }

  .resize-handle {
    width: 4px;
    cursor: col-resize;
    background: var(--border);
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .resize-handle:hover,
  .resizing .resize-handle {
    background: var(--accent);
  }

  .content.resizing {
    user-select: none;
    cursor: col-resize;
  }

  .detail-panel {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 2000;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 20px;
    min-width: 340px;
    max-width: 420px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
  }

  .modal-title {
    font-size: var(--font-size-title);
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 12px;
  }

  .modal-text {
    font-size: var(--font-size-base);
    color: var(--text-secondary);
    margin-bottom: 16px;
    line-height: 1.5;
  }

  .modal-input {
    width: 100%;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-base);
    margin-bottom: 16px;
    outline: none;
  }
  .modal-input:focus { border-color: var(--accent); }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .modal-btn {
    padding: 6px 14px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    font-weight: 600;
    cursor: pointer;
  }
  .modal-btn.cancel {
    background: var(--bg-secondary);
    color: var(--text-secondary);
  }
  .modal-btn.cancel:hover { color: var(--text-primary); }
  .modal-btn.confirm {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
  }
  .modal-btn.confirm:hover { opacity: 0.9; }
  .modal-btn.confirm:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .modal-btn.danger {
    background: var(--accent-red);
    color: #fff;
    border-color: var(--accent-red);
  }
  .modal-btn.danger:hover { opacity: 0.9; }

  .delete-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 8px 0;
  }
  .delete-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }
  .delete-option:hover { border-color: var(--accent); }
  .delete-option input[type="radio"] { accent-color: var(--accent); }
</style>
