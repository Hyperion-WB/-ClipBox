<script lang="ts">
  import { t } from "$lib/i18n.svelte";
  interface Props {
    activeTags: string[];
    onChange: (tags: string[]) => void;
  }

  let { activeTags, onChange }: Props = $props();

  const TAGS = [
    { id: "#code", label: "#code" },
    { id: "#url", label: "#url" },
    { id: "#img", label: "#img" },
  ] as const;

  function toggle(tag: string) {
    const set = new Set(activeTags);
    if (set.has(tag)) set.delete(tag);
    else set.add(tag);
    onChange([...set]);
  }
</script>

<div class="tag-filter">
  {#each TAGS as tag}
    <button
      type="button"
      class="tag"
      class:active={activeTags.includes(tag.id)}
      onclick={() => toggle(tag.id)}
    >
      {tag.label}
    </button>
  {/each}
  <span class="hint">{t("tag.hint")}</span>
</div>

<style>
  .tag-filter {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px 6px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .tag {
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 999px;
    cursor: pointer;
  }

  .tag.active {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .hint {
    font-size: 10px;
    color: var(--text-muted);
    margin-left: auto;
  }
</style>
