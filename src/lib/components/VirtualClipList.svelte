<script lang="ts">
  import type { ListLayout } from "$lib/grouping";
  import { HEADER_HEIGHT } from "$lib/grouping";
  import { t } from "$lib/i18n.svelte";
  import ClipItemRow from "./ClipItem.svelte";

  interface Props {
    layout: ListLayout;
    selectedRowIndex: number;
    selectedIds: Set<number>;
    multiSelectMode: boolean;
    onSaveImage?: (id: number) => void;
    onSelect: (rowIndex: number, e: MouseEvent) => void;
    onContextMenu: (rowIndex: number, e: MouseEvent) => void;
    onPin: (id: number) => void;
    onDelete: (id: number) => void;
    onPastePlain: (clipIndex: number) => void;
    onPasteSegment: (value: string) => void;
    onFormatPaste: (clipIndex: number) => void;
    onOpenPath: (path: string) => void;
    onOpenUrl: (url: string) => void;
    panelOpen?: boolean;
  }

  let {
    layout,
    selectedRowIndex,
    selectedIds,
    multiSelectMode,
    onSaveImage,
    onSelect,
    onContextMenu,
    onPin,
    onDelete,
    onPastePlain,
    onPasteSegment,
    onFormatPaste,
    onOpenPath,
    onOpenUrl,
    panelOpen = false,
  }: Props = $props();

  let container = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(320);
  let rafId = 0;
  let wasPanelOpen = false;

  function measureViewport() {
    const el = container;
    if (!el) return;
    const h = el.clientHeight;
    if (h > 0) viewportHeight = h;
  }

  $effect(() => {
    const el = container;
    if (!el) return;

    measureViewport();
    const ro = new ResizeObserver(() => measureViewport());
    ro.observe(el);

    const onScrollPassive = () => {
      if (rafId) return;
      rafId = requestAnimationFrame(() => {
        scrollTop = el.scrollTop;
        rafId = 0;
      });
    };
    el.addEventListener("scroll", onScrollPassive, { passive: true });

    return () => {
      ro.disconnect();
      el.removeEventListener("scroll", onScrollPassive);
    };
  });

  // Only reset scroll when the panel transitions closed → open.
  $effect(() => {
    const el = container;
    const opening = panelOpen && !wasPanelOpen;
    wasPanelOpen = panelOpen;
    if (!opening || !el) return;
    el.scrollTop = 0;
    scrollTop = 0;
    queueMicrotask(measureViewport);
  });

  function findVisibleRange(): { start: number; end: number } {
    const { offsets, heights, rows } = layout;
    if (rows.length === 0) return { start: 0, end: 0 };

    const viewH = Math.max(viewportHeight, 200);
    const top = scrollTop;
    const bottom = top + viewH + 120;

    let start = 0;
    for (let i = 0; i < rows.length; i++) {
      if (offsets[i] + heights[i] > top - 80) {
        start = Math.max(0, i - 2);
        break;
      }
    }

    let end = rows.length;
    for (let i = start; i < rows.length; i++) {
      if (offsets[i] >= bottom) {
        end = Math.min(rows.length, i + 2);
        break;
      }
    }

    if (end <= start) {
      end = Math.min(rows.length, start + Math.max(8, Math.ceil(viewH / 48) + 2));
    }

    return { start, end };
  }

  const visible = $derived(findVisibleRange());
  const offsetY = $derived(layout.offsets[visible.start] ?? 0);
</script>

{#if layout.rows.length === 0}
  <div class="empty">{t("history.empty")}</div>
{:else}
  <div class="virtual-list" bind:this={container} bind:clientHeight={viewportHeight}>
    <div class="spacer" style="height: {layout.totalHeight}px">
      <div class="window" style="transform: translate3d(0, {offsetY}px, 0)">
        {#each layout.rows.slice(visible.start, visible.end) as row, i (row.kind === "header" ? row.id : row.id)}
          {@const rowIndex = visible.start + i}
          {#if row.kind === "header"}
            <div class="section-header" style="height: {HEADER_HEIGHT}px">{row.title}</div>
          {:else}
            <ClipItemRow
              item={row.item}
              index={row.clipIndex}
              rowHeight={row.rowHeight}
              highlights={row.highlights}
              selected={selectedRowIndex === rowIndex}
              checked={selectedIds.has(row.item.id)}
              {multiSelectMode}
              onSaveImage={row.item.content_type === "image" && onSaveImage
                ? () => onSaveImage(row.item.id)
                : undefined}
              onSelect={(e) => onSelect(rowIndex, e)}
              onContextMenu={(e) => onContextMenu(rowIndex, e)}
              onPin={() => onPin(row.item.id)}
              onDelete={() => onDelete(row.item.id)}
              onPastePlain={() => onPastePlain(row.clipIndex)}
              onPasteSegment={onPasteSegment}
              onFormatPaste={() => onFormatPaste(row.clipIndex)}
              {onOpenPath}
              {onOpenUrl}
            />
          {/if}
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .virtual-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
  }

  .spacer { position: relative; width: 100%; }

  .window {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    will-change: transform;
  }

  .section-header {
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    box-sizing: border-box;
  }

  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 13px;
    padding: 24px;
  }
</style>
