<script lang="ts">
  import type { Highlight } from "$lib/extractHighlights";
  import type { ClipItem } from "$lib/types";
  import { itemHasFormatAction } from "$lib/formatContent";
  import { t } from "$lib/i18n.svelte";
  import {
    analyzeSensitive,
    maskForDisplay,
    sensitiveKindLabel,
    type SensitiveKind,
  } from "$lib/sensitiveMask";
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
    maskSensitive?: boolean;
    revealed?: boolean;
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
    onToggleReveal?: () => void;
  }

  let {
    item,
    index,
    selected,
    checked,
    multiSelectMode,
    rowHeight = 44,
    highlights = [],
    maskSensitive = true,
    revealed = false,
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
    onToggleReveal,
  }: Props = $props();

  const expanded = $derived(highlights.length > 0);
  const sensitiveInfo = $derived(
    item.content_type !== "image" && item.content_type !== "file"
      ? analyzeSensitive(item.content_text)
      : { sensitive: false, kinds: [] as SensitiveKind[] },
  );
  const showMasked = $derived(maskSensitive && sensitiveInfo.sensitive && !revealed);
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

  const displayText = $derived(
    preview(showMasked ? maskForDisplay(item.content_text, true) : item.content_text),
  );

  const sensitiveTitle = $derived(
    sensitiveInfo.kinds.map((k) => sensitiveKindLabel(k, t)).join(" · "),
  );

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
  const showHotkey = $derived(!multiSelectMode && hotkeyNum !== null);
</script>

<div
  class="clip-item"
  class:selected
  class:checked
  class:pinned={item.pinned}
  class:expanded
  class:has-chips={expanded}
  class:sensitive={showMasked}
  class:has-hotkey={showHotkey}
  style:min-height="{rowHeight}px"
  role="button"
  tabindex="-1"
  onclick={onSelect}
  oncontextmenu={onContextMenu}
