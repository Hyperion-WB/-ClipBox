<script lang="ts">

  import type { ClipItem } from "$lib/types";

  import { t } from "$lib/i18n.svelte";

  import { analyzeSensitive, maskForDisplay } from "$lib/sensitiveMask";

  import ClipThumb from "./ClipThumb.svelte";



  interface Props {

    item: ClipItem;

    selected: boolean;

    showPin?: boolean;

    maskSensitive?: boolean;

    onSelect: () => void;

    onSaveImage?: () => void;

    onContextMenu?: (e: MouseEvent) => void;

  }



  let {

    item,

    selected,

    showPin = false,

    maskSensitive = true,

    onSelect,

    onSaveImage,

    onContextMenu,

  }: Props = $props();



  function preview(text: string): string {

    const one = text.replace(/\s+/g, " ").trim();

    return one.length > 120 ? one.slice(0, 120) + "…" : one;

  }

  const displayText = $derived.by(() => {
    if (item.content_type === "image" || item.content_type === "file") {
      return preview(item.content_text);
    }
    const masked = maskSensitive && analyzeSensitive(item.content_text).sensitive;
    return preview(masked ? maskForDisplay(item.content_text, true) : item.content_text);
  });

</script>



<button

  type="button"

  class="card"

  class:selected

  onclick={onSelect}

  oncontextmenu={onContextMenu}

>

  <div class="card-body">

    {#if item.content_type === "image" && item.has_thumbnail}

      <ClipThumb clipId={item.id} />

    {/if}

    <span class="text">{displayText}</span>

  </div>

  <div class="card-actions">

    {#if item.content_type === "image" && onSaveImage}

      <span

        class="save-btn"

        role="button"

        tabindex="-1"

        title={t("menu.saveImageTitle")}

        onclick={(e) => { e.stopPropagation(); onSaveImage(); }}

      >↓</span>

    {/if}

    {#if showPin && item.pinned}

      <span class="pin" title={t("clip.pinned")}>📌</span>

    {/if}

  </div>

</button>



<style>

  .card {

    width: 100%;

    display: block;

    text-align: left;

    border: 1px solid var(--border);

    border-radius: 10px;

    background: var(--bg);

    padding: 10px 12px;

    margin-bottom: 6px;

    cursor: pointer;

    position: relative;

    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);

    transition: border-color 0.15s, box-shadow 0.15s;

  }



  .card:hover, .card.selected {

    border-color: var(--accent);

    box-shadow: 0 2px 8px rgba(0, 129, 255, 0.12);

  }



  .card-body {

    display: flex;

    align-items: flex-start;

    gap: 10px;

    min-width: 0;

    padding-right: 24px;

  }



  .text {

    font-size: 13px;

    line-height: 1.45;

    color: var(--text);

    word-break: break-word;

    flex: 1;

  }



  .card-actions {

    position: absolute;

    right: 8px;

    bottom: 6px;

    display: flex;

    gap: 4px;

    align-items: center;

  }



  .save-btn {

    font-size: 12px;

    color: var(--text-muted);

    padding: 2px 4px;

    border-radius: 4px;

    cursor: pointer;

  }



  .save-btn:hover {

    background: var(--hover);

    color: var(--accent);

  }



  .pin {

    font-size: 11px;

    opacity: 0.7;

  }

</style>

