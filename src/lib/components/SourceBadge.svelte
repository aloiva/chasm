<script lang="ts">
  let { source, size = 'sm' }: { source: string; size?: 'sm' | 'md' } = $props();

  const badges: Record<string, { label: string; cls: string }> = {
    'copilot-cli': { label: 'CLI', cls: 'badge-cli' },
    'vscode-copilot': { label: 'VSC', cls: 'badge-vsc' },
  };

  const info = $derived(badges[source] ?? { label: source.slice(0, 3).toUpperCase(), cls: 'badge-default' });
</script>

<span class="badge {info.cls} size-{size}">{info.label}</span>

<style>
  .badge {
    display: inline-block;
    font-family: var(--font-mono);
    font-weight: 600;
    border-radius: 4px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .size-sm { font-size: 10px; padding: 2px 6px; }
  .size-md { font-size: 11px; padding: 3px 8px; }
  .badge-cli { background: rgba(56, 139, 253, 0.15); color: var(--accent); }
  .badge-vsc { background: rgba(63, 185, 80, 0.15); color: var(--accent-green); }
  .badge-default { background: var(--bg-tertiary); color: var(--text-muted); }
</style>