>
  {#if multiSelectMode}
    <input type="checkbox" checked={checked} tabindex="-1" readonly />
  {:else if showHotkey}
    <span class="hotkey-num">{hotkeyNum}</span>
  {:else}
    <span class="hotkey-num empty"></span>
  {/if}

  {#if !showHotkey}
    <div class="type-badge" data-type={item.content_type}>{typeLabel(item.content_type)}</div>
  {/if}

  <div class="content">
    {#if item.content_type === "image" && item.has_thumbnail}
      <ClipThumb clipId={item.id} />
    {/if}
    <div class="text-block">
      <span class="text" class:masked={showMasked}>{displayText}</span>
      {#if showMasked}
        <span class="sensitive-badge" title={sensitiveTitle}>{t("sensitive.badge")}</span>
      {/if}
      {#if expanded && onPasteSegment}
        <ClipHighlights
          items={highlights}
          onPaste={onPasteSegment}
          {onOpenPath}
          {onOpenUrl}
          {maskSensitive}
          revealed={revealed}
        />
      {:else if item.source_app}
        <SourceBadge name={item.source_app} />
      {/if}
    </div>
  </div>

  <div class="actions">
    {#if showMasked && onToggleReveal}
      <button
        class="action-btn reveal"
        title={t("sensitive.reveal")}
        onclick={(e) => { e.stopPropagation(); onToggleReveal(); }}
      >👁</button>
    {:else if revealed && sensitiveInfo.sensitive && onToggleReveal}
      <button
        class="action-btn reveal"
        title={t("sensitive.hide")}
        onclick={(e) => { e.stopPropagation(); onToggleReveal(); }}
      >🙈</button>
    {/if}
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
      >
        <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" fill="none">
          <path d="M3.5 4.5h7.5M3.5 8h5M3.5 11.5h6.5" stroke="currentColor" stroke-width="1.35" stroke-linecap="round"/>
          <path d="M12.5 6.5 14.5 8 12.5 9.5" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
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
    gap: 8px;
    padding: 6px 6px 6px 8px;
    cursor: pointer;
    border-radius: 10px;
    margin: 0 4px;
    box-sizing: border-box;
    transition:
      background-color var(--duration-fast, 0.15s) var(--ease-smooth, ease),
      box-shadow var(--duration-fast, 0.15s) var(--ease-smooth, ease);
    contain: layout style;
  }

  .clip-item.has-hotkey {
    gap: 10px;
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

  .clip-item.selected {
    box-shadow: inset 2px 0 0 var(--accent);
  }

  .clip-item.pinned {
    background: color-mix(in srgb, var(--accent) 5%, transparent);
  }

  .clip-item.pinned:hover,
  .clip-item.pinned.selected {
    background: color-mix(in srgb, var(--accent) 10%, var(--hover));
  }

  .clip-item.sensitive.selected {
    box-shadow: inset 2px 0 0 var(--sensitive-accent);
  }

  .hotkey-num {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    background: linear-gradient(145deg, var(--accent), color-mix(in srgb, var(--accent) 82%, #000));
    color: white;
    font-size: 11px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
    box-shadow: 0 1px 3px color-mix(in srgb, var(--accent) 35%, transparent);
  }

  .clip-item:not(.expanded) .hotkey-num {
    margin-top: 0;
  }

  .hotkey-num.empty {
    background: transparent;
    box-shadow: none;
    width: 20px;
  }

  .type-badge {
    width: 22px;
    height: 22px;
    border-radius: 6px;
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

  .clip-item.sensitive {
    background: color-mix(in srgb, var(--sensitive-tint) 6%, transparent);
    box-shadow: inset 2px 0 0 var(--sensitive-accent);
  }

  .clip-item.sensitive:hover,
  .clip-item.sensitive.selected {
    background: color-mix(in srgb, var(--sensitive-tint) 10%, var(--hover));
  }

  .text.masked {
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
  }

  .sensitive-badge {
    display: inline-block;
    margin-top: 2px;
    margin-right: 4px;
    padding: 0 5px;
    font-size: 10px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--sensitive-accent) 12%, var(--surface));
    color: var(--sensitive-accent);
    line-height: 1.5;
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
    border-radius: 6px;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition:
      background-color var(--duration-fast, 0.15s) var(--ease-smooth, ease),
      color var(--duration-fast, 0.15s) var(--ease-smooth, ease),
      transform var(--duration-fast, 0.15s) var(--ease-spring, ease);
  }

  .pin-btn:hover {
    background: var(--hover-strong);
    color: var(--text);
    transform: scale(1.06);
  }

  .pin-btn:active {
    transform: scale(0.94);
  }

  .pin-btn.is-pinned {
    color: var(--accent);
  }

  .actions {
    display: flex;
    gap: 1px;
    opacity: 0;
    transform: translateX(3px);
    flex-shrink: 0;
    align-self: flex-start;
    margin-top: 0;
    padding-left: 2px;
    pointer-events: none;
    transition:
      opacity var(--duration-normal, 0.2s) var(--ease-smooth, ease),
      transform var(--duration-normal, 0.2s) var(--ease-smooth, ease);
  }

  .clip-item:not(.expanded) .actions {
    margin-top: 0;
  }

  .clip-item:hover .actions,
  .clip-item.selected .actions {
    opacity: 1;
    transform: translateX(0);
    pointer-events: auto;
  }

  .action-btn {
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    font-size: 11px;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition:
      background-color var(--duration-fast, 0.15s) var(--ease-smooth, ease),
      color var(--duration-fast, 0.15s) var(--ease-smooth, ease),
      transform var(--duration-fast, 0.15s) var(--ease-spring, ease);
  }

  .action-btn:hover {
    background: var(--hover-strong);
    color: var(--text);
    transform: scale(1.06);
  }

  .action-btn:active {
    transform: scale(0.94);
  }

  .action-btn.delete:hover {
    color: #e53935;
  }

  .action-btn.format {
    color: var(--accent);
  }

  .action-btn.format svg {
    display: block;
  }

  .action-btn.reveal {
    font-size: 10px;
  }

  input[type="checkbox"] {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    pointer-events: none;
    margin-top: 4px;
  }

  @media (prefers-reduced-motion: reduce) {
    .clip-item,
    .actions,
    .action-btn,
    .pin-btn {
      transition: none;
    }
    .action-btn:hover,
    .pin-btn:hover,
    .action-btn:active,
    .pin-btn:active {
      transform: none;
    }
  }
</style>
