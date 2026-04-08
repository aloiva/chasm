<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import SessionList from '$lib/components/SessionList.svelte';
  import SessionDetail from '$lib/components/SessionDetail.svelte';
  import { sessions, loading, selectedSessionId } from '$lib/stores/sessions';

  onMount(async () => {
    loading.set(true);
    try {
      const result = await invoke('list_sessions');
      sessions.set(result as any[]);
    } catch (e) {
      console.error('Initial scan failed:', e);
    } finally {
      loading.set(false);
    }
  });

  const showDetail = $derived($selectedSessionId !== null);
</script>

<div class="app">
  <Toolbar />
  <div class="content">
    <div class="sidebar" class:collapsed={showDetail}>
      <SessionList />
    </div>
    {#if showDetail}
      <div class="detail-panel">
        <SessionDetail />
      </div>
    {/if}
  </div>
</div>

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
    width: 400px;
    min-width: 400px;
    flex-shrink: 0;
  }

  .detail-panel {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
