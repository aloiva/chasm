<script lang="ts">
  import { searchQuery, selectedSource, sortBy, loading } from '$lib/stores/sessions';
  import { invoke } from '@tauri-apps/api/core';
  import { sessions, sources } from '$lib/stores/sessions';

  async function scan() {
    loading.set(true);
    try {
      const result = await invoke('list_sessions');
      sessions.set(result as any[]);
    } catch (e) {
      console.error('Scan failed:', e);
    } finally {
      loading.set(false);
    }
  }
</script>

<div class="toolbar">
  <input
    type="text"
    class="search"
    placeholder="Search sessions..."
    bind:value={$searchQuery}
  />
  <select class="select" bind:value={$selectedSource}>
    <option value="all">All Sources</option>
    <option value="copilot-cli">Copilot CLI</option>
    <option value="vscode-copilot">VS Code</option>
  </select>
  <select class="select" bind:value={$sortBy}>
    <option value="updated">Last Modified</option>
    <option value="created">Created</option>
    <option value="turns">Turn Count</option>
    <option value="size">Size</option>
  </select>
  <button class="scan-btn" onclick={scan} disabled={$loading}>
    {$loading ? '...' : '⟳ Scan'}
  </button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
  }
  .search {
    flex: 1;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 12px;
    font-size: var(--font-size-base);
    background: var(--bg-primary);
    color: var(--text-primary);
    outline: none;
    font-family: var(--font-mono);
  }
  .search:focus { border-color: var(--accent); }
  .search::placeholder { color: var(--text-muted); }
  .select {
    padding: 5px 8px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-tertiary);
    font-size: var(--font-size-small);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    cursor: pointer;
  }
  .scan-btn {
    padding: 5px 12px;
    border-radius: var(--radius);
    border: 1px solid #238636;
    background: #238636;
    color: #fff;
    font-size: var(--font-size-small);
    font-family: var(--font-mono);
    cursor: pointer;
    font-weight: 600;
  }
  .scan-btn:hover { background: #2ea043; }
  .scan-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
