<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    value?: string;
    options: Option[];
    onchange?: (value: string) => void;
  }

  let { value = $bindable(""), options, onchange }: Props = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement | null>(null);

  const selectedLabel = $derived(
    options.find((o) => o.value === value)?.label ?? value,
  );

  function pick(v: string) {
    value = v;
    open = false;
    onchange?.(v);
  }

  function onWindowClick(e: MouseEvent) {
    if (!open || !root?.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window onclick={onWindowClick} />

<div class="select-wrap" bind:this={root}>
  <button type="button" class="select-trigger" onclick={() => (open = !open)}>
    <span class="select-value">{selectedLabel}</span>
    <span class="chevron" class:open></span>
  </button>
  {#if open}
    <ul class="select-menu" role="listbox">
      {#each options as opt (opt.value)}
        <li>
          <button
            type="button"
            class="select-option"
            class:selected={opt.value === value}
            onclick={() => pick(opt.value)}
          >
            {opt.label}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .select-wrap {
    position: relative;
    width: 100%;
  }

  .select-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 10px;
    font-size: 13px;
    background: var(--bg);
    color: var(--text);
    cursor: pointer;
    text-align: left;
    box-sizing: border-box;
  }

  .select-trigger:hover {
    background: var(--hover);
  }

  .select-value {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chevron {
    width: 8px;
    height: 8px;
    border-right: 2px solid var(--text-muted);
    border-bottom: 2px solid var(--text-muted);
    transform: rotate(45deg);
    transition: transform 0.15s ease;
    flex-shrink: 0;
    margin-top: -2px;
  }

  .chevron.open {
    transform: rotate(-135deg);
    margin-top: 2px;
  }

  .select-menu {
    position: absolute;
    left: 0;
    right: 0;
    top: calc(100% + 4px);
    margin: 0;
    padding: 4px;
    list-style: none;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.14);
    z-index: 50;
    max-height: 200px;
    overflow-y: auto;
  }

  .select-option {
    width: 100%;
    border: none;
    background: transparent;
    text-align: left;
    padding: 8px 10px;
    font-size: 13px;
    color: var(--text);
    border-radius: 6px;
    cursor: pointer;
  }

  .select-option:hover,
  .select-option.selected {
    background: var(--hover);
    color: var(--accent);
  }
</style>
