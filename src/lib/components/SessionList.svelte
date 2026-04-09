<script lang="ts">
  import type { SessionSummary } from '$lib/types/session';
  import SessionCard from './SessionCard.svelte';
  import {
    filteredGroupedSessions,
    filteredSessions,
    viewMode,
    collapsedGroups,
    selectedGroupKey,
    groupFilter,
    selectGroup,
    selectSession,
  } from '$lib/stores/sessions';

  let { oncontextmenu, ongroupcontextmenu }: {
    oncontextmenu?: (e: MouseEvent, session: SessionSummary) => void;
    ongroupcontextmenu?: (e: MouseEvent, key: string) => void;
  } = $props();

  function resolveHeader(key: string): string {
    if ($viewMode === 'source') return key.toUpperCase();
    return key;
  }

  function toggleCollapse(key: string) {
    collapsedGroups.update(s => {
      const next = new Set(s);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  const groupKeys = $derived(Object.keys($filteredGroupedSessions));

  function collapseAll() {
    collapsedGroups.set(new Set(groupKeys));
  }

  function expandAll() {
    collapsedGroups.set(new Set());
  }
</script>

<div class="list-container">
  <div class="list-toolbar">
    <button class="tb-btn" onclick={collapseAll} title="Collapse all groups">⊟</button>
    <button class="tb-btn" onclick={expandAll} title="Expand all groups">⊞</button>
    <div class="group-filter-wrap">
      <input
        type="text"
        class="group-filter"
        placeholder="Filter groups... (comma-separated, /regex/)"
        bind:value={$groupFilter}
      />
      {#if $groupFilter}
        <button class="group-filter-clear" onclick={() => groupFilter.set('')} aria-label="Clear filter">×</button>
      {/if}
    </div>
  </div>

  <div class="list">
    {#each Object.entries($filteredGroupedSessions) as [key, sessions] (key)}
      {@const isCollapsed = $collapsedGroups.has(key)}
      {@const isSelected = $selectedGroupKey === key}
      <div class="group">
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="group-header"
          class:selected={isSelected}
          class:collapsed-header={isCollapsed}
          onclick={() => toggleCollapse(key)}
          ondblclick={() => selectGroup(key)}
          oncontextmenu={(e: MouseEvent) => { e.preventDefault(); ongroupcontextmenu?.(e, key); }}
        >
          <span class="chevron">
            {isCollapsed ? '▸' : '▾'}
          </span>
          <span class="group-label">{resolveHeader(key)}</span>
          <span class="group-count">{sessions.length}</span>
          <button
            class="group-open-btn"
            onclick={(e: MouseEvent) => { e.stopPropagation(); selectGroup(key); }}
            title="Open in panel"
          >View →</button>
        </div>
        {#if !isCollapsed}
          {#each sessions as session (session.id)}
            <SessionCard {session} {oncontextmenu} />
          {/each}
        {/if}
      </div>
    {/each}

    {#if $filteredSessions.length === 0}
      <div class="empty">No sessions found</div>
    {/if}
  </div>
</div>

<style>
  .list-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .list-toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .tb-btn {
    padding: 2px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-tertiary);
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    line-height: 1;
  }
  .tb-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-secondary);
  }

  .group-filter-wrap {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .group-filter {
    width: 100%;
    padding: 3px 24px 3px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    outline: none;
  }
  .group-filter:focus { border-color: var(--accent); }
  .group-filter::placeholder { color: var(--text-muted); }

  .group-filter-clear {
    position: absolute;
    right: 4px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    padding: 0 3px;
    line-height: 1;
  }
  .group-filter-clear:hover { color: var(--text-primary); }

  .list {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .group {
    margin-bottom: 2px;
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    border: none;
    border-left: 2px solid transparent;
    border-radius: var(--radius);
    background: none;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    cursor: pointer;
    transition: background 0.1s;
    text-align: left;
  }
  .group-header:hover {
    background: var(--bg-secondary);
    color: var(--text-secondary);
  }
  .group-header.selected {
    background: var(--bg-tertiary);
    border-left-color: var(--accent);
    color: var(--text-primary);
  }

  .group-header.collapsed-header {
    opacity: 0.7;
  }
  .group-header.collapsed-header:hover {
    opacity: 1;
  }

  .chevron {
    color: inherit;
    font-size: 12px;
    line-height: 1;
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    transition: transform 0.15s;
  }

  .group-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .group-count {
    flex-shrink: 0;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 1px 5px;
    border-radius: 8px;
    font-size: 9px;
    min-width: 18px;
    text-align: center;
  }

  .group-open-btn {
    flex-shrink: 0;
    padding: 1px 6px;
    border: 1px solid transparent;
    border-radius: var(--radius);
    background: none;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s, background 0.15s, border-color 0.15s;
    white-space: nowrap;
  }
  .group-header:hover .group-open-btn {
    opacity: 1;
  }
  .group-header.selected .group-open-btn {
    opacity: 1;
    border-color: var(--accent);
  }
  .group-open-btn:hover {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 20px;
    font-size: var(--font-size-base);
  }
</style>
