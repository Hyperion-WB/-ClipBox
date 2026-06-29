<script lang="ts">
  import type { Snippet } from "$lib/types";
  import { t } from "$lib/i18n.svelte";

  interface Props {
    snippets: Snippet[];
    selectedIndex: number;
    onSelect: (index: number) => void;
    onPaste: (snippet: Snippet) => void;
    onCreate: (title: string, content: string) => void;
    onUpdate: (id: number, title: string, content: string) => void;
    onDelete: (id: number) => void;
    onReorder: (ids: number[]) => void;
  }

  let {
    snippets, selectedIndex, onSelect, onPaste, onCreate, onUpdate, onDelete, onReorder,
  }: Props = $props();

  let editingId = $state<number | null>(null);
  let editTitle = $state("");
  let editContent = $state("");
  let showAdd = $state(false);
  let newTitle = $state("");
  let newContent = $state("");
  let dragIndex = $state<number | null>(null);
  let dropIndex = $state<number | null>(null);

  function startEdit(snippet: Snippet) {
    editingId = snippet.id;
    editTitle = snippet.title;
    editContent = snippet.content;
  }

  function saveEdit() {
    if (editingId !== null) {
      onUpdate(editingId, editTitle, editContent);
      editingId = null;
    }
  }

  function submitAdd() {
    if (newTitle.trim() && newContent.trim()) {
      onCreate(newTitle.trim(), newContent.trim());
      newTitle = "";
      newContent = "";
      showAdd = false;
    }
  }

  function onDragStart(index: number) {
    dragIndex = index;
  }

  function onDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    dropIndex = index;
  }

  function onDrop() {
    if (dragIndex === null || dropIndex === null || dragIndex === dropIndex) {
      dragIndex = null;
      dropIndex = null;
      return;
    }
    const ids = snippets.map((s) => s.id);
    const [moved] = ids.splice(dragIndex, 1);
    ids.splice(dropIndex, 0, moved);
    onReorder(ids);
    dragIndex = null;
    dropIndex = null;
  }
</script>

<div class="snippet-tab">
  <div class="header">
    <div class="header-text">
      <span class="title">{t("snippets.title")}</span>
      <span class="hint">{t("snippets.clickHint")}</span>
    </div>
    <button type="button" class="add-btn" onclick={() => (showAdd = !showAdd)}>
      {showAdd ? t("snippets.cancel") : t("snippets.add")}
    </button>
  </div>

  {#if showAdd}
    <div class="add-form">
      <input bind:value={newTitle} placeholder={t("snippets.titlePlaceholder")} />
      <textarea bind:value={newContent} placeholder={t("snippets.contentPlaceholder")} rows="2"></textarea>
      <button type="button" class="save-btn" onclick={submitAdd}>{t("snippets.save")}</button>
    </div>
  {/if}

  <div class="list">
    {#each snippets as snippet, i (snippet.id)}
      {#if editingId === snippet.id}
        <div class="edit-form">
          <input bind:value={editTitle} />
          <textarea bind:value={editContent} rows="2"></textarea>
          <div class="edit-actions">
            <button type="button" onclick={saveEdit}>{t("snippets.save")}</button>
            <button type="button" onclick={() => (editingId = null)}>{t("snippets.cancel")}</button>
          </div>
        </div>
      {:else}
        <div
          class="snippet-item"
          class:selected={selectedIndex === i}
          class:drop-target={dropIndex === i}
          role="button"
          tabindex="0"
          ondragover={(e) => onDragOver(e, i)}
          ondrop={onDrop}
          onclick={() => onPaste(snippet)}
          onkeydown={(e) => e.key === "Enter" && onPaste(snippet)}
        >
          <span
            class="drag-handle"
            draggable="true"
            ondragstart={() => onDragStart(i)}
            onclick={(e) => e.stopPropagation()}
          >⋮⋮</span>
          <div class="snippet-info">
            <span class="snippet-title">{snippet.title}</span>
            <span class="snippet-preview">{snippet.content}</span>
          </div>
          <div class="snippet-actions">
            <button type="button" title={t("snippets.edit")} onclick={(e) => { e.stopPropagation(); startEdit(snippet); }}>✎</button>
            <button type="button" title={t("snippets.delete")} class="delete" onclick={(e) => { e.stopPropagation(); onDelete(snippet.id); }}>×</button>
          </div>
        </div>
      {/if}
    {:else}
      <div class="empty">{t("snippets.empty")}</div>
    {/each}
  </div>
</div>

<style>
  .snippet-tab { display: flex; flex-direction: column; flex: 1; overflow: hidden; }
  .header { display: flex; justify-content: space-between; align-items: flex-start; padding: 8px 12px; border-bottom: 1px solid var(--border); gap: 8px; }
  .header-text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .title { font-size: 12px; color: var(--text-muted); font-weight: 500; }
  .hint { font-size: 10px; color: var(--text-muted); opacity: 0.85; }
  .add-btn { border: none; background: var(--accent); color: white; padding: 4px 10px; border-radius: 6px; font-size: 12px; cursor: pointer; }
  .add-form, .edit-form { padding: 8px 12px; display: flex; flex-direction: column; gap: 6px; border-bottom: 1px solid var(--border); }
  input, textarea { border: 1px solid var(--border); border-radius: 6px; padding: 6px 8px; font-size: 12px; background: var(--surface); color: var(--text); resize: none; font-family: inherit; }
  .save-btn { align-self: flex-end; border: none; background: var(--accent); color: white; padding: 4px 12px; border-radius: 6px; font-size: 12px; cursor: pointer; }
  .list { flex: 1; overflow-y: auto; padding: 4px 0; }
  .snippet-item { display: flex; align-items: center; padding: 8px 12px; margin: 2px 6px; border-radius: 8px; cursor: grab; }
  .snippet-item:hover, .snippet-item.selected { background: var(--hover); }
  .snippet-item.drop-target { border-top: 2px solid var(--accent); }
  .drag-handle { color: var(--text-muted); font-size: 12px; margin-right: 8px; cursor: grab; }
  .snippet-info { flex: 1; min-width: 0; }
  .snippet-title { font-size: 13px; font-weight: 500; color: var(--text); display: block; }
  .snippet-preview { font-size: 11px; color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: block; }
  .snippet-actions { display: flex; gap: 2px; opacity: 0; }
  .snippet-item:hover .snippet-actions { opacity: 1; }
  .snippet-actions button { width: 24px; height: 24px; border: none; background: transparent; border-radius: 4px; cursor: pointer; font-size: 12px; color: var(--text-muted); }
  .snippet-actions .delete:hover { color: #e53935; }
  .edit-actions { display: flex; gap: 6px; justify-content: flex-end; }
  .edit-actions button { border: 1px solid var(--border); background: var(--surface); padding: 4px 10px; border-radius: 6px; font-size: 12px; cursor: pointer; color: var(--text); }
  .empty { padding: 24px; text-align: center; color: var(--text-muted); font-size: 13px; }
</style>
