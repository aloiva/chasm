<script lang="ts">
  import type { SessionSummary } from '$lib/types/session';
  import SessionCard from './SessionCard.svelte';
  import { groupedSessions, filteredSessions, viewMode, sources } from '$lib/stores/sessions';

  let { oncontextmenu }: {
    oncontextmenu?: (e: MouseEvent, session: SessionSummary) => void;
  } = $props();

  function resolveHeader(key: string): string {
    if ($viewMode === 'folder' || $viewMode === 'branch' || $viewMode === 'date') {
      return key;
    }
    // Resolve source name from sources store
    const info = $sources.find(s => s.name === key);
    return info?.display_name?.toUpperCase() ?? key.toUpperCase();
  }
</script>

<div class="list">
  {#each Object.entries($groupedSessions) as [key, sessions]}
    <div class="source-header">
      {resolveHeader(key)} · {sessions.length} session{sessions.length !== 1 ? 's' : ''}
    </div>
    {#each sessions as session (session.id)}
      <SessionCard {session} {oncontextmenu} />
    {/each}
  {/each}

  {#if $filteredSessions.length === 0}
    <div class="empty">No sessions found</div>
  {/if}
</div>

<style>
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }
  .source-header {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--text-muted);
    padding: 10px 12px 4px;
  }
  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 20px;
    font-size: var(--font-size-base);
  }
</style>
