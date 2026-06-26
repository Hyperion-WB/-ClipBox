<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    count?: number;
    collapsed: boolean;
    onToggle: () => void;
    children: Snippet;
    hint?: string;
  }

  let { title, count, collapsed, onToggle, children, hint }: Props = $props();
</script>

<section class="block" class:collapsed>
  <button type="button" class="block-header" onclick={onToggle}>
    <span class="chevron" aria-hidden="true"></span>
    <span class="title">{title}</span>
    {#if count !== undefined}
      <span class="count">{count}</span>
    {/if}
    {#if hint && collapsed}
      <span class="hint">{hint}</span>
    {/if}
  </button>
  {#if !collapsed}
    <div class="block-body">
      {@render children()}
    </div>
  {/if}
</section>

<style>
  .block {
    margin: 0 10px 8px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .block-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--text);
    font-size: 13px;
    text-align: left;
  }

  .block-header:hover {
    background: var(--hover);
  }

  .chevron {
    width: 8px;
    height: 8px;
    border-right: 2px solid var(--text-muted);
    border-bottom: 2px solid var(--text-muted);
    transform: rotate(45deg);
    transition: transform 0.2s ease;
    flex-shrink: 0;
    margin-top: -2px;
  }

  .collapsed .chevron {
    transform: rotate(-45deg);
    margin-top: 2px;
  }

  .title {
    font-weight: 600;
    flex-shrink: 0;
  }

  .count {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--hover);
    padding: 1px 7px;
    border-radius: 999px;
  }

  .hint {
    flex: 1;
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: right;
  }

  .block-body {
    padding: 0 8px 8px;
  }
</style>
