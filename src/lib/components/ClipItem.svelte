<script lang="ts">

  import type { ClipItem } from "$lib/types";

  import { t } from "$lib/i18n.svelte";

  import ClipThumb from "./ClipThumb.svelte";



  interface Props {

    item: ClipItem;

    index: number;

    selected: boolean;

    checked: boolean;

    multiSelectMode: boolean;

    onSelect: (e: MouseEvent) => void;

    onContextMenu: (e: MouseEvent) => void;

    onPin: () => void;

    onDelete: () => void;

    onPastePlain: () => void;

    onSaveImage?: () => void;

  }



  let {

    item,

    index,

    selected,

    checked,

    multiSelectMode,

    onSelect,

    onContextMenu,

    onPin,

    onDelete,

    onPastePlain,

    onSaveImage,

  }: Props = $props();



  function preview(text: string): string {

    const oneLine = text.replace(/\s+/g, " ").trim();

    return oneLine.length > 72 ? oneLine.slice(0, 72) + "…" : oneLine;

  }



  function typeLabel(type: ClipItem["content_type"]): string {

    switch (type) {

      case "image": return t("clip.typeImage");

      case "html": return "H";

      case "file": return "F";

      default: return "T";

    }

  }



  const hotkeyNum = $derived(index < 9 ? index + 1 : null);

</script>



<div

  class="clip-item"

  class:selected

  class:checked

  class:pinned={item.pinned}

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

      {#if item.source_app}

        <span class="source">{item.source_app}</span>

      {/if}

    </div>

  </div>



  <div class="actions">

    {#if item.content_type === "image" && onSaveImage}

      <button class="action-btn" title={t("menu.saveImageTitle")} onclick={(e) => { e.stopPropagation(); onSaveImage(); }}>↓</button>

    {/if}

    {#if item.content_type === "html"}

      <button class="action-btn" title={t("menu.pastePlain")} onclick={(e) => { e.stopPropagation(); onPastePlain(); }}>¶</button>

    {/if}

    <button class="action-btn" class:active={item.pinned} title={item.pinned ? t("menu.unpin") : t("menu.pin")} onclick={(e) => { e.stopPropagation(); onPin(); }}>📌</button>

    <button class="action-btn delete" title={t("menu.delete")} onclick={(e) => { e.stopPropagation(); onDelete(); }}>×</button>

  </div>

</div>



<style>

  .clip-item {

    display: flex;

    align-items: center;

    gap: 8px;

    height: 44px;

    padding: 0 10px;

    cursor: pointer;

    border-radius: 8px;

    margin: 0 6px;

    contain: layout style paint;

  }



  .clip-item:hover, .clip-item.selected, .clip-item.checked {

    background: var(--hover);

  }



  .clip-item.pinned {

    border-left: 2px solid var(--accent);

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

  }



  .hotkey-num.empty { background: transparent; }



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

  }



  .type-badge[data-type="image"] { background: #e8f4fd; color: #1976d2; }

  .type-badge[data-type="html"] { background: #fff3e0; color: #f57c00; }

  .type-badge[data-type="file"] { background: #f3e5f5; color: #7b1fa2; }



  .content {

    flex: 1;

    min-width: 0;

    display: flex;

    align-items: center;

    gap: 8px;

  }



  .text-block { min-width: 0; flex: 1; }



  .text {

    font-size: 13px;

    color: var(--text);

    white-space: nowrap;

    overflow: hidden;

    text-overflow: ellipsis;

    display: block;

  }



  .source {

    font-size: 10px;

    color: var(--text-muted);

    display: block;

    white-space: nowrap;

    overflow: hidden;

    text-overflow: ellipsis;

  }



  .actions {

    display: flex;

    gap: 2px;

    opacity: 0;

  }



  .clip-item:hover .actions, .clip-item.selected .actions {

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



  .action-btn:hover { background: var(--hover-strong); color: var(--text); }

  .action-btn.active { color: var(--accent); }

  .action-btn.delete:hover { color: #e53935; }



  input[type="checkbox"] {

    width: 14px;

    height: 14px;

    flex-shrink: 0;

    pointer-events: none;

  }

</style>

