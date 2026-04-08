<script lang="ts">
  import type { SessionSummary } from '$lib/types/session';

  let {
    session,
    x,
    y,
    onclose,
    onresume,
    ondelete,
    onrename,
    onpreview,
    oncopyid,
    onopenfiles,
  }: {
    session: SessionSummary;
    x: number;
    y: number;
    onclose: () => void;
    onresume: (session: SessionSummary) => void;
    ondelete: (session: SessionSummary) => void;
    onrename: (session: SessionSummary) => void;
    onpreview: (session: SessionSummary) => void;
    oncopyid: (session: SessionSummary) => void;
    onopenfiles: (session: SessionSummary) => void;
  } = $props();

  // Clamp menu position so it doesn't overflow the viewport
  const menuWidth = 180;
  const menuHeight = 200;
  const clampedX = $derived(Math.min(x, window.innerWidth - menuWidth));
  const clampedY = $derived(Math.min(y, window.innerHeight - menuHeight));

  const isReadOnly = $derived(session.source === 'vscode-copilot');
  const isDeleted = $derived(!session.exists_on_disk);

  function handleAction(action: () => void) {
    action();
    onclose();
  }
</script>

<!-- Backdrop to capture outside clicks -->
<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="backdrop" onclick={onclose} oncontextmenu={(e: MouseEvent) => { e.preventDefault(); onclose(); }}>
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div
    class="context-menu"
    style="left: {clampedX}px; top: {clampedY}px;"
    onclick={(e: MouseEvent) => e.stopPropagation()}
  >
    <button class="menu-item" onclick={() => handleAction(() => onpreview(session))}>
      <span class="icon">👁</span> Preview
    </button>

    <button class="menu-item" onclick={() => handleAction(() => oncopyid(session))}>
      <span class="icon">📋</span> Copy ID
    </button>

    <button
      class="menu-item"
      onclick={() => handleAction(() => onopenfiles(session))}
      disabled={!session.cwd || isDeleted}
      title={!session.cwd ? 'No folder path available' : ''}
    >
      <span class="icon">📂</span> Open Folder
    </button>

    <button
      class="menu-item"
      onclick={() => handleAction(() => onresume(session))}
      disabled={isDeleted}
    >
      <span class="icon">▶</span> Resume
    </button>

    <div class="divider"></div>

    <button
      class="menu-item"
      onclick={() => handleAction(() => onrename(session))}
      disabled={isReadOnly || isDeleted}
      title={isReadOnly ? 'Rename not supported for VS Code sessions' : ''}
    >
      <span class="icon">✏️</span> Rename
    </button>

    <button
      class="menu-item danger"
      onclick={() => handleAction(() => ondelete(session))}
      disabled={isReadOnly || isDeleted}
      title={isReadOnly ? 'Delete not supported for VS Code sessions' : ''}
    >
      <span class="icon">🗑</span> Delete
    </button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
  }

  .context-menu {
    position: fixed;
    z-index: 1001;
    min-width: 160px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 4px 0;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .menu-item {
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
  .menu-item:hover:not(:disabled) { background: var(--bg-secondary); }
  .menu-item:disabled {
    color: var(--text-muted);
    cursor: not-allowed;
    opacity: 0.5;
  }
  .menu-item.danger:not(:disabled) { color: var(--accent-red); }
  .menu-item.danger:hover:not(:disabled) {
    background: rgba(248, 81, 73, 0.1);
  }

  .icon { font-size: 12px; width: 16px; text-align: center; }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
</style>
