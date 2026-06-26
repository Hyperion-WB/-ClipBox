<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    label: string;
    hint?: string;
    control: Snippet;
  }

  let { label, hint, control }: Props = $props();
</script>

<div class="row">
  <div class="row-text">
    <span class="row-label">{label}</span>
    {#if hint}
      <span class="row-hint">{hint}</span>
    {/if}
  </div>
  <div class="row-control">
    {@render control()}
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }

  .row:last-child {
    border-bottom: none;
  }

  .row-text {
    flex: 1;
    min-width: 0;
  }

  .row-label {
    display: block;
    font-size: 13px;
    color: var(--text);
    line-height: 1.35;
  }

  .row-hint {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 2px;
    line-height: 1.35;
  }

  .row-control {
    flex-shrink: 0;
  }
</style>
