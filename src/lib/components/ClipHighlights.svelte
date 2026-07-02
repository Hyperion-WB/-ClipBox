<script lang="ts">
  import type { Highlight } from "$lib/extractHighlights";
  import { highlightIcon } from "$lib/extractHighlights";
  import { partitionHighlights, shouldCollapseChips } from "$lib/chipLayout";
  import { formatHighlight } from "$lib/formatContent";
  import { maskForDisplay } from "$lib/sensitiveMask";
  import { t } from "$lib/i18n.svelte";

  interface Props {
    items: Highlight[];
    onPaste: (value: string) => void;
    onOpenPath?: (value: string) => void;
    onOpenUrl?: (value: string) => void;
    maskSensitive?: boolean;
    revealed?: boolean;
  }

  let { items, onPaste, onOpenPath, onOpenUrl, maskSensitive = true, revealed = false }: Props = $props();
  let expanded = $state(false);

  const foldable = $derived(shouldCollapseChips(items));
  const { links, extras } = $derived(partitionHighlights(items));
  const visible = $derived(foldable && !expanded ? links : items);
  const hiddenCount = $derived(foldable ? extras.length : 0);

  function chipTitle(h: Highlight): string {
    if (h.kind === "json" || h.kind === "code") return t("chip.formatPasteHint");
    const raw = h.value.length > 80 ? h.value.slice(0, 80) + "…" : h.value;
    if (maskSensitive && !revealed && (h.kind === "otp" || h.kind === "tracking")) {
      return maskForDisplay(raw, true);
    }
    return raw;
  }

  function chipLabel(h: Highlight): string {
    if (maskSensitive && !revealed && (h.kind === "otp" || h.kind === "tracking")) {
      return maskForDisplay(h.label, true);
    }
    return h.label;
  }

  function onChip(e: MouseEvent, h: Highlight) {
    e.stopPropagation();
    if (h.kind === "url" && onOpenUrl) {
      onOpenUrl(h.value);
    } else if (h.kind === "ip" && onOpenUrl) {
      onOpenUrl(`http://${h.value}`);
    } else if (h.kind === "path" && onOpenPath) {
      onOpenPath(h.value);
    } else if (h.kind === "json" || h.kind === "code") {
      try {
        onPaste(formatHighlight(h));
      } catch {
        onPaste(h.value);
      }
    } else {
      onPaste(h.value);
    }
  }

  function toggleExpand(e: MouseEvent) {
    e.stopPropagation();
    expanded = !expanded;
  }
</script>

<div class="chip-row">
  <div class="chips">
    {#each visible as h (h.kind + h.value)}
      <button
        type="button"
        class="chip"
        class:url={h.kind === "url"}
        class:path={h.kind === "path"}
        class:otp={h.kind === "otp"}
        class:tracking={h.kind === "tracking"}
        class:json={h.kind === "json"}
        class:code={h.kind === "code"}
        title={chipTitle(h)}
        onclick={(e) => onChip(e, h)}
      >
        <span class="icon">{highlightIcon(h.kind)}</span>
        <span class="label">{chipLabel(h)}</span>
      </button>
    {/each}

    {#if foldable && hiddenCount > 0 && !expanded}
      <button
        type="button"
        class="chip toggle"
        title={t("chip.showMore", { n: hiddenCount })}
        onclick={toggleExpand}
      >
        <span class="toggle-icon">▸</span>
        <span class="toggle-label">{hiddenCount}</span>
      </button>
    {:else if foldable && expanded}
      <button
        type="button"
        class="chip toggle"
        title={t("chip.collapse")}
        onclick={toggleExpand}
      >
        <span class="toggle-icon">▾</span>
      </button>
    {/if}
  </div>

  {#if expanded && foldable && extras.length > 0}
    <div class="chip-extra" role="group" aria-label={t("chip.moreGroup")}>
      {#each extras as h (h.kind + h.value)}
        <button
          type="button"
          class="chip"
          class:otp={h.kind === "otp"}
          class:tracking={h.kind === "tracking"}
          class:json={h.kind === "json"}
          class:code={h.kind === "code"}
          title={chipTitle(h)}
          onclick={(e) => onChip(e, h)}
        >
          <span class="icon">{highlightIcon(h.kind)}</span>
          <span class="label">{chipLabel(h)}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .chip-row {
    margin-top: 3px;
    min-width: 0;
  }

  .chips {
    display: flex;
    flex-wrap: nowrap;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    min-height: 20px;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    max-width: min(150px, 42vw);
    border: 1px solid var(--chip-border, transparent);
    border-radius: 999px;
    padding: 1px 7px 1px 5px;
    font-size: 10px;
    line-height: 1.35;
    background: var(--chip-bg, color-mix(in srgb, var(--accent) 10%, var(--surface)));
    color: var(--chip-fg, var(--text));
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color var(--duration-fast, 0.15s) var(--ease-smooth, ease),
      border-color var(--duration-fast, 0.15s) var(--ease-smooth, ease),
      transform var(--duration-fast, 0.15s) var(--ease-spring, ease);
  }

  .chip:hover {
    background: var(--chip-bg-hover, color-mix(in srgb, var(--accent) 18%, var(--surface)));
    border-color: var(--chip-border-hover, var(--chip-border, transparent));
    transform: translateY(-1px);
  }

  .chip:active {
    transform: translateY(0) scale(0.97);
  }

  .chip.toggle {
    max-width: none;
    min-width: 28px;
    padding: 1px 7px;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, var(--surface));
    border-color: color-mix(in srgb, var(--accent) 25%, var(--border));
    font-weight: 600;
  }

  .toggle-icon {
    font-size: 9px;
    line-height: 1;
  }

  .toggle-label {
    font-size: 10px;
  }

  .chip.url {
    --chip-bg: var(--chip-url-bg);
    --chip-bg-hover: var(--chip-url-bg-hover);
    --chip-fg: var(--chip-url-fg);
    --chip-border: var(--chip-url-border);
  }

  .chip.path {
    --chip-bg: var(--chip-path-bg);
    --chip-bg-hover: var(--chip-path-bg-hover);
    --chip-fg: var(--chip-path-fg);
    --chip-border: var(--chip-path-border);
  }

  .chip.otp {
    --chip-bg: var(--chip-otp-bg);
    --chip-bg-hover: var(--chip-otp-bg-hover);
    --chip-fg: var(--chip-otp-fg);
    --chip-border: var(--chip-otp-border);
  }

  .chip.tracking {
    --chip-bg: var(--chip-tracking-bg);
    --chip-bg-hover: var(--chip-tracking-bg-hover);
    --chip-fg: var(--chip-tracking-fg);
    --chip-border: var(--chip-tracking-border);
  }

  .chip.json,
  .chip.code {
    --chip-bg: var(--chip-code-bg);
    --chip-bg-hover: var(--chip-code-bg-hover);
    --chip-fg: var(--chip-code-fg);
    --chip-border: var(--chip-code-border);
  }

  .chip-extra {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
    padding: 4px 0 0;
    border-top: 1px dashed var(--border);
  }

  .icon {
    font-size: 9px;
    flex-shrink: 0;
    font-family: ui-monospace, monospace;
  }

  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
</style>
