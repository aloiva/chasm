<script lang="ts">
  import { settings, updateSetting } from '$lib/stores/settings';
  import { invoke } from '@tauri-apps/api/core';

  let open = $state(false);
  let confirmingReindex = $state(false);
  let reindexStatus = $state<'idle' | 'running' | 'done' | 'error'>('idle');

  function toggle() {
    open = !open;
    if (!open) {
      confirmingReindex = false;
      reindexStatus = 'idle';
    }
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.settings-container')) {
      open = false;
      confirmingReindex = false;
    }
  }

  async function runReindex() {
    reindexStatus = 'running';
    try {
      await invoke('reindex_sessions');
      reindexStatus = 'done';
    } catch (e) {
      console.error('Reindex failed:', e);
      reindexStatus = 'error';
    }
    confirmingReindex = false;
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="settings-container">
  <button class="settings-btn" onclick={toggle} title="Settings">
    ⚙
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="settings-dropdown" onclick={(e: MouseEvent) => e.stopPropagation()}>
      <div class="settings-header">Settings</div>

      <label class="setting-row">
        <input
          type="checkbox"
          checked={$settings.enableDobby}
          onchange={() => updateSetting('enableDobby', !$settings.enableDobby)}
        />
        <span class="setting-label">Enable Dobby</span>
      </label>

      <div class="settings-divider"></div>
      <div class="settings-header">Experimental</div>

      {#if confirmingReindex}
        <div class="confirm-box">
          <p class="confirm-text">This will open a Copilot CLI terminal and run <code>/chronicle reindex</code> to rebuild the session index. Stale entries will be removed.</p>
          <div class="confirm-actions">
            <button class="confirm-btn confirm-yes" onclick={runReindex}>Reindex</button>
            <button class="confirm-btn confirm-no" onclick={() => { confirmingReindex = false; }}>Cancel</button>
          </div>
        </div>
      {:else}
        <button class="action-row" onclick={() => { confirmingReindex = true; reindexStatus = 'idle'; }}>
          <span>🔄 Reindex Sessions</span>
          {#if reindexStatus === 'done'}
            <span class="status-badge done">✓ Started</span>
          {:else if reindexStatus === 'error'}
            <span class="status-badge error">✗ Failed</span>
          {/if}
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .settings-container {
    position: relative;
  }

  .settings-btn {
    padding: 5px 8px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: 14px;
    cursor: pointer;
  }
  .settings-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-secondary);
  }

  .settings-dropdown {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    min-width: 200px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px 0;
    z-index: 100;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .settings-header {
    padding: 4px 12px 8px;
    font-size: var(--font-size-small);
    color: var(--text-secondary);
    font-weight: 600;
    border-bottom: 1px solid var(--border);
    margin-bottom: 4px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    color: var(--text-primary);
  }
  .setting-row:hover {
    background: var(--bg-secondary);
  }

  .setting-label {
    user-select: none;
  }

  .settings-divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }

  .action-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    color: var(--text-primary);
    background: none;
    border: none;
    text-align: left;
  }
  .action-row:hover {
    background: var(--bg-secondary);
  }

  .confirm-box {
    padding: 8px 12px;
  }

  .confirm-text {
    font-size: var(--font-size-small);
    color: var(--text-secondary);
    margin: 0 0 8px;
    line-height: 1.4;
  }

  .confirm-text code {
    color: var(--accent);
    font-family: var(--font-mono);
  }

  .confirm-actions {
    display: flex;
    gap: 6px;
  }

  .confirm-btn {
    padding: 4px 10px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    font-size: var(--font-size-small);
    font-family: var(--font-mono);
    cursor: pointer;
  }

  .confirm-yes {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
  }
  .confirm-yes:hover {
    opacity: 0.9;
  }

  .confirm-no {
    background: var(--bg-secondary);
    color: var(--text-secondary);
  }
  .confirm-no:hover {
    color: var(--text-primary);
  }

  .status-badge {
    font-size: 11px;
    padding: 1px 6px;
    border-radius: var(--radius);
  }
  .status-badge.done {
    color: var(--accent);
  }
  .status-badge.error {
    color: var(--danger);
  }
</style>
