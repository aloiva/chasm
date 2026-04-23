<script lang="ts">
  import { viewMode } from '$lib/stores/sessions';

  const views = [
    { value: 'source' as const, label: 'Source' },
    { value: 'folder' as const, label: 'Folder' },
    { value: 'branch' as const, label: 'Branch' },
    { value: 'date' as const, label: 'Date' },
  ];

  let open = $state(false);

  const currentView = $derived(views.find(v => v.value === $viewMode) ?? views[0]);

  function select(value: typeof views[number]['value']) {
    viewMode.set(value);
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="view-selector">
  <button class="trigger" onclick={() => (open = !open)}>
    View: {currentView.label} ▾
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="backdrop" onclick={() => (open = false)}></div>
    <div class="dropdown">
      {#each views as view}
        <button
          class="option"
          class:active={$viewMode === view.value}
          onclick={() => select(view.value)}
        >
          {view.label}
          {#if $viewMode === view.value}
            <span class="check">✓</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .view-selector { position: relative; }
  .trigger {
    padding: 4px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    cursor: pointer;
    white-space: nowrap;
  }
  .trigger:hover { border-color: var(--accent); color: var(--text-primary); }

  .backdrop { position: fixed; inset: 0; z-index: 99; }

  .dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    z-index: 100;
    min-width: 160px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 4px 0;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .option {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    border: none;
    background: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    cursor: pointer;
    text-align: left;
  }
  .option:hover { background: var(--bg-secondary); }
  .option.active { color: var(--accent); }
  .check { margin-left: auto; color: var(--accent); font-size: 11px; }
</style>
