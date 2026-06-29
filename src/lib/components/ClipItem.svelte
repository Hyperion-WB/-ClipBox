<script lang="ts">
  import type { Highlight } from "$lib/extractHighlights";
  import type { ClipItem } from "$lib/types";
  import { itemHasFormatAction } from "$lib/formatContent";
  import { t } from "$lib/i18n.svelte";
  import ClipHighlights from "./ClipHighlights.svelte";
  import ClipThumb from "./ClipThumb.svelte";
  import SourceBadge from "./SourceBadge.svelte";

  interface Props {
    item: ClipItem;
    index: number;
    selected: boolean;
    checked: boolean;
    multiSelectMode: boolean;
    rowHeight?: number;
    highlights?: Highlight[];
    onSelect: (e: MouseEvent) => void;
    onContextMenu: (e: MouseEvent) => void;
    onPin: () => void;
    onDelete: () => void;
    onPastePlain: () => void;
    onPasteSegment?: (value: string) => void;
    onOpenPath?: (value: string) => void;
    onOpenUrl?: (value: string) => void;
    onSaveImage?: () => void;
    onFormatPaste?: () => void;
  }

  let {
    item,
    index,
    selected,
    checked,
    multiSelectMode,
    rowHeight = 44,
    highlights = [],
    onSelect,
    onContextMenu,
    onPin,
    onDelete,
    onPastePlain,
    onPasteSegment,
    onOpenPath,
    onOpenUrl,
    onSaveImage,
    onFormatPaste,
  }: Props = $props();

  const expanded = $derived(highlights.length > 0);
  const canFormat = $derived(
    item.content_type !== "image" &&
      item.content_type !== "file" &&
      itemHasFormatAction(item.content_text, item.content_type),
  );

  function preview(text: string): string {
    const oneLine = text.replace(/\s+/g, " ").trim();
    const max = expanded ? 120 : 100;
    return oneLine.length > max ? oneLine.slice(0, max) + "…" : oneLine;
  }

  function typeLabel(type: ClipItem["content_type"]): string {
    switch (type) {
      case "image":
        return t("clip.typeImage");
      case "html":
        return "H";
      case "file":
        return "F";
      default:
        return "T";
    }
  }

  const hotkeyNum = $derived(index < 9 ? index + 1 : null);
</script>

<div
  class="clip-item"
  class:selected
  class:checked
  class:pinned={item.pinned}
  class:expanded
  class:has-chips={expanded}
  style:min-height="{rowHeight}px"
  role="button"
  tabindex="-1"
  onclick={onSelect}
  oncontextmenu={onContextMenu}
