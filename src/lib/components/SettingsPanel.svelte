<script lang="ts">
  import { settings, updateSetting } from '$lib/stores/settings';
  import { invoke } from '@tauri-apps/api/core';

  let open = $state(false);
  let confirmingReindex = $state(false);
  let reindexStatus = $state<'idle' | 'running' | 'done' | 'error'>('idle');
  let copilotPath = $state('');
  let pathStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let pathError = $state('');

  let dobbyPaths = $state($settings.dobbyAgentsPaths || 'C:\\dobby\\agents');
  let dobbyStatus = $state<'idle' | 'saved' | 'error'>('idle');
  let dobbyError = $state('');

  async function loadCopilotPath() {
    try {
      const saved = $settings.copilotCliPath;
      if (saved) {
        copilotPath = saved;
      } else {
        copilotPath = await invoke<string>('get_copilot_cli_path');
      }
    } catch {
      copilotPath = '';
    }
  }

  // Load the path when the component mounts
  $effect(() => {
    loadCopilotPath();
  });

  function toggle() {
    open = !open;
    if (!open) {
      confirmingReindex = false;
      reindexStatus = 'idle';
      pathStatus = 'idle';
      pathError = '';
    }
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.settings-container')) {
      open = false;
      confirmingReindex = false;
    }
  }

  async function saveCopilotPath() {
    pathStatus = 'saving';
    pathError = '';
    try {
      await invoke('set_copilot_cli_path', { path: copilotPath });
      updateSetting('copilotCliPath', copilotPath);
      pathStatus = 'saved';
    } catch (e: any) {
      pathStatus = 'error';
      pathError = typeof e === 'string' ? e : e?.message || 'Failed to set path';
    }
  }

  async function resetCopilotPath() {
    try {
      const defaultPath = await invoke<string>('get_copilot_cli_path');
      copilotPath = defaultPath;
      updateSetting('copilotCliPath', '');
      await invoke('set_copilot_cli_path', { path: defaultPath });
      pathStatus = 'idle';
      pathError = '';
    } catch {
      // ignore
    }
  }

  async function saveDobbyPaths() {
    dobbyStatus = 'idle';
    dobbyError = '';
    try {
      await invoke('set_dobby_paths', { paths: dobbyPaths });
      updateSetting('dobbyAgentsPaths', dobbyPaths);
      dobbyStatus = 'saved';
    } catch (e: any) {
      dobbyStatus = 'error';
      dobbyError = typeof e === 'string' ? e : e?.message || 'Failed to set paths';
    }
  }

  function resetDobbyPaths() {
    const def = 'C:\\dobby\\agents';
    dobbyPaths = def;
    updateSetting('dobbyAgentsPaths', def);
    invoke('set_dobby_paths', { paths: def });
    dobbyStatus = 'idle';
    dobbyError = '';
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

      {#if $settings.enableDobby}
        <div class="path-setting">
          <label class="setting-sublabel">Agent Paths <span class="hint">(semicolon-separated)</span></label>
          <input
            type="text"
            class="path-input"
            placeholder="C:\dobby\agents"
            bind:value={dobbyPaths}
            onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') saveDobbyPaths(); }}
          />
          <div class="path-actions">
            <button class="path-btn" onclick={saveDobbyPaths} title="Apply">Apply</button>
            <button class="path-btn path-reset" onclick={resetDobbyPaths} title="Reset to default">Reset</button>
          </div>
          {#if dobbyStatus === 'saved'}
            <span class="path-status saved">✓ Saved</span>
          {:else if dobbyStatus === 'error'}
            <span class="path-status error">{dobbyError}</span>
          {/if}
        </div>
      {/if}

      <div class="settings-divider"></div>
      <div class="settings-header">Copilot CLI Path</div>

      <div class="path-setting">
        <input
          type="text"
          class="path-input"
          placeholder="~/.copilot"
          bind:value={copilotPath}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') saveCopilotPath(); }}
        />
        <div class="path-actions">
          <button class="path-btn" onclick={saveCopilotPath} title="Apply path">Apply</button>
          <button class="path-btn path-reset" onclick={resetCopilotPath} title="Reset to default">Reset</button>
        </div>
        {#if pathStatus === 'saved'}
          <span class="path-status saved">✓ Saved — refresh to reload sessions</span>
        {:else if pathStatus === 'error'}
          <span class="path-status error">{pathError}</span>
        {/if}
      </div>

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

  .path-setting {
    padding: 6px 12px;
  }

  .path-input {
    width: 100%;
    padding: 4px 8px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    box-sizing: border-box;
  }
  .path-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .path-actions {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }

  .path-btn {
    padding: 3px 8px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--accent);
    color: var(--bg-primary);
    font-size: var(--font-size-small);
    font-family: var(--font-mono);
    cursor: pointer;
  }
  .path-btn:hover {
    opacity: 0.9;
  }

  .path-reset {
    background: var(--bg-secondary);
    color: var(--text-secondary);
  }
  .path-reset:hover {
    color: var(--text-primary);
  }

  .path-status {
    display: block;
    margin-top: 4px;
    font-size: 11px;
    font-family: var(--font-mono);
  }
  .path-status.saved {
    color: var(--accent);
  }
  .path-status.error {
    color: var(--danger);
  }
  .setting-sublabel {
    font-size: 11px;
    color: var(--text-secondary);
    margin-bottom: 4px;
    font-family: var(--font-mono);
  }
  .hint {
    opacity: 0.6;
  }
</style>
