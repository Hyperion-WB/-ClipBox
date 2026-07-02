<script lang="ts">
  import { dragWindow } from "$lib/window";
  import { t } from "$lib/i18n.svelte";
  import type { ClipItem, Snippet } from "$lib/types";
  import CollapsibleBlock from "./CollapsibleBlock.svelte";
  import MinimalCard from "./MinimalCard.svelte";

  interface Props {
    snippets: Snippet[];
    pinnedClips: ClipItem[];
    historyClips: ClipItem[];
    pinnedCollapseThreshold: number;
    selectedClipId: number | null;
    onPasteClip: (item: ClipItem) => void;
    onPasteSnippet: (snippet: Snippet) => void;
    onClearHistory: () => void;
    onOpenSettings: () => void;
    onClipContextMenu?: (item: ClipItem, e: MouseEvent) => void;
    onSaveImage?: (item: ClipItem) => void;
    maskSensitive?: boolean;
  }

  let {
    snippets,
    pinnedClips,
    historyClips,
    pinnedCollapseThreshold,
    selectedClipId,
    onPasteClip,
    onPasteSnippet,
    onClearHistory,
    onOpenSettings,
    onClipContextMenu,
    onSaveImage,
    maskSensitive = true,
  }: Props = $props();

  let snippetsCollapsed = $state(true);
  let pinnedCollapsed = $state(false);

  $effect(() => {
    pinnedCollapsed = pinnedClips.length > pinnedCollapseThreshold;
  });

  const snippetsHint = $derived(
    snippets.length > 0 ? snippets[0].title : undefined,
  );
</script>

<div class="minimal">
  <div class="minimal-toolbar">
    <span class="toolbar-title" onmousedown={dragWindow} role="presentation">{t("history.clipboard")}</span>
    <div class="toolbar-actions">
      <button type="button" class="tool-btn" onclick={onClearHistory}>{t("history.clearAll")}</button>
      <button type="button" class="tool-btn icon" title={t("tabs.settings")} onclick={onOpenSettings}>⚙</button>
    </div>
  </div>

  <div class="minimal-scroll">
    {#if snippets.length > 0}
      <CollapsibleBlock
        title={t("history.snippets")}
        count={snippets.length}
        collapsed={snippetsCollapsed}
        onToggle={() => (snippetsCollapsed = !snippetsCollapsed)}
        hint={snippetsHint}
      >
        {#each snippets as snippet (snippet.id)}
          <button
            type="button"
            class="snippet-card"
            onclick={() => onPasteSnippet(snippet)}
          >
            <span class="snippet-title">{snippet.title}</span>
            <span class="snippet-content">{snippet.content}</span>
          </button>
        {/each}
      </CollapsibleBlock>
    {/if}

    {#if pinnedClips.length > 0}
      <CollapsibleBlock
        title={t("history.pinned")}
        count={pinnedClips.length}
        collapsed={pinnedCollapsed}
        onToggle={() => (pinnedCollapsed = !pinnedCollapsed)}
        hint={pinnedClips.length > pinnedCollapseThreshold ? t("history.pinnedFolded", { n: pinnedCollapseThreshold }) : undefined}
      >
        {#each pinnedClips as item (item.id)}
          <MinimalCard
            {item}
            {maskSensitive}
            selected={selectedClipId === item.id}
            showPin
            onSelect={() => onPasteClip(item)}
            onSaveImage={onSaveImage ? () => onSaveImage(item) : undefined}
            onContextMenu={(e) => onClipContextMenu?.(item, e)}
          />
        {/each}
      </CollapsibleBlock>
    {/if}

    {#if historyClips.length > 0}
      <div class="history-section">
        {#if snippets.length > 0 || pinnedClips.length > 0}
          <div class="section-label">{t("history.recent")}</div>
        {/if}
        {#each historyClips as item (item.id)}
          <MinimalCard
            {item}
            {maskSensitive}
            selected={selectedClipId === item.id}
            onSelect={() => onPasteClip(item)}
            onSaveImage={onSaveImage ? () => onSaveImage(item) : undefined}
            onContextMenu={(e) => onClipContextMenu?.(item, e)}
          />
        {/each}
      </div>
    {:else if snippets.length === 0 && pinnedClips.length === 0}
      <div class="empty">{t("history.empty")}</div>
    {/if}
  </div>
</div>

<style>
  .minimal {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .minimal-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
  }

  .toolbar-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    cursor: grab;
    user-select: none;
  }

  .toolbar-title:active {
    cursor: grabbing;
  }

  .toolbar-actions {
    display: flex;
    gap: 6px;
  }

  .tool-btn {
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
  }

  .tool-btn:hover {
    background: var(--hover);
  }

  .tool-btn.icon {
    padding: 4px 8px;
    font-size: 14px;
  }

  .minimal-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0 10px;
    min-height: 0;
  }

  .history-section {
    padding: 0 10px;
  }

  .section-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    padding: 4px 4px 8px;
  }

  .snippet-card {
    width: 100%;
    display: block;
    text-align: left;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--bg);
    padding: 10px 12px;
    margin-bottom: 6px;
    cursor: pointer;
  }

  .snippet-card:hover {
    border-color: var(--accent);
  }

  .snippet-title {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
    margin-bottom: 4px;
  }

  .snippet-content {
    display: block;
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .empty {
    padding: 40px 20px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>
