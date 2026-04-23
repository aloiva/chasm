<script lang="ts">
  import { settings, updateSetting } from '$lib/stores/settings';
  import { invoke } from '@tauri-apps/api/core';

  let open = $state(false);
  let confirmingReindex = $state(false);
  let reindexStatus = $state<'idle' | 'running' | 'done' | 'error'>('idle');

  let copilotPath = $state('');
  let copilotDbPath = $state('');
  let pathStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let pathError = $state('');
  let dbPathStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let dbPathError = $state('');
  let agentvizPath = $state('');
  let agentvizPathStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let agentvizPathError = $state('');
  let vscodePath = $state('');
  let vscodePathStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let vscodePathError = $state('');

  // Cache state
  let cachePath = $state('');
  let cachePathStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let cachePathError = $state('');
  let cacheEnabled = $state(true);
  let clearCacheStatus = $state<'idle' | 'clearing' | 'cleared' | 'error'>('idle');

  // Setup popup state
  let showAgentvizSetup = $state(false);
  let setupPath = $state('');
  let setupPathStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let setupPathError = $state('');

  // Max sessions state
  let maxSessions = $state(3);

  async function loadPaths() {
    try {
      const savedPath = $settings.copilotCliPath;
      copilotPath = savedPath || await invoke<string>('get_copilot_cli_path');
    } catch {
      copilotPath = '';
    }
    try {
      const savedDb = $settings.copilotDbPath;
      copilotDbPath = savedDb || await invoke<string>('get_copilot_db_path');
    } catch {
      copilotDbPath = '';
    }
    agentvizPath = $settings.agentvizPath || '';
    maxSessions = $settings.agentvizMaxSessions;
    try {
      const savedVscode = $settings.vscodeWorkspacePath;
      vscodePath = savedVscode || await invoke<string>('get_vscode_workspace_path');
    } catch {
      vscodePath = '';
    }
    try {
      cachePath = $settings.cacheDir || await invoke<string>('get_cache_dir');
    } catch {
      cachePath = '';
    }
    try {
      cacheEnabled = $settings.cacheEnabled !== undefined ? $settings.cacheEnabled : await invoke<boolean>('get_cache_enabled');
    } catch {
      cacheEnabled = true;
    }
  }

  $effect(() => {
    loadPaths();
  });

  function toggle() {
    open = !open;
    if (!open) {
      confirmingReindex = false;
      reindexStatus = 'idle';
      pathStatus = 'idle';
      pathError = '';
      dbPathStatus = 'idle';
      dbPathError = '';
    }
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.settings-container') && !target.closest('.modal-overlay')) {
      open = false;
      confirmingReindex = false;
    }
  }

  function handleEnableAgentviz() {
    if ($settings.enableAgentviz) {
      // Turning off
      updateSetting('enableAgentviz', false);
    } else {
      // Turning on — show setup popup if no path configured
      if (!$settings.agentvizPath) {
        setupPath = '';
        setupPathStatus = 'idle';
        setupPathError = '';
        showAgentvizSetup = true;
      } else {
        updateSetting('enableAgentviz', true);
      }
    }
  }

  async function confirmSetup() {
    setupPathStatus = 'saving';
    setupPathError = '';
    try {
      await invoke('validate_agentviz_path', { path: setupPath });
      updateSetting('agentvizPath', setupPath);
      updateSetting('enableAgentviz', true);
      agentvizPath = setupPath;
      agentvizPathStatus = 'idle';
      showAgentvizSetup = false;
    } catch (e: any) {
      setupPathStatus = 'error';
      setupPathError = typeof e === 'string' ? e : e?.message || 'Invalid path';
    }
  }

  function cancelSetup() {
    showAgentvizSetup = false;
  }

  async function saveCopilotPath() {
    pathStatus = 'saving';
    pathError = '';
    try {
      await invoke('set_copilot_cli_path', { path: copilotPath });
      updateSetting('copilotCliPath', copilotPath);
      pathStatus = 'saved';
      window.dispatchEvent(new CustomEvent('chasm-rescan'));
    } catch (e: any) {
      pathStatus = 'error';
      pathError = typeof e === 'string' ? e : e?.message || 'Failed to set path';
    }
  }

  async function resetCopilotPath() {
    try {
      updateSetting('copilotCliPath', '');
      copilotPath = await invoke<string>('get_copilot_cli_path');
      pathStatus = 'idle';
      pathError = '';
    } catch { /* ignore */ }
  }

  async function saveCopilotDbPath() {
    dbPathStatus = 'saving';
    dbPathError = '';
    try {
      await invoke('set_copilot_db_path', { path: copilotDbPath });
      updateSetting('copilotDbPath', copilotDbPath);
      dbPathStatus = 'saved';
      window.dispatchEvent(new CustomEvent('chasm-rescan'));
    } catch (e: any) {
      dbPathStatus = 'error';
      dbPathError = typeof e === 'string' ? e : e?.message || 'Failed to set path';
    }
  }

  async function resetCopilotDbPath() {
    try {
      updateSetting('copilotDbPath', '');
      copilotDbPath = await invoke<string>('get_copilot_db_path');
      dbPathStatus = 'idle';
      dbPathError = '';
    } catch { /* ignore */ }
  }

  async function saveVscodePath() {
    vscodePathStatus = 'saving';
    vscodePathError = '';
    try {
      await invoke('set_vscode_workspace_path', { path: vscodePath });
      updateSetting('vscodeWorkspacePath', vscodePath);
      vscodePathStatus = 'saved';
      window.dispatchEvent(new CustomEvent('chasm-rescan'));
    } catch (e: any) {
      vscodePathStatus = 'error';
      vscodePathError = typeof e === 'string' ? e : e?.message || 'Failed to set path';
    }
  }

  async function resetVscodePath() {
    try {
      updateSetting('vscodeWorkspacePath', '');
      vscodePath = await invoke<string>('get_vscode_workspace_path');
      vscodePathStatus = 'idle';
      vscodePathError = '';
    } catch { /* ignore */ }
  }

  async function toggleCacheEnabled() {
    cacheEnabled = !cacheEnabled;
    try {
      await invoke('set_cache_enabled', { enabled: cacheEnabled });
      updateSetting('cacheEnabled', cacheEnabled);
    } catch { /* ignore */ }
  }

  async function saveCachePath() {
    cachePathStatus = 'saving';
    cachePathError = '';
    try {
      await invoke('set_cache_dir', { path: cachePath });
      updateSetting('cacheDir', cachePath);
      cachePathStatus = 'saved';
    } catch (e: any) {
      cachePathStatus = 'error';
      cachePathError = typeof e === 'string' ? e : e?.message || 'Failed to set path';
    }
  }

  async function resetCachePath() {
    try {
      updateSetting('cacheDir', '');
      cachePath = await invoke<string>('get_cache_dir');
      cachePathStatus = 'idle';
      cachePathError = '';
    } catch { /* ignore */ }
  }

  async function clearCache() {
    clearCacheStatus = 'clearing';
    try {
      await invoke('clear_cache');
      clearCacheStatus = 'cleared';
      setTimeout(() => { clearCacheStatus = 'idle'; }, 2000);
    } catch {
      clearCacheStatus = 'error';
    }
  }

  async function saveAgentvizPath() {
    agentvizPathStatus = 'saving';
    agentvizPathError = '';
    try {
      await invoke('validate_agentviz_path', { path: agentvizPath });
      updateSetting('agentvizPath', agentvizPath);
      agentvizPathStatus = 'saved';
    } catch (e: any) {
      agentvizPathStatus = 'error';
      agentvizPathError = typeof e === 'string' ? e : e?.message || 'Invalid path';
    }
  }

  function resetAgentvizPath() {
    updateSetting('agentvizPath', '');
    agentvizPath = '';
    agentvizPathStatus = 'idle';
    agentvizPathError = '';
  }

  async function onMaxSessionsChange() {
    if (maxSessions < 1 || isNaN(maxSessions)) return;
    updateSetting('agentvizMaxSessions', maxSessions);
    // Kill excess processes when lowering the limit
    try {
      await invoke('trim_agentviz', { maxSessions: maxSessions });
    } catch { /* ignore */ }
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

      <label class="setting-row">
        <input
          type="checkbox"
          checked={$settings.enableAgentviz}
          onclick={(e: MouseEvent) => { e.preventDefault(); handleEnableAgentviz(); }}
        />
        <span class="setting-label">Enable agentviz</span>
      </label>

      {#if $settings.enableAgentviz}
        <div class="path-setting">
          <input
            type="text"
            class="path-input"
            placeholder="C:\dev\repos\agentviz"
            bind:value={agentvizPath}
            onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') saveAgentvizPath(); }}
          />
          <div class="path-actions">
            <button class="path-btn" onclick={saveAgentvizPath} title="Apply path">Apply</button>
            <button class="path-btn path-reset" onclick={resetAgentvizPath} title="Clear path">Reset</button>
          </div>
          {#if agentvizPathStatus === 'saved'}
            <span class="path-status saved">✓ Applied</span>
          {:else if agentvizPathStatus === 'error'}
            <span class="path-status error">{agentvizPathError}</span>
          {/if}
        </div>

        <div class="path-setting">
          <label class="port-label">Max parallel sessions</label>
          <div class="port-range-row">
            <input
              type="number"
              class="port-input"
              min="1"
              bind:value={maxSessions}
              onchange={onMaxSessionsChange}
            />
          </div>
        </div>
      {/if}

      <div class="settings-divider"></div>
      <div class="settings-header">Copilot CLI Sessions Path</div>

      <div class="path-setting">
        <input
          type="text"
          class="path-input"
          placeholder="~/.copilot/session-state"
          bind:value={copilotPath}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') saveCopilotPath(); }}
        />
        <div class="path-actions">
          <button class="path-btn" onclick={saveCopilotPath} title="Apply path">Apply</button>
          <button class="path-btn path-reset" onclick={resetCopilotPath} title="Reset to default">Reset</button>
        </div>
        {#if pathStatus === 'saved'}
          <span class="path-status saved">✓ Applied</span>
        {:else if pathStatus === 'error'}
          <span class="path-status error">{pathError}</span>
        {/if}
      </div>

      <div class="settings-divider"></div>
      <div class="settings-header">Session Store DB Path</div>

      <div class="path-setting">
        <input
          type="text"
          class="path-input"
          placeholder="~/.copilot/session-store.db"
          bind:value={copilotDbPath}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') saveCopilotDbPath(); }}
        />
        <div class="path-actions">
          <button class="path-btn" onclick={saveCopilotDbPath} title="Apply path">Apply</button>
          <button class="path-btn path-reset" onclick={resetCopilotDbPath} title="Reset to default">Reset</button>
        </div>
        {#if dbPathStatus === 'saved'}
          <span class="path-status saved">✓ Applied</span>
        {:else if dbPathStatus === 'error'}
          <span class="path-status error">{dbPathError}</span>
        {/if}
      </div>

      <div class="settings-divider"></div>
      <div class="settings-header">VS Code Workspace Storage Path</div>

      <div class="path-setting">
        <input
          type="text"
          class="path-input"
          placeholder="%APPDATA%/Code/User/workspaceStorage"
          bind:value={vscodePath}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') saveVscodePath(); }}
        />
        <div class="path-actions">
          <button class="path-btn" onclick={saveVscodePath} title="Apply path">Apply</button>
          <button class="path-btn path-reset" onclick={resetVscodePath} title="Reset to default">Reset</button>
        </div>
        {#if vscodePathStatus === 'saved'}
          <span class="path-status saved">✓ Applied</span>
        {:else if vscodePathStatus === 'error'}
          <span class="path-status error">{vscodePathError}</span>
        {/if}
      </div>

      <div class="settings-divider"></div>
      <div class="settings-header">Cache</div>

      <label class="setting-row">
        <span>Enable disk cache</span>
        <input type="checkbox" checked={cacheEnabled} onchange={toggleCacheEnabled} />
      </label>

      <div class="path-setting">
        <input
          type="text"
          class="path-input"
          placeholder="~/.chasm/cache/"
          bind:value={cachePath}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') saveCachePath(); }}
        />
        <div class="path-actions">
          <button class="path-btn" onclick={saveCachePath} title="Apply cache path">Apply</button>
          <button class="path-btn path-reset" onclick={resetCachePath} title="Reset to default">Reset</button>
        </div>
        {#if cachePathStatus === 'saved'}
          <span class="path-status saved">✓ Applied</span>
        {:else if cachePathStatus === 'error'}
          <span class="path-status error">{cachePathError}</span>
        {/if}
      </div>

      <button class="action-row" onclick={clearCache}>
        <span>🗑 Clear Cache</span>
        {#if clearCacheStatus === 'cleared'}
          <span class="status-badge done">✓ Cleared</span>
        {:else if clearCacheStatus === 'error'}
          <span class="status-badge error">Failed</span>
        {/if}
      </button>

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

{#if showAgentvizSetup}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-overlay" onclick={cancelSetup}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content" onclick={(e: MouseEvent) => e.stopPropagation()}>
      <div class="modal-header">Set up agentviz</div>
      <div class="modal-body">
        <p class="modal-instructions">Clone and build agentviz, then enter the path below:</p>
        <div class="modal-commands">
          <code>git clone https://github.com/jayparikh/agentviz</code>
          <code>cd agentviz && npm install && npm run build</code>
        </div>
        <input
          type="text"
          class="path-input"
          placeholder="C:\dev\repos\agentviz"
          bind:value={setupPath}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') confirmSetup(); }}
        />
        {#if setupPathStatus === 'error'}
          <span class="path-status error">{setupPathError}</span>
        {/if}
      </div>
      <div class="modal-actions">
        <button class="confirm-btn confirm-yes" onclick={confirmSetup}>Apply</button>
        <button class="confirm-btn confirm-no" onclick={cancelSetup}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

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

  .port-label {
    font-size: var(--font-size-small);
    font-family: var(--font-mono);
    color: var(--text-secondary);
    margin-bottom: 4px;
    display: block;
  }

  .port-range-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .port-input {
    width: 70px;
    padding: 4px 6px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }
  .port-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .modal-content {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
    min-width: 360px;
    max-width: 440px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 12px;
  }

  .modal-body {
    margin-bottom: 12px;
  }

  .modal-instructions {
    font-size: var(--font-size-small);
    color: var(--text-secondary);
    margin: 0 0 8px;
    line-height: 1.4;
  }

  .modal-commands {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 10px;
    user-select: all;
  }
  .modal-commands code {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    background: var(--bg-primary);
    padding: 3px 6px;
    border-radius: 2px;
    display: block;
  }

  .modal-actions {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }
</style>
