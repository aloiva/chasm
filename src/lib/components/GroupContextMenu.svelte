<script lang="ts">
  let {
    groupKey,
    x,
    y,
    onclose,
    onview,
  }: {
    groupKey: string;
    x: number;
    y: number;
    onclose: () => void;
    onview: (key: string) => void;
  } = $props();

  const menuWidth = 160;
  const menuHeight = 40;
  const clampedX = $derived(Math.min(x, window.innerWidth - menuWidth));
  const clampedY = $derived(Math.min(y, window.innerHeight - menuHeight));

  function handleAction(action: () => void) {
    action();
    onclose();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="backdrop" onclick={onclose} oncontextmenu={(e: MouseEvent) => { e.preventDefault(); onclose(); }}>
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div
    class="context-menu"
    style="left: {clampedX}px; top: {clampedY}px;"
    onclick={(e: MouseEvent) => e.stopPropagation()}
  >
    <button class="menu-item" onclick={() => handleAction(() => onview(groupKey))}>
      <span class="icon">👁</span> View
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
    min-width: 140px;
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
  .menu-item:hover { background: var(--bg-secondary); }

  .icon { width: 16px; text-align: center; }
</style>
