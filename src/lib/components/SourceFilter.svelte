<script lang="ts">
  import type { SourceInfo } from '$lib/types/session';
  import { selectedSources } from '$lib/stores/sessions';

  let { sources }: { sources: SourceInfo[] } = $props();

  let open = $state(false);

  function toggle(name: string) {
    selectedSources.update(current => {
      const next = new Set(current);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  }

  const filterCount = $derived($selectedSources.size);
  const label = $derived(filterCount > 0 ? `Sources (${filterCount})` : 'Sources');

  const availableNames = $derived(sources.filter(s => s.available).map(s => s.name));
  const allSelected = $derived(availableNames.length > 0 && availableNames.every(n => $selectedSources.has(n)));

  function selectAll() {
    selectedSources.set(new Set(availableNames));
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="source-dropdown">
  <button class="dropdown-btn" class:has-filter={filterCount > 0} onclick={() => open = !open}>
    {label} <span class="caret">{open ? '▲' : '▼'}</span>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="dropdown-backdrop" onclick={() => open = false}></div>
    <div class="dropdown-panel">
      <div class="dropdown-actions">
        <button class="action-btn" onclick={selectAll} disabled={allSelected}>Select All</button>
        <button class="action-btn" onclick={() => selectedSources.set(new Set())} disabled={filterCount === 0}>Clear All</button>
      </div>
      <div class="dropdown-divider"></div>
      {#each sources.filter(s => s.available) as source (source.name)}
        <label class="dropdown-row">
          <input
            type="checkbox"
            checked={$selectedSources.has(source.name)}
            onchange={() => toggle(source.name)}
          />
          <span class="row-icon">{source.icon}</span>
          <span class="row-label">{source.display_name}</span>
        </label>
      {/each}
    </div>
  {/if}
</div>

<style>
  .source-dropdown {
    position: relative;
  }

  .dropdown-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    cursor: pointer;
    white-space: nowrap;
  }

  .dropdown-btn.has-filter {
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .dropdown-btn:hover {
    border-color: var(--text-secondary);
    color: var(--text-primary);
  }

  .caret {
    font-size: 8px;
    color: var(--text-muted);
  }

  .dropdown-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .dropdown-panel {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    z-index: 100;
    min-width: 180px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 4px 0;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .dropdown-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    color: var(--text-primary);
    transition: background 0.1s;
  }

  .dropdown-row:hover {
    background: var(--bg-secondary);
  }

  .dropdown-row input[type="checkbox"] {
    accent-color: var(--accent);
    cursor: pointer;
  }

  .row-icon {
    font-size: 10px;
    width: 20px;
    text-align: center;
  }

  .row-label {
    flex: 1;
  }

  .dropdown-divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }

  .dropdown-actions {
    display: flex;
    gap: 4px;
    padding: 4px 8px;
  }

  .action-btn {
    flex: 1;
    padding: 3px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    cursor: pointer;
    white-space: nowrap;
  }

  .action-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
