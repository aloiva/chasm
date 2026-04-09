<script lang="ts">
  import type { SessionSummary } from '$lib/types/session';
  import SourceBadge from './SourceBadge.svelte';
  import { formatDate, truncate } from '$lib/utils/format';
  import { invoke } from '@tauri-apps/api/core';
  import { selectSession, viewMode, filteredGroupedSessions, selectedGroupKey } from '$lib/stores/sessions';
  import { settings } from '$lib/stores/settings';

  const sessions = $derived<SessionSummary[]>(
    $selectedGroupKey ? ($filteredGroupedSessions[$selectedGroupKey] ?? []) : []
  );

  const totalTurns = $derived(sessions.reduce((sum, s) => sum + (s.turn_count ?? 0), 0));
  const uniqueSources = $derived([...new Set(sessions.map(s => s.source))]);

  /** Derive the working directory for new sessions based on view/group context */
  const sessionPath = $derived.by(() => {
    const key = $selectedGroupKey ?? '';
    if ($viewMode === 'folder' && key) return key;
    // Non-folder views: let the backend default to user home
    return '';
  });

  const isDobbyPath = $derived.by(() => {
    const lower = sessionPath.toLowerCase();
    const paths = ($settings.dobbyAgentsPaths || '').split(';').map(p => p.trim().toLowerCase()).filter(Boolean);
    return paths.some(p => lower.startsWith(p));
  });

  let showTypePicker = $state(false);
  let launching = $state(false);

  function closePanel() {
    selectedGroupKey.set(null);
  }

  async function startSession(type: string) {
    launching = true;
    try {
      await invoke('new_session', {
        path: sessionPath,
        sessionType: type,
      });
    } catch (e: any) {
      console.error('New session failed:', e);
    } finally {
      launching = false;
      showTypePicker = false;
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (showTypePicker) {
      const target = e.target as HTMLElement;
      if (!target.closest('.new-session-wrap')) {
        showTypePicker = false;
      }
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="group-detail" onclick={handleClickOutside}>
  <div class="gd-header">
    <div class="gd-top-row">
      <h2 class="gd-title">{$selectedGroupKey}</h2>
      <div class="gd-actions">
        <div class="new-session-wrap">
          <button
            class="new-session-btn"
            onclick={() => { showTypePicker = !showTypePicker; }}
            disabled={launching}
            title="Start a new session"
          >
            + New Session
          </button>
          {#if showTypePicker}
            <div class="type-picker">
              <button class="type-option" onclick={() => startSession('cli')}>
                Copilot CLI
              </button>
              {#if isDobbyPath && $settings.enableDobby}
                <button class="type-option" onclick={() => startSession('dobby')}>
                  Dobby
                </button>
              {/if}
            </div>
          {/if}
        </div>
        <button class="close-btn" onclick={closePanel} title="Close panel">×</button>
      </div>
    </div>
    <div class="gd-stats">
      <span class="stat">{sessions.length} session{sessions.length !== 1 ? 's' : ''}</span>
      <span class="stat">{totalTurns} turn{totalTurns !== 1 ? 's' : ''}</span>
      <span class="stat">{uniqueSources.length} source{uniqueSources.length !== 1 ? 's' : ''}</span>
    </div>
  </div>

  <div class="tile-grid">
    {#each sessions as session (session.id)}
      {@const title = session.title ?? truncate(session.first_message, 50) ?? '(unnamed)'}
      <button
        class="tile"
        class:deleted={!session.exists_on_disk}
        onclick={() => selectSession(session.id + ':' + session.source)}
      >
        <div class="tile-top">
          <SourceBadge source={session.source} />
          {#if session.turn_count}
            <span class="tile-turns">{session.turn_count} turns</span>
          {/if}
        </div>
        <div class="tile-title">{title}</div>
        {#if session.last_active}
          <div class="tile-date">{formatDate(session.last_active)}</div>
        {/if}
      </button>
    {/each}
  </div>

  {#if sessions.length === 0}
    <div class="gd-empty">No sessions in this group</div>
  {/if}
</div>

<style>
  .group-detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .gd-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .gd-top-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }

  .close-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    padding: 2px 8px;
    line-height: 1;
    flex-shrink: 0;
  }
  .close-btn:hover {
    color: var(--text-primary);
    border-color: var(--accent);
  }

  .gd-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 8px 0;
    word-break: break-all;
  }

  .gd-stats {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .stat {
    font-size: var(--font-size-small);
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    padding: 2px 8px;
    border-radius: var(--radius);
  }

  .tile-grid {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 10px;
    align-content: start;
  }

  .tile {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }
  .tile:hover {
    border-color: var(--accent);
    background: var(--bg-tertiary);
  }
  .tile.deleted {
    opacity: 0.5;
  }

  .tile-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }

  .tile-turns {
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .tile-title {
    font-size: var(--font-size-small);
    color: var(--text-primary);
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .tile-date {
    font-size: 10px;
    color: var(--text-muted);
  }

  .gd-empty {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 20px;
    font-size: var(--font-size-base);
  }

  .gd-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .new-session-wrap {
    position: relative;
  }

  .new-session-btn {
    padding: 4px 10px;
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    background: var(--bg-tertiary);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    cursor: pointer;
    white-space: nowrap;
  }
  .new-session-btn:hover {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .new-session-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .type-picker {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 10;
    min-width: 140px;
    overflow: hidden;
  }

  .type-option {
    display: block;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    text-align: left;
    cursor: pointer;
  }
  .type-option:hover {
    background: var(--bg-tertiary);
  }
</style>