>
  {#if multiSelectMode}
    <input type="checkbox" checked={checked} tabindex="-1" readonly />
  {:else if hotkeyNum !== null}
    <span class="hotkey-num">{hotkeyNum}</span>
  {:else}
    <span class="hotkey-num empty"></span>
  {/if}

  <div class="type-badge" data-type={item.content_type}>{typeLabel(item.content_type)}</div>

  <div class="content">
    {#if item.content_type === "image" && item.has_thumbnail}
      <ClipThumb clipId={item.id} />
    {/if}
    <div class="text-block">
      <span class="text">{preview(item.content_text)}</span>
      {#if expanded && onPasteSegment}
        <ClipHighlights
          items={highlights}
          onPaste={onPasteSegment}
          {onOpenPath}
          {onOpenUrl}
        />
      {:else if item.source_app}
        <SourceBadge name={item.source_app} />
      {/if}
    </div>
  </div>

  <div class="actions">
    {#if item.content_type === "file" && onOpenPath}
      <button
        class="action-btn"
        title={t("menu.openPath")}
        onclick={(e) => { e.stopPropagation(); onOpenPath(item.content_text); }}
      >↗</button>
    {/if}
    {#if item.content_type === "image" && onSaveImage}
      <button class="action-btn" title={t("menu.saveImageTitle")} onclick={(e) => { e.stopPropagation(); onSaveImage(); }}>↓</button>
    {/if}
    {#if item.content_type === "html"}
      <button class="action-btn" title={t("menu.pastePlain")} onclick={(e) => { e.stopPropagation(); onPastePlain(); }}>¶</button>
    {/if}
    {#if canFormat && onFormatPaste}
      <button
        class="action-btn format"
        title={t("menu.formatPaste")}
        onclick={(e) => { e.stopPropagation(); onFormatPaste(); }}
      >{"{ }"}</button>
    {/if}
    <button
      class="pin-btn"
      class:is-pinned={item.pinned}
      title={item.pinned ? t("menu.unpin") : t("menu.pin")}
      onclick={(e) => { e.stopPropagation(); onPin(); }}
    >
      <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true">
        <path
          d="M3.5 1.5h5v7.5L6 7.25 3.5 9V1.5z"
          fill={item.pinned ? "currentColor" : "none"}
          stroke="currentColor"
          stroke-width="1.1"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button class="action-btn delete" title={t("menu.delete")} onclick={(e) => { e.stopPropagation(); onDelete(); }}>×</button>
  </div>
</div>

<style>
  .clip-item {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    padding: 6px 4px 6px 8px;
    cursor: pointer;
    border-radius: 8px;
    margin: 0 4px;
    box-sizing: border-box;
  }

  .clip-item.has-chips {
    padding-bottom: 7px;
  }

  .clip-item:not(.expanded) {
    align-items: center;
  }

  .clip-item:hover,
  .clip-item.selected,
  .clip-item.checked {
    background: var(--hover);
  }

  .clip-item.pinned {
    background: color-mix(in srgb, var(--accent) 5%, transparent);
  }

  .clip-item.pinned:hover,
  .clip-item.pinned.selected {
    background: color-mix(in srgb, var(--accent) 10%, var(--hover));
  }

  .hotkey-num {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    background: var(--accent);
    color: white;
    font-size: 11px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .clip-item:not(.expanded) .hotkey-num {
    margin-top: 0;
  }

  .hotkey-num.empty {
    background: transparent;
  }

  .type-badge {
    width: 22px;
    height: 22px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 600;
    flex-shrink: 0;
    background: var(--badge-bg);
    color: var(--badge-text);
    margin-top: 1px;
  }

  .clip-item:not(.expanded) .type-badge {
    margin-top: 0;
  }

  .type-badge[data-type="image"] {
    background: #e8f4fd;
    color: #1976d2;
  }
  .type-badge[data-type="html"] {
    background: #fff3e0;
    color: #f57c00;
  }
  .type-badge[data-type="file"] {
    background: #f3e5f5;
    color: #7b1fa2;
  }

  .content {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .clip-item:not(.expanded) .content {
    align-items: center;
  }

  .text-block {
    min-width: 0;
    flex: 1;
    padding-right: 2px;
  }

  .text {
    font-size: 13px;
    color: var(--text);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.4;
    word-break: break-word;
  }

  .clip-item:not(.expanded) .text {
    -webkit-line-clamp: 1;
    line-clamp: 1;
    white-space: nowrap;
    display: block;
    text-overflow: ellipsis;
  }

  .clip-item.expanded .content {
    padding-right: 4px;
  }

  .pin-btn {
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .pin-btn:hover {
    background: var(--hover-strong);
    color: var(--text);
  }

  .pin-btn.is-pinned {
    color: var(--accent);
  }

  .actions {
    display: flex;
    gap: 1px;
    opacity: 0;
    flex-shrink: 0;
    align-self: flex-start;
    margin-top: 0;
    padding-left: 2px;
  }

  .clip-item:not(.expanded) .actions {
    margin-top: 0;
  }

  .clip-item:hover .actions,
  .clip-item.selected .actions {
    opacity: 1;
  }

  .action-btn {
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    color: var(--text-muted);
  }

  .action-btn:hover {
    background: var(--hover-strong);
    color: var(--text);
  }

  .action-btn.delete:hover {
    color: #e53935;
  }

  .action-btn.format {
    font-family: ui-monospace, monospace;
    font-size: 9px;
    letter-spacing: -0.5px;
    color: var(--accent);
  }

  input[type="checkbox"] {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    pointer-events: none;
    margin-top: 4px;
  }
</style>
