<script lang="ts">
  import type { SessionSummary } from '$lib/types/session';
  import SourceBadge from './SourceBadge.svelte';
  import { formatDate, formatDateRange, truncate } from '$lib/utils/format';
  import { selectedSessionId } from '$lib/stores/sessions';

  let { session, oncontextmenu }: {
    session: SessionSummary;
    oncontextmenu?: (e: MouseEvent, session: SessionSummary) => void;
  } = $props();

  const isActive = $derived($selectedSessionId === session.id);
  const displayTitle = $derived(
    session.title ?? truncate(session.first_message, 60) ?? '(unnamed session)'
  );

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    oncontextmenu?.(e, session);
  }
</script>

<button
  class="card"
  class:active={isActive}
  class:deleted={!session.exists_on_disk}
  onclick={() => selectedSessionId.set(session.id + ':' + session.source)}
  oncontextmenu={handleContextMenu}
>
  <SourceBadge source={session.source} />
  <div class="body">
    <div class="title">
      {#if session.status === 'recent'}
        <span class="status-dot" title="Recently active"></span>
      {/if}
      {displayTitle}
    </div>
    <div class="meta">
      {#if session.cwd}
        <span class="cwd">{session.cwd}</span>
      {/if}
      {#if session.branch}
        <span class="branch">· {session.branch}</span>
      {/if}
    </div>
    {#if session.first_message && session.title}
      <div class="preview">"{truncate(session.first_message, 80)}"</div>
    {/if}
  </div>
  <div class="right">
    <div class="turns">
      {session.turn_count} turn{session.turn_count !== 1 ? 's' : ''}
      {#if session.has_checkpoints}
        <span class="ckpt">· ckpt</span>
      {/if}
    </div>
    <div class="date">{formatDateRange(session.created_at, session.updated_at)}</div>
    {#if !session.exists_on_disk}
      <div class="deleted-badge">deleted</div>
    {/if}
  </div>
</button>

<style>
  .card {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: var(--card-padding, 8px 12px);
    border-radius: var(--radius);
    cursor: pointer;
    transition: background 0.1s;
    border: none;
    border-left: 2px solid transparent;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    width: 100%;
  }
  .card:hover { background: var(--bg-secondary); }
  .card.active { background: var(--bg-tertiary); border-left-color: var(--accent); }
  .card.deleted { opacity: 0.5; }

  .body { flex: 1; min-width: 0; }
  .title {
    font-size: var(--font-size-title);
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent-green);
    flex-shrink: 0;
    box-shadow: 0 0 4px var(--accent-green);
  }
  .meta {
    font-size: var(--font-size-small);
    color: var(--text-muted);
    margin-top: 2px;
    font-family: var(--font-ui);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .preview {
    font-size: var(--font-size-small);
    color: var(--text-secondary);
    margin-top: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .right { text-align: right; flex-shrink: 0; }
  .turns { font-size: var(--font-size-small); color: var(--text-muted); }
  .ckpt { color: var(--accent-yellow); }
  .date { font-size: 10px; color: var(--border); margin-top: 2px; }
  .deleted-badge {
    font-size: 9px;
    color: var(--accent-red);
    background: rgba(248, 81, 73, 0.1);
    padding: 1px 4px;
    border-radius: 3px;
    margin-top: 2px;
    display: inline-block;
  }
</style>
