<script lang="ts">
  import type { ClipItem, ContextMenuAction } from "$lib/types";
  import { itemHasFormatAction } from "$lib/formatContent";
  import { t } from "$lib/i18n.svelte";

  interface Props {
    x: number;
    y: number;
    item: ClipItem;
    onAction: (action: ContextMenuAction) => void;
    onClose: () => void;
  }

  let { x, y, item, onAction, onClose }: Props = $props();

  const menuItems = $derived.by(() => {
    const items: { action: ContextMenuAction; label: string; danger?: boolean }[] = [
      { action: "paste", label: t("menu.paste") },
    ];
    if (item.content_type === "html") {
      items.push({ action: "pastePlain", label: t("menu.pastePlain") });
    }
    if (itemHasFormatAction(item.content_text, item.content_type)) {
      items.push({ action: "formatPaste", label: t("menu.formatPaste") });
    }
    items.push({ action: "copy", label: t("menu.copy") });
    if (item.content_type === "file") {
      items.push({ action: "openPath", label: t("menu.openPath") });
    }
    if (item.content_type === "image") {
      items.push({ action: "saveImage", label: t("menu.saveImage") });
    }
    items.push({
      action: "pin",
      label: item.pinned ? t("menu.unpin") : t("menu.pin"),
    });
    items.push({ action: "delete", label: t("menu.delete"), danger: true });
    return items;
  });
</script>

<svelte:window onclick={onClose} oncontextmenu={(e) => { e.preventDefault(); onClose(); }} />

<div class="menu" style="left: {x}px; top: {y}px" role="menu">
  {#each menuItems as mi (mi.action)}
    <button
      type="button"
      class:danger={mi.danger}
      onclick={(e) => { e.stopPropagation(); onAction(mi.action); onClose(); }}
    >
      {mi.label}
    </button>
  {/each}
</div>

<style>
  .menu {
    position: fixed;
    z-index: 9999;
    min-width: 148px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.14);
    padding: 4px;
  }

  .menu button {
    display: block;
    width: 100%;
    border: none;
    background: transparent;
    text-align: left;
    padding: 8px 12px;
    font-size: 13px;
    color: var(--text);
    border-radius: 6px;
    cursor: pointer;
  }

  .menu button:hover { background: var(--hover); }
  .menu button.danger { color: #e53935; }
</style>
