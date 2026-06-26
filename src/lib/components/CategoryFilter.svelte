<script lang="ts">
  import type { ClipCategory } from "$lib/types";
  import { t } from "$lib/i18n.svelte";

  interface Props {
    value: ClipCategory;
    onChange: (category: ClipCategory) => void;
  }

  let { value, onChange }: Props = $props();

  const items = $derived([
    { id: "all" as ClipCategory, label: t("category.all"), shortcut: "Alt+1" },
    { id: "text" as ClipCategory, label: t("category.text"), shortcut: "Alt+2" },
    { id: "image" as ClipCategory, label: t("category.image"), shortcut: "Alt+3" },
    { id: "file" as ClipCategory, label: t("category.file"), shortcut: "Alt+4" },
  ]);
</script>

<div class="categories">
  {#each items as item (item.id)}
    <button
      type="button"
      class:active={value === item.id}
      title={item.shortcut}
      onclick={() => onChange(item.id)}
    >
      {item.label}
    </button>
  {/each}
</div>

<style>
  .categories {
    display: flex;
    gap: 6px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .categories button {
    flex: 1;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    border-radius: 6px;
    padding: 4px 0;
    font-size: 12px;
    cursor: pointer;
  }

  .categories button.active {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .categories button:hover:not(.active) {
    background: var(--hover);
  }
</style>
