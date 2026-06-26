<script lang="ts">
  interface Props {
    title: string;
    defaultOpen?: boolean;
    children: import("svelte").Snippet;
  }

  let { title, defaultOpen = false, children }: Props = $props();
  let open = $state(defaultOpen);
</script>

<div class="section">
  <button type="button" class="section-header" onclick={() => (open = !open)}>
    <span class="chevron" class:open></span>
    <span class="title">{title}</span>
  </button>
  {#if open}
    <div class="section-body">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .section {
    margin: 0 12px 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
  }

  .section-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    text-align: left;
  }

  .section-header:hover {
    background: var(--hover);
  }

  .chevron {
    width: 8px;
    height: 8px;
    border-right: 2px solid var(--text-muted);
    border-bottom: 2px solid var(--text-muted);
    transform: rotate(-45deg);
    transition: transform 0.2s ease;
    flex-shrink: 0;
  }

  .chevron.open {
    transform: rotate(45deg);
  }

  .title {
    flex: 1;
  }

  .section-body {
    padding: 0 14px 12px;
    border-top: 1px solid var(--border);
  }
</style>
