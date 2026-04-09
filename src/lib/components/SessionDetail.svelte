<script lang="ts">
  import type { SessionDetail } from '$lib/types/session';
  import SourceBadge from './SourceBadge.svelte';
  import { formatDate, formatDateRange } from '$lib/utils/format';
  import { invoke } from '@tauri-apps/api/core';
  import { selectedSessionId, refreshCounter } from '$lib/stores/sessions';

  let detail = $state<SessionDetail | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // React to selection changes and refreshes
  $effect(() => {
    const sel = $selectedSessionId;
    const _rc = $refreshCounter; // trigger re-fetch on refresh
    if (!sel) { detail = null; return; }
    const [id, source] = sel.split(':');
    if (id && source) loadDetail(source, id);
  });

  async function loadDetail(source: string, id: string) {
    loading = true;
    error = null;
    try {
      detail = await invoke('get_session_detail', { source, id }) as SessionDetail;
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load session';
      detail = null;
    } finally {
      loading = false;
    }
  }

  async function resumeSession() {
    if (!detail) return;
    try {
      const action = await invoke('resume_session', {
        source: detail.summary.source,
        id: detail.summary.id,
      });
      // Tauri will spawn the process
    } catch (e: any) {
      console.error('Resume failed:', e);
    }
  }

  async function openFiles() {
    if (!detail?.summary.cwd) return;
    try {
      await invoke('open_folder', { path: detail.summary.cwd });
    } catch (e: any) {
      console.error('Open folder failed:', e);
    }
  }

  async function openSessionFolder() {
    if (!detail?.summary.storage_path) return;
    try {
      await invoke('open_folder', { path: detail.summary.storage_path });
    } catch (e: any) {
      console.error('Open session folder failed:', e);
    }
  }

  function goBack() {
    selectedSessionId.set(null);
  }
</script>

