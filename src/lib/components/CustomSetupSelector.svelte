<script lang="ts">
  import {
    customSetups,
    activeSetupId,
    applySetup,
    saveCurrentAsSetup,
    deleteSetup,
    deleteAllUserSetups,
    clearActiveSetup,
  } from '$lib/stores/customSetups';
  import { settings } from '$lib/stores/settings';

  let open = $state(false);
  let saving = $state(false);
  let newName = $state('');

  const visibleSetups = $derived(
    $settings.enableDobby ? $customSetups : $customSetups.filter(s => s.id !== 'dobby')
  );
  const activeSetup = $derived(visibleSetups.find((s) => s.id === $activeSetupId));
  const hasUserSetups = $derived(visibleSetups.some(s => !s.builtIn));

  function handleSelect(setup: (typeof $customSetups)[number]) {
    applySetup(setup);
    open = false;
  }

  function handleSave() {
    if (newName.trim()) {
      saveCurrentAsSetup(newName.trim());
      newName = '';
      saving = false;
      open = false;
    }
  }

  function handleDelete(e: MouseEvent, id: string) {
    e.stopPropagation();
    deleteSetup(id);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
      saving = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="setup-selector">
  <button class="trigger" onclick={() => (open = !open)}>
    {activeSetup ? `Setup: ${activeSetup.name}` : 'Setups'} ▾
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div
      class="backdrop"
      onclick={() => {
        open = false;
        saving = false;
      }}
    ></div>
    <div class="dropdown">
      <div class="panel-header">
        <span>Custom Setups</span>
        {#if $activeSetupId}
          <button
            class="clear-btn"
            onclick={() => {
              clearActiveSetup();
              open = false;
            }}>Reset</button
          >
        {/if}
      </div>

      {#each visibleSetups as setup}
        <div class="option-row">
          <button
            class="option"
            class:active={$activeSetupId === setup.id}
            onclick={() => handleSelect(setup)}
          >
            <span class="option-name">{setup.name}</span>
            {#if setup.builtIn}
              <span class="built-in">built-in</span>
            {/if}
            {#if $activeSetupId === setup.id}
              <span class="check">✓</span>
            {/if}
          </button>
          {#if !setup.builtIn}
            <button
              class="delete-btn"
              onclick={(e: MouseEvent) => handleDelete(e, setup.id)}
              title="Delete setup"
            >
              ×
            </button>
          {/if}
        </div>
      {/each}

      <div class="divider"></div>

      {#if saving}
        <div class="save-row">
          <input
            type="text"
            class="save-input"
            placeholder="Setup name..."
            bind:value={newName}
            onkeydown={(e: KeyboardEvent) => {
              if (e.key === 'Enter') handleSave();
            }}
          />
          <button class="save-confirm" onclick={handleSave}>Save</button>
        </div>
      {:else}
        <button class="option save-trigger" onclick={() => (saving = true)}>
          + Save current as setup
        </button>
      {/if}

      {#if hasUserSetups}
        <div class="divider"></div>
        <button class="option remove-all" onclick={() => deleteAllUserSetups()}>
          🗑 Remove all custom setups
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .setup-selector {
    position: relative;
  }

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
  .trigger:hover {
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 100;
    min-width: 220px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 4px 0;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--font-size-small);
    font-weight: 600;
    color: var(--text-primary);
    padding: 6px 12px 6px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 4px;
  }

  .clear-btn {
    border: none;
    background: none;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    padding: 2px 6px;
  }
  .clear-btn:hover {
    text-decoration: underline;
  }

  .option-row {
    display: flex;
    align-items: center;
  }

  .option {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    padding: 6px 12px;
    border: none;
    background: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    cursor: pointer;
    text-align: left;
  }
  .option:hover {
    background: var(--bg-secondary);
  }
  .option.active {
    color: var(--accent);
  }

  .option-name {
    flex: 1;
  }

  .built-in {
    font-size: 9px;
    color: var(--text-muted);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 1px 4px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .check {
    color: var(--accent);
    font-size: 11px;
  }

  .delete-btn {
    border: none;
    background: none;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    padding: 4px 8px;
    line-height: 1;
  }
  .delete-btn:hover {
    color: var(--error);
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }

  .save-trigger {
    color: var(--text-muted);
    font-style: italic;
  }

  .remove-all {
    color: var(--error, #f85149);
    font-size: 11px;
  }
  .remove-all:hover {
    background: rgba(248, 81, 73, 0.1);
  }

  .save-row {
    display: flex;
    gap: 4px;
    padding: 4px 8px;
  }

  .save-input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }
  .save-input:focus {
    border-color: var(--accent);
    outline: none;
  }

  .save-confirm {
    padding: 4px 10px;
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    background: var(--accent);
    color: var(--bg-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    font-weight: 600;
    cursor: pointer;
  }
  .save-confirm:hover {
    opacity: 0.9;
  }
</style>
