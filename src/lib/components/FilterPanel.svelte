<script lang="ts">
  import { filters, resetFilters, activeFilterCount } from '$lib/stores/sessions';

  let open = $state(false);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }

  function updateFilter<K extends keyof typeof $filters>(key: K, value: typeof $filters[K]) {
    filters.update(f => ({ ...f, [key]: value }));
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="filter-panel">
  <button class="trigger" onclick={() => (open = !open)}>
    ⚙ Filters
    {#if $activeFilterCount > 0}
      <span class="badge">{$activeFilterCount}</span>
    {/if}
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="backdrop" onclick={() => (open = false)}></div>
    <div class="dropdown">
      <div class="panel-header">
        <span>Filters</span>
        {#if $activeFilterCount > 0}
          <button class="clear-btn" onclick={() => resetFilters()}>Clear all</button>
        {/if}
      </div>

      <div class="filter-group">
        <label class="filter-label">Folder starts with</label>
        <input
          type="text"
          class="filter-input"
          placeholder="e.g. C:\projects"
          value={$filters.folderStartsWith}
          oninput={(e: Event) => updateFilter('folderStartsWith', (e.target as HTMLInputElement).value)}
        />
      </div>

      <div class="filter-group">
        <label class="filter-label">Folder contains</label>
        <input
          type="text"
          class="filter-input"
          placeholder="e.g. my-app"
          value={$filters.folderContains}
          oninput={(e: Event) => updateFilter('folderContains', (e.target as HTMLInputElement).value)}
        />
      </div>

      <div class="filter-group">
        <label class="filter-label">Branch</label>
        <input
          type="text"
          class="filter-input"
          placeholder="e.g. main"
          value={$filters.branch}
          oninput={(e: Event) => updateFilter('branch', (e.target as HTMLInputElement).value)}
        />
      </div>

      <div class="filter-group">
        <label class="filter-label">Min turns</label>
        <input
          type="number"
          class="filter-input"
          placeholder="e.g. 5"
          value={$filters.minTurns ?? ''}
          oninput={(e: Event) => {
            const v = (e.target as HTMLInputElement).value;
            updateFilter('minTurns', v ? parseInt(v) : null);
          }}
        />
      </div>

      <div class="filter-group">
        <label class="filter-label">Max turns</label>
        <input
          type="number"
          class="filter-input"
          placeholder="e.g. 50"
          value={$filters.maxTurns ?? ''}
          oninput={(e: Event) => {
            const v = (e.target as HTMLInputElement).value;
            updateFilter('maxTurns', v ? parseInt(v) : null);
          }}
        />
      </div>

      <div class="filter-row">
        <label class="filter-label">Checkpoints</label>
        <select
          class="filter-select"
          value={$filters.hasCheckpoints === null ? 'any' : $filters.hasCheckpoints ? 'yes' : 'no'}
          onchange={(e: Event) => {
            const v = (e.target as HTMLSelectElement).value;
            updateFilter('hasCheckpoints', v === 'any' ? null : v === 'yes');
          }}
        >
          <option value="any">Any</option>
          <option value="yes">Has checkpoints</option>
          <option value="no">No checkpoints</option>
        </select>
      </div>

      <div class="filter-row">
        <label class="filter-label">On disk</label>
        <select
          class="filter-select"
          value={$filters.existsOnDisk === null ? 'any' : $filters.existsOnDisk ? 'yes' : 'no'}
          onchange={(e: Event) => {
            const v = (e.target as HTMLSelectElement).value;
            updateFilter('existsOnDisk', v === 'any' ? null : v === 'yes');
          }}
        >
          <option value="any">Any</option>
          <option value="yes">Exists</option>
          <option value="no">Deleted</option>
        </select>
      </div>

      <div class="filter-row">
        <label class="filter-label">Hide deleted</label>
        <input
          type="checkbox"
          class="filter-checkbox"
          checked={$filters.hideDeleted}
          onchange={() => updateFilter('hideDeleted', !$filters.hideDeleted)}
        />
      </div>

      <div class="filter-row">
        <label class="filter-label">Status</label>
        <select
          class="filter-select"
          value={$filters.status ?? 'any'}
          onchange={(e: Event) => {
            const v = (e.target as HTMLSelectElement).value;
            updateFilter('status', v === 'any' ? null : v);
          }}
        >
          <option value="any">Any</option>
          <option value="recent">Recent (active)</option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">Date range</label>
        <div class="date-row">
          <input
            type="date"
            class="filter-input date"
            value={$filters.dateFrom}
            oninput={(e: Event) => updateFilter('dateFrom', (e.target as HTMLInputElement).value)}
          />
          <span class="date-sep">→</span>
          <input
            type="date"
            class="filter-input date"
            value={$filters.dateTo}
            oninput={(e: Event) => updateFilter('dateTo', (e.target as HTMLInputElement).value)}
          />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .filter-panel { position: relative; }

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
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .trigger:hover { border-color: var(--accent); color: var(--text-primary); }

  .badge {
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 10px;
    padding: 0 5px;
    border-radius: 8px;
    font-weight: 700;
    min-width: 16px;
    text-align: center;
  }

  .backdrop { position: fixed; inset: 0; z-index: 99; }

  .dropdown {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 100;
    width: 280px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    max-height: 420px;
    overflow-y: auto;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--font-size-small);
    font-weight: 600;
    color: var(--text-primary);
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 8px;
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
  .clear-btn:hover { text-decoration: underline; }

  .filter-group { margin-bottom: 8px; }
  .filter-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .filter-label {
    display: block;
    font-size: 10px;
    color: var(--text-muted);
    margin-bottom: 3px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .filter-input {
    width: 100%;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }
  .filter-input:focus { border-color: var(--accent); outline: none; }
  .filter-input.date { width: 110px; }

  .filter-select {
    padding: 3px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .filter-checkbox {
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .date-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .date-sep { color: var(--text-muted); font-size: 12px; }
</style>