{#if !$selectedSessionId}
  <div class="empty-detail">
    <div class="empty-icon">📋</div>
    <div class="empty-text">Select a session to view its conversation</div>
  </div>
{:else if loading}
  <div class="loading">Loading…</div>
{:else if error}
  <div class="error">{error}</div>
{:else if detail}
  <div class="detail">
    <div class="header">
      <div class="header-top-row">
        <button class="back-btn" onclick={goBack}>← Back</button>
        <button class="close-btn" onclick={goBack} title="Close panel">×</button>
      </div>
      <div class="header-body">
        <div class="header-info">
          <div class="header-title">
            <SourceBadge source={detail.summary.source} size="md" />
            <span class="title-text">{detail.summary.title ?? '(unnamed)'}</span>
          </div>
          <div class="header-meta">
            <button
              class="session-id"
              title="Click to copy: {detail.summary.id}"
              onclick={() => navigator.clipboard.writeText(detail.summary.id)}
            >🔑 {detail.summary.id.length > 24 ? detail.summary.id.slice(0, 24) + '…' : detail.summary.id}</button>
            {#if detail.summary.cwd}
              <span class="meta-item" title={detail.summary.cwd}>📁 {detail.summary.cwd}</span>
            {/if}
            {#if detail.summary.branch}
              <span>🌿 {detail.summary.branch}</span>
            {/if}
            <span>💬 {detail.turns.length} turns</span>
            {#if detail.checkpoints.length > 0}
              <span>📌 {detail.checkpoints.length} checkpoints</span>
            {/if}
            <span>📅 {formatDateRange(detail.summary.created_at, detail.summary.updated_at)}</span>
          </div>
          {#if detail.files_touched.length > 0}
            <div class="files-badge">{detail.files_touched.length} files touched</div>
          {/if}
        </div>
        <div class="header-actions">
          <button class="resume-btn" onclick={resumeSession}>▶ Resume</button>
          {#if detail.summary.cwd}
            <button class="open-files-btn" onclick={openFiles}>📂 Open Folder</button>
          {/if}
          {#if detail.summary.storage_path}
            <button class="open-files-btn" onclick={openSessionFolder}>🗂 Session Folder</button>
          {/if}
        </div>
      </div>
    </div>

    <div class="timeline">
      {#each detail.turns as turn}
        <!-- Check if a checkpoint should appear before this turn -->
        {#each detail.checkpoints.filter(c => c.after_turn === turn.turn_index) as ckpt}
          <div class="checkpoint-divider">
            ── checkpoint {ckpt.number}: {ckpt.title ?? 'untitled'} ──
          </div>
        {/each}

        {#if turn.user_message}
          <div class="turn user-turn">
            <div class="turn-label">
              You <span class="turn-meta">turn {turn.turn_index}{turn.timestamp ? ` · ${formatDate(turn.timestamp)}` : ''}</span>
            </div>
            <div class="turn-content">{turn.user_message}</div>
          </div>
        {/if}

        {#if turn.assistant_response}
          <div class="turn assistant-turn">
            <div class="turn-label">Copilot</div>
            <div class="turn-content assistant-content">{turn.assistant_response}</div>
          </div>
        {/if}
      {/each}

      {#if detail.turns.length === 0}
        <div class="empty-turns">No conversation data available</div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .empty-detail {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }
  .empty-icon { font-size: 48px; margin-bottom: 12px; opacity: 0.3; }
  .empty-text { font-size: var(--font-size-base); }
  .loading, .error {
    display: flex; align-items: center; justify-content: center;
    height: 100%; color: var(--text-muted);
  }
  .error { color: var(--accent-red); }

  .detail { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  .header {
    display: flex; flex-direction: column; gap: 8px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }
  .back-btn {
    padding: 4px 10px; border-radius: var(--radius);
    border: 1px solid var(--border); background: var(--bg-tertiary);
    color: var(--text-secondary); cursor: pointer; font-family: var(--font-mono);
    font-size: var(--font-size-small); flex-shrink: 0;
  }
  .back-btn:hover { border-color: var(--accent); color: var(--text-primary); }
  .header-top-row {
    display: flex; align-items: center; justify-content: space-between;
  }
  .close-btn {
    background: none; border: 1px solid var(--border); border-radius: var(--radius);
    color: var(--text-muted); font-size: 18px; cursor: pointer; padding: 2px 8px;
    line-height: 1; flex-shrink: 0;
  }
  .close-btn:hover { color: var(--text-primary); border-color: var(--accent); }
  .header-body {
    display: flex; align-items: flex-start; gap: 8px;
  }
  .header-info { flex: 1; min-width: 0; overflow: hidden; }
  .header-title {
    display: flex; align-items: center; gap: 8px;
    font-size: var(--font-size-title); font-weight: 600; color: var(--text-primary);
  }
  .title-text {
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .header-meta {
    display: flex; flex-wrap: wrap; gap: 8px 12px;
    font-size: var(--font-size-small); color: var(--text-muted);
    margin-top: 6px; font-family: var(--font-ui);
  }
  .meta-item {
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
    max-width: 220px;
  }
  .session-id {
    cursor: pointer;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    transition: color 0.15s;
    background: none;
    border: none;
    padding: 0;
    font-size: inherit;
    text-align: left;
  }
  .session-id:hover { color: var(--accent); }
  .files-badge {
    font-size: 10px; color: var(--accent-yellow);
    margin-top: 4px;
  }
  .header-actions {
    display: flex; flex-direction: column; gap: 4px;
    flex-shrink: 0;
  }
  .resume-btn {
    padding: 6px 14px; border-radius: var(--radius);
    border: 1px solid var(--accent-green); background: var(--accent-green);
    color: #fff; cursor: pointer; font-family: var(--font-mono);
    font-size: var(--font-size-small); font-weight: 600; flex-shrink: 0;
  }
  .resume-btn:hover { opacity: 0.9; }
  .open-files-btn {
    padding: 6px 14px; border-radius: var(--radius);
    border: 1px solid var(--border); background: var(--bg-tertiary);
    color: var(--text-secondary); cursor: pointer; font-family: var(--font-mono);
    font-size: var(--font-size-small); flex-shrink: 0;
  }
  .open-files-btn:hover { border-color: var(--accent); color: var(--text-primary); }

  .timeline {
    flex: 1; min-height: 0; overflow-y: auto; padding: 12px 16px;
  }

  .turn { margin-bottom: 16px; }
  .turn-label {
    font-size: var(--font-size-small); font-weight: 600;
    margin-bottom: 4px;
  }
  .user-turn .turn-label { color: var(--accent); }
  .assistant-turn .turn-label { color: var(--accent-green); }
  .turn-meta { font-weight: 400; color: var(--text-muted); margin-left: 8px; }

  .turn-content {
    font-size: var(--font-size-base); color: var(--text-primary);
    line-height: 1.5; white-space: pre-wrap; word-break: break-word;
    padding: 8px 12px; border-radius: var(--radius);
    border-left: 2px solid var(--border);
    background: var(--bg-secondary);
  }
  .assistant-content {
    max-height: 300px; overflow-y: auto;
    color: var(--text-secondary);
  }

  .checkpoint-divider {
    text-align: center; color: var(--accent-yellow);
    font-size: var(--font-size-small); font-family: var(--font-mono);
    padding: 8px 0; margin: 12px 0;
    border-top: 1px dashed var(--border);
    border-bottom: 1px dashed var(--border);
  }

  .empty-turns {
    text-align: center; color: var(--text-muted);
    padding: 40px; font-size: var(--font-size-base);
  }
</style>
