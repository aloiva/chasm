<script lang="ts">
  import { searchQuery, sortBy } from '$lib/stores/sessions';
  import { invoke } from '@tauri-apps/api/core';
  import { sources } from '$lib/stores/sessions';
  import ViewSelector from './ViewSelector.svelte';
  import FilterPanel from './FilterPanel.svelte';
  import CustomSetupSelector from './CustomSetupSelector.svelte';
  import SettingsPanel from './SettingsPanel.svelte';
  import type { SourceInfo } from '$lib/types/session';
  import { onMount } from 'svelte';

  onMount(async () => {
    try {
      const result = await invoke('get_available_sources') as SourceInfo[];
      sources.set(result);
    } catch (e) {
      console.error('Failed to fetch sources:', e);
    }
  });
</script>

<div class="toolbar">
  <div class="search-wrap">
    <input
      type="text"
      class="search"
      placeholder="Search sessions... (comma-separated)"
      bind:value={$searchQuery}
    />
    {#if $searchQuery}
      <button class="search-clear" onclick={() => searchQuery.set('')} aria-label="Clear search">×</button>
    {/if}
  </div>
  <ViewSelector />
  <select class="select" bind:value={$sortBy}>
    <option value="updated">Sort: Modified</option>
    <option value="created">Sort: Created</option>
    <option value="turns">Sort: Turns</option>
    <option value="size">Sort: Size</option>
    <option value="title">Sort: Title (A-Z)</option>
    <option value="branch">Sort: Branch</option>
    <option value="folder">Sort: Folder</option>
    <option value="source">Sort: Source</option>
  </select>
  <FilterPanel />
  <CustomSetupSelector />
  <SettingsPanel />
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
  .search-wrap {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }
  .search {
    width: 100%;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 28px 6px 12px;
    font-size: var(--font-size-base);
    background: var(--bg-primary);
    color: var(--text-primary);
    outline: none;
    font-family: var(--font-mono);
  }
  .search:focus { border-color: var(--accent); }
  .search::placeholder { color: var(--text-muted); }
  .search-clear {
    position: absolute;
    right: 6px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 16px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }
  .search-clear:hover { color: var(--text-primary); }
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
</style>
