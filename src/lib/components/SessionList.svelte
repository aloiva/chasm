<script lang="ts">
  import SessionCard from './SessionCard.svelte';
  import { groupedSessions, filteredSessions } from '$lib/stores/sessions';

  const sourceNames: Record<string, string> = {
    'copilot-cli': 'COPILOT CLI',
    'vscode-copilot': 'VS CODE COPILOT',
  };
</script>

<div class="list">
  {#each Object.entries($groupedSessions) as [source, sessions]}
    <div class="source-header">
      {sourceNames[source] ?? source.toUpperCase()} · {sessions.length} session{sessions.length !== 1 ? 's' : ''}
    </div>
    {#each sessions as session (session.id)}
      <SessionCard {session} />
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
