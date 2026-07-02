<script lang="ts">

  import { api } from "$lib/api";

  import {

    buildListLayout,

    clearHighlightCache,

    nextClipRowIndex,

    rowIndexForClipIndex,

  } from "$lib/grouping";

  import type {

    AppSettings,

    ClipCategory,

    ClipItem,

    ContextMenuAction,

    HistoryStats,

    Snippet,

    ThemeMode,

  } from "$lib/types";

  import CategoryFilter from "./CategoryFilter.svelte";

  import ContextMenu from "./ContextMenu.svelte";

  import SearchBar from "./SearchBar.svelte";

  import SettingsPanel from "./Settings.svelte";

  import SnippetTab from "./SnippetTab.svelte";

  import TagFilter from "./TagFilter.svelte";

  import VirtualClipList from "./VirtualClipList.svelte";

  import MinimalHistoryView from "./MinimalHistoryView.svelte";

  import { applyTheme, watchSystemTheme } from "$lib/theme";
  import { dragWindow } from "$lib/window";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import {
    formatCode,
    formatJson,
    isJsonText,
  } from "$lib/formatContent";
  import { clearSourceBadgeCache } from "$lib/sourceBadgeCache";
  import { clearThumbCache } from "$lib/thumbCache";
  import { setLocale, t } from "$lib/i18n.svelte";
  type Tab = "history" | "snippets" | "settings";

  const TAB_ORDER: Record<Tab, number> = { history: 0, snippets: 1, settings: 2 };

  let activeTab = $state<Tab>("history");
  let tabDir = $state(1);

  let clips = $state<ClipItem[]>([]);

  let snippets = $state<Snippet[]>([]);

  let settings = $state<AppSettings | null>(null);

  let stats = $state<HistoryStats | null>(null);

  let paused = $state(false);

  let searchQuery = $state("");

  let activeTags = $state<string[]>([]);

  let category = $state<ClipCategory>("all");

  let selectedClipIndex = $state(0);

  let selectedRowIndex = $state(0);

  let selectedIds = $state<Set<number>>(new Set());

  let multiSelectMode = $state(false);

  let searchInput = $state<HTMLInputElement | null>(null);

  let panelOpen = $state(false);

  let contextMenu = $state<{ x: number; y: number; item: ClipItem } | null>(null);

  let diskLabel = $state("");

  let toast = $state("");

  let lastTrashedId = $state<number | null>(null);

  let undoVisible = $state(false);

  let undoTimer: ReturnType<typeof setTimeout> | undefined;

  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  let clipReloadTimer: ReturnType<typeof setTimeout> | 0 = 0;

  let lastClickedRow = $state(0);

  let revealedIds = $state<Set<number>>(new Set());



  const flatClips = $derived.by(() => {

    const pinned = clips.filter((c) => c.pinned);

    const history = clips.filter((c) => !c.pinned);

    return [...pinned, ...history];

  });



  const listLayout = $derived(

    buildListLayout(flatClips, {

      groupByTime: settings?.group_by_time ?? false,

      groupBySource: settings?.group_by_source ?? false,

    }),

  );



  const selectedItem = $derived.by(() => {

    const row = listLayout.rows[selectedRowIndex];

    return row?.kind === "clip" ? row.item : null;

  });



  function buildQuery(): string {

    return [searchQuery, ...activeTags].map((s) => s.trim()).filter(Boolean).join(" ");

  }



  function syncRowFromClipIndex() {

    const row = rowIndexForClipIndex(listLayout, selectedClipIndex);

    if (row >= 0) selectedRowIndex = row;

  }



  async function loadClips() {

    clearHighlightCache();

    clips = await api.listClips(buildQuery() || undefined, category);

    selectedClipIndex = Math.min(selectedClipIndex, Math.max(0, flatClips.length - 1));

    syncRowFromClipIndex();

  }



  async function loadSnippets() {

    snippets = await api.listSnippets();

    selectedClipIndex = Math.min(selectedClipIndex, Math.max(0, snippets.length - 1));

  }



  async function loadSettings() {
    settings = await api.getSettings();
    paused = await api.isMonitorPaused();
    setLocale(settings.locale);
    applyTheme(settings.theme);
  }

  async function loadStats() {

    const data = await api.getHistoryStats();

    await applyStats(data);

  }



  function toggleReveal(id: number) {
    const next = new Set(revealedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    revealedIds = next;
  }

  function switchTab(tab: Tab) {
    tabDir = TAB_ORDER[tab] >= TAB_ORDER[activeTab] ? 1 : -1;
    activeTab = tab;
    if (tab === "history" && panelOpen) loadClips();
    if (tab === "snippets" && panelOpen) loadSnippets();
  }

  async function applyStats(statsData: HistoryStats) {
    stats = statsData;
    diskLabel = await api.formatDiskSize(statsData.disk_bytes);
  }

  async function clearAllHistory() {
    const count = stats
      ? Math.max(0, stats.total_clips - stats.pinned_clips)
      : flatClips.filter((c) => !c.pinned).length;
    if (count === 0) {
      toast = t("history.clearAllEmpty");
      setTimeout(() => (toast = ""), 2000);
      return;
    }
    const ok = await confirm(t("history.clearAllConfirm", { n: count }), {
      title: "ClipBox",
      kind: "warning",
    });
    if (!ok) return;
    await api.clearHistory(true);
    selectedClipIndex = 0;
    selectedRowIndex = 0;
    selectedIds = new Set();
    multiSelectMode = false;
    await refreshPanelData();
    toast = t("history.clearedToTrash", { n: count });
    setTimeout(() => (toast = ""), 2500);
  }



  async function refreshPanelData() {

    await Promise.all([loadClips(), loadSnippets(), loadStats()]);

  }



  function showTrashToast(id: number) {

    lastTrashedId = id;

    undoVisible = true;

    toast = t("history.trashed");

    if (undoTimer) clearTimeout(undoTimer);

    undoTimer = setTimeout(() => {

      undoVisible = false;

      lastTrashedId = null;

      toast = "";

    }, 8000);

  }



  async function trashClip(id: number) {

    await api.deleteClip(id);

    showTrashToast(id);

    await refreshPanelData();

  }



  async function undoTrash() {

    if (lastTrashedId == null) return;

    await api.restoreClip(lastTrashedId);

    lastTrashedId = null;

    undoVisible = false;

    if (undoTimer) clearTimeout(undoTimer);

    toast = t("history.restored");

    setTimeout(() => (toast = ""), 2000);

    await refreshPanelData();

  }



  const pinnedClips = $derived(flatClips.filter((c) => c.pinned));

  const historyOnlyClips = $derived(flatClips.filter((c) => !c.pinned));

  const isMinimal = $derived(settings?.minimal_mode ?? false);

  const nearHistoryLimit = $derived(

    settings && stats ? stats.total_clips >= settings.max_history * 0.9 : false,

  );



  function onSearch(value: string) {

    searchQuery = value;

    if (searchTimer) clearTimeout(searchTimer);

    searchTimer = setTimeout(() => loadClips(), 300);

  }



  function onTagsChange(tags: string[]) {

    activeTags = tags;

    loadClips();

  }



  function onCategoryChange(c: ClipCategory) {

    category = c;

    loadClips();

  }



  function clipAtIndex(clipIndex: number): ClipItem | undefined {

    return flatClips[clipIndex];

  }



  async function pasteAt(clipIndex: number, plainTextOnly = false) {

    const item = clipAtIndex(clipIndex);

    if (!item) return;

    await api.pasteItem(item.id, plainTextOnly);

  }

  async function pasteSegment(value: string) {
    await api.pasteText(value);
  }

  async function pasteFormatted(item: ClipItem) {
    try {
      const text = item.content_text.trim();
      const formatted = isJsonText(text) ? formatJson(text) : formatCode(text);
      await api.pasteText(formatted);
    } catch {
      await api.pasteText(item.content_text);
    }
  }

  async function openPath(path: string) {
    try {
      await api.openPath(path.trim());
    } catch (err) {
      toast = err instanceof Error ? err.message : String(err);
      setTimeout(() => (toast = ""), 2500);
    }
  }

  async function openUrl(url: string) {
    try {
      await api.openUrl(url.trim());
    } catch (err) {
      toast = err instanceof Error ? err.message : String(err);
      setTimeout(() => (toast = ""), 2500);
    }
  }



  async function pasteSelected(plainTextOnly = false) {

    if (multiSelectMode && selectedIds.size > 0) return;

    await pasteAt(selectedClipIndex, plainTextOnly);

  }



  async function saveImageToFolder(item: ClipItem) {

    const { open } = await import("@tauri-apps/plugin-dialog");

    const folder = await open({

      directory: true,

      multiple: false,

      defaultPath: settings?.image_save_dir || undefined,

    });

    if (!folder || typeof folder !== "string") return;

    try {

      const path = await api.saveClipImage(item.id, folder);

      const name = path.split(/[/\\]/).pop() ?? path;

      toast = t("history.saved", { name });

    } catch (err) {

      toast = err instanceof Error ? err.message : String(err);

    }

    setTimeout(() => (toast = ""), 2500);

  }



  async function handleContextAction(action: ContextMenuAction, item: ClipItem) {

    switch (action) {

      case "paste":

        await api.pasteItem(item.id, false);
        break;

      case "pastePlain":

        await api.pasteItem(item.id, true);

        break;

      case "formatPaste":

        await pasteFormatted(item);

        break;

      case "copy":

        await api.copyItemToClipboard(item.id, item.content_type === "html");

        break;

      case "openPath":

        await openPath(item.content_text);

        break;

      case "saveImage":

        await saveImageToFolder(item);

        break;

      case "pin":

        await api.togglePin(item.id);

        await loadClips();

        break;

      case "delete":

        await trashClip(item.id);

        break;

    }

  }



  function rowClip(rowIndex: number): ClipItem | null {

    const row = listLayout.rows[rowIndex];

    return row?.kind === "clip" ? row.item : null;

  }



  function handleItemSelect(rowIndex: number, e: MouseEvent) {

    const item = rowClip(rowIndex);

    if (!item) return;



    if (e.ctrlKey || e.metaKey) {

      multiSelectMode = true;

      const next = new Set(selectedIds);

      if (next.has(item.id)) next.delete(item.id);

      else next.add(item.id);

      selectedIds = next;

      selectedRowIndex = rowIndex;

      const row = listLayout.rows[rowIndex];

      if (row?.kind === "clip") selectedClipIndex = row.clipIndex;

      return;

    }



    if (e.shiftKey && multiSelectMode) {

      const start = Math.min(lastClickedRow, rowIndex);

      const end = Math.max(lastClickedRow, rowIndex);

      const next = new Set(selectedIds);

      for (let i = start; i <= end; i++) {

        const clip = rowClip(i);

        if (clip) next.add(clip.id);

      }

      selectedIds = next;

      selectedRowIndex = rowIndex;

      return;

    }



    const row = listLayout.rows[rowIndex];

    if (row?.kind === "clip") selectedClipIndex = row.clipIndex;

    selectedRowIndex = rowIndex;

    lastClickedRow = rowIndex;



    if (multiSelectMode) return;

    pasteAt(selectedClipIndex, false);

  }



  async function batchDelete() {

    if (selectedIds.size === 0) return;

    await api.deleteClips([...selectedIds]);

    selectedIds = new Set();

    multiSelectMode = false;

    await refreshPanelData();

  }



  function exitMultiSelect() {

    multiSelectMode = false;

    selectedIds = new Set();

  }



  function handleKeydown(e: KeyboardEvent) {

    if (e.key === "Escape") {

      if (multiSelectMode) {

        exitMultiSelect();

        return;

      }

      if (contextMenu) {

        contextMenu = null;

        return;

      }

      api.hidePanel();

      return;

    }



    if (e.altKey && !e.ctrlKey && !e.shiftKey) {

      const catMap: Record<string, ClipCategory> = {

        "1": "all", "2": "text", "3": "image", "4": "file",

      };

      if (catMap[e.key] && activeTab === "history") {

        e.preventDefault();

        onCategoryChange(catMap[e.key]);

        return;

      }

    }



    if (activeTab === "history" && !multiSelectMode && !e.ctrlKey && !e.altKey && !e.metaKey) {

      const num = parseInt(e.key, 10);

      if (num >= 1 && num <= 9 && flatClips[num - 1]) {

        e.preventDefault();

        pasteAt(num - 1, false);

        return;

      }

    }



    if (e.ctrlKey && e.key === "f") {

      e.preventDefault();

      searchInput?.focus();

      return;

    }



    if (e.ctrlKey && e.key === "a" && activeTab === "history") {

      e.preventDefault();

      multiSelectMode = true;

      selectedIds = new Set(flatClips.map((c) => c.id));

      return;

    }



    if (activeTab === "settings") return;



    if (activeTab === "history") {

      const maxRow = listLayout.rows.length - 1;

      if (e.key === "ArrowDown") {

        e.preventDefault();

        const next = nextClipRowIndex(listLayout, selectedRowIndex, 1);

        selectedRowIndex = next;

        const row = listLayout.rows[next];

        if (row?.kind === "clip") selectedClipIndex = row.clipIndex;

      } else if (e.key === "ArrowUp") {

        e.preventDefault();

        const next = nextClipRowIndex(listLayout, selectedRowIndex, -1);

        selectedRowIndex = next;

        const row = listLayout.rows[next];

        if (row?.kind === "clip") selectedClipIndex = row.clipIndex;

      } else if (e.key === "Enter") {

        e.preventDefault();

        if (multiSelectMode) return;

        pasteSelected(e.shiftKey);

      } else if (e.key === "Delete" && multiSelectMode) {

        batchDelete();

      }

      return;

    }



    const max = snippets.length - 1;

    if (e.key === "ArrowDown") {

      e.preventDefault();

      selectedClipIndex = Math.min(selectedClipIndex + 1, max);

    } else if (e.key === "ArrowUp") {

      e.preventDefault();

      selectedClipIndex = Math.max(selectedClipIndex - 1, 0);

    } else if (e.key === "Enter") {

      e.preventDefault();

      const s = snippets[selectedClipIndex];

      if (s) {

        api.pasteSnippet(s.content);

      }

    }

  }



  function releaseMemory() {

    clips = [];

    snippets = [];

    stats = null;

    selectedIds = new Set();

    multiSelectMode = false;

    clearHighlightCache();

    clearSourceBadgeCache();

    clearThumbCache();

  }



  $effect(() => {

    loadSettings();



    import("@tauri-apps/api/event").then(({ listen }) => {

      const unsubs: Array<() => void> = [];



      listen("panel-opened", async () => {

        panelOpen = true;

        await refreshPanelData();

      }).then((u) => unsubs.push(u));



      listen("panel-closed", () => {

        panelOpen = false;

        contextMenu = null;

        if (settings?.release_memory_on_close) releaseMemory();

      }).then((u) => unsubs.push(u));



      listen<number>("clip-added", () => {

        if (panelOpen && activeTab === "history") {

          if (clipReloadTimer) clearTimeout(clipReloadTimer);

          clipReloadTimer = setTimeout(() => {

            loadClips();

            loadStats();

            clipReloadTimer = 0;

          }, 180);

        }

      }).then((u) => unsubs.push(u));



      listen("open-settings", () => {

        activeTab = "settings";

      }).then((u) => unsubs.push(u));



      listen<string>("settings-changed", (ev) => {

        applyTheme(ev.payload as ThemeMode);

      }).then((u) => unsubs.push(u));



      return () => unsubs.forEach((u) => u());

    });

  });



  $effect(() => {

    const themeMode = settings?.theme;

    if (!themeMode) return;

    return watchSystemTheme(() => {

      if (themeMode === "system") applyTheme("system");

    });

  });

</script>



<svelte:window onkeydown={handleKeydown} />



<div class="panel">

  {#if settings?.window_draggable !== false}

    <div class="drag-bar" onmousedown={dragWindow} role="presentation">

      <span class="drag-grip"></span>

    </div>

  {/if}



  {#if isMinimal && activeTab === "settings" && settings}

    <SettingsPanel

      {settings}

      {stats}

      {diskLabel}

      {paused}

      onSave={async (s) => { await api.saveSettings(s); settings = s; setLocale(s.locale); applyTheme(s.theme); await loadStats(); await loadClips(); }}

      onTogglePause={async (p) => { await api.setMonitorPaused(p); paused = p; }}

      onClose={() => switchTab("history")}

      onExport={async (path) => api.exportBackup(path)}

      onImport={async (dest, src) => api.importBackup(dest, src)}

      onMigrate={async (path) => api.migrateStoragePath(path)}

      onClearHistory={async (keepPinned) => { await api.clearHistory(keepPinned); await refreshPanelData(); }}

      onCleanup={async () => { const n = await api.runCleanup(); await refreshPanelData(); return n; }}
      onDataChanged={refreshPanelData}
      onStatsChange={applyStats}

    />

  {:else if isMinimal}

    <MinimalHistoryView

      {snippets}

      pinnedClips={pinnedClips}

      historyClips={historyOnlyClips}

      pinnedCollapseThreshold={settings?.pinned_collapse_threshold ?? 10}

      selectedClipId={selectedItem?.id ?? null}

      onPasteClip={async (item) => { await api.pasteItem(item.id, false); }}

      onPasteSnippet={async (s) => { await api.pasteSnippet(s.content); }}

      onClearHistory={clearAllHistory}

      onOpenSettings={() => switchTab("settings")}

      onClipContextMenu={(item, e) => {

        e.preventDefault();

        contextMenu = { x: e.clientX, y: e.clientY, item };

      }}

      onSaveImage={(item) => saveImageToFolder(item)}

      maskSensitive={settings?.mask_sensitive ?? true}

    />

  {:else}

  <div class="tabs" data-tauri-drag-region="false">

    <button class:active={activeTab === "history"} onclick={() => switchTab("history")}>

      {t("tabs.history")}

    </button>

    <button class:active={activeTab === "snippets"} onclick={() => switchTab("snippets")}>

      {t("tabs.snippets")}

    </button>

    <button class:active={activeTab === "settings"} onclick={() => switchTab("settings")}>

      {t("tabs.settings")}

    </button>

  </div>



  <div class="tab-viewport">

  {#if activeTab === "history"}

    <div class="tab-panel" class:from-left={tabDir < 0} class:from-right={tabDir > 0}>

    <div class="history-toolbar" data-tauri-drag-region="false">
      {#if stats}
        <span class="stats-inline">
          {t("history.stats", { total: stats.total_clips, pinned: stats.pinned_clips, images: stats.image_count, files: stats.file_count, disk: diskLabel })}
        </span>
      {/if}
      <button type="button" class="clear-btn" onclick={clearAllHistory}>{t("history.clearAll")}</button>
    </div>

    {#if nearHistoryLimit && settings}
      <p class="limit-hint">{t("history.nearLimit", { current: stats?.total_clips ?? 0, max: settings.max_history })}</p>
    {/if}



    <SearchBar

      value={searchQuery}

      placeholder={settings?.enable_smart_search ? t("history.searchSmartPlaceholder") : t("history.searchPlaceholder")}

      onInput={onSearch}

      bind:inputRef={searchInput}

    />



    {#if settings?.enable_tag_filters}

      <TagFilter {activeTags} onChange={onTagsChange} />

    {/if}



    <CategoryFilter value={category} onChange={onCategoryChange} />

    {#if multiSelectMode}

      <div class="batch-bar">

        <span>{t("history.selected", { n: selectedIds.size })}</span>

        <button onclick={batchDelete}>{t("history.batchDelete")}</button>

        <button onclick={exitMultiSelect}>{t("history.cancel")}</button>

      </div>

    {/if}



    <div class="history-body">

      <VirtualClipList

        layout={listLayout}

        {panelOpen}

        selectedRowIndex={selectedRowIndex}

        {selectedIds}

        {multiSelectMode}

        maskSensitive={settings?.mask_sensitive ?? true}

        {revealedIds}

        onSaveImage={(id) => { const item = flatClips.find((c) => c.id === id); if (item) saveImageToFolder(item); }}

        onSelect={handleItemSelect}

        onContextMenu={(rowIndex, e) => {

          e.preventDefault();

          const item = rowClip(rowIndex);

          if (item) contextMenu = { x: e.clientX, y: e.clientY, item };

        }}

        onPin={async (id) => { await api.togglePin(id); await loadClips(); }}

        onDelete={async (id) => { await trashClip(id); }}

        onPastePlain={(i) => pasteAt(i, true)}

        onPasteSegment={pasteSegment}

        onFormatPaste={async (i) => {
          const item = clipAtIndex(i);
          if (item) await pasteFormatted(item);
        }}

        onOpenPath={openPath}

        onOpenUrl={openUrl}

        onToggleReveal={toggleReveal}

      />

    </div>

    </div>

  {:else if activeTab === "snippets"}

    <div class="tab-panel" class:from-left={tabDir < 0} class:from-right={tabDir > 0}>

    <SnippetTab

      {snippets}

      selectedIndex={selectedClipIndex}

      onSelect={(i) => (selectedClipIndex = i)}

      onPaste={async (s) => { await api.pasteSnippet(s.content); }}

      onCreate={async (title, content) => { await api.createSnippet(title, content); await loadSnippets(); }}

      onUpdate={async (id, title, content) => { await api.updateSnippet(id, title, content); await loadSnippets(); }}

      onDelete={async (id) => { await api.deleteSnippet(id); await loadSnippets(); }}

      onReorder={async (ids) => { await api.reorderSnippets(ids); await loadSnippets(); }}

    />

    </div>

  {:else if settings}

    <div class="tab-panel" class:from-left={tabDir < 0} class:from-right={tabDir > 0}>

    <SettingsPanel

      {settings}

      {stats}

      {diskLabel}

      {paused}

      onSave={async (s) => { await api.saveSettings(s); settings = s; setLocale(s.locale); applyTheme(s.theme); await loadStats(); await loadClips(); }}

      onTogglePause={async (p) => { await api.setMonitorPaused(p); paused = p; }}

      onClose={() => switchTab("history")}

      onExport={async (path) => api.exportBackup(path)}

      onImport={async (dest, src) => api.importBackup(dest, src)}

      onMigrate={async (path) => api.migrateStoragePath(path)}

      onClearHistory={async (keepPinned) => { await api.clearHistory(keepPinned); await refreshPanelData(); }}

      onCleanup={async () => { const n = await api.runCleanup(); await refreshPanelData(); return n; }}
      onDataChanged={refreshPanelData}
      onStatsChange={applyStats}

    />

    </div>

  {/if}

  </div>



  {/if}



  {#if toast}

    <div class="toast">
      {toast}
      {#if undoVisible}
        <button type="button" class="undo-btn" onclick={undoTrash}>{t("history.undo")}</button>
      {/if}
    </div>

  {/if}



  {#if contextMenu}

    <ContextMenu

      x={contextMenu.x}

      y={contextMenu.y}

      item={contextMenu.item}

      onAction={(a) => handleContextAction(a, contextMenu!.item)}

      onClose={() => (contextMenu = null)}

    />

  {/if}

</div>



<style>

  .panel {

    width: 420px;

    height: 520px;

    display: flex;

    flex-direction: column;

    background: var(--bg);

    border-radius: 12px;

    border: 1px solid var(--border);

    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.18);

    overflow: hidden;

    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;

    color: var(--text);

  }



  .drag-bar {

    display: flex;

    align-items: center;

    justify-content: center;

    gap: 8px;

    padding: 6px 12px 4px;

    background: var(--bg);

    flex-shrink: 0;

    cursor: grab;

    user-select: none;

  }



  .drag-bar:active {

    cursor: grabbing;

  }



  .drag-grip {

    width: 36px;

    height: 4px;

    border-radius: 2px;

    background: var(--border);

  }



  .history-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 2px 10px 4px;
    flex-shrink: 0;
  }

  .stats-inline {
    font-size: 10px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .clear-btn {

    border: 1px solid var(--border);

    background: var(--surface);

    color: var(--text);

    font-size: 11px;

    padding: 3px 10px;

    border-radius: 6px;

    cursor: pointer;

  }



  .clear-btn:hover {

    background: var(--hover);

  }



  .toast {

    position: fixed;

    bottom: 12px;

    left: 50%;

    transform: translateX(-50%);

    z-index: 10000;

    padding: 8px 14px;

    background: var(--surface);

    border: 1px solid var(--border);

    border-radius: 8px;

    display: flex;

    align-items: center;

    gap: 10px;

    font-size: 12px;

    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);

  }



  .undo-btn {

    border: none;

    background: var(--accent);

    color: white;

    padding: 4px 10px;

    border-radius: 6px;

    font-size: 11px;

    cursor: pointer;

  }



  .limit-hint {

    margin: 0;

    padding: 0 10px 4px;

    font-size: 10px;

    color: #e65100;

    flex-shrink: 0;

  }



  .tabs {

    display: flex;

    border-bottom: 1px solid var(--border);

    background: var(--surface);

    flex-shrink: 0;

  }



  .tabs button {

    flex: 1;

    border: none;

    background: transparent;

    padding: 8px 6px;

    font-size: 13px;

    color: var(--text-muted);

    cursor: pointer;

    border-bottom: 2px solid transparent;

  }



  .tabs button.active {

    color: var(--accent);

    border-bottom-color: var(--accent);

    font-weight: 500;

  }



  .tab-viewport {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tab-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    --tab-from-x: 0px;
    animation: tab-enter 240ms linear both;
  }

  .tab-panel.from-right {
    --tab-from-x: 14px;
  }

  .tab-panel.from-left {
    --tab-from-x: -14px;
  }

  @keyframes tab-enter {
    from {
      opacity: 0;
      transform: translateX(var(--tab-from-x));
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .tab-panel {
      animation: none;
    }
  }



  .history-body {

    flex: 1;

    display: flex;

    flex-direction: column;

    min-height: 0;

  }



  .batch-bar {

    display: flex;

    align-items: center;

    gap: 8px;

    padding: 6px 12px;

    background: var(--hover);

    font-size: 12px;

    flex-shrink: 0;

  }



  .batch-bar button {

    border: none;

    background: var(--accent);

    color: white;

    padding: 2px 8px;

    border-radius: 4px;

    font-size: 11px;

    cursor: pointer;

  }



  .batch-bar button:last-child {

    background: transparent;

    color: var(--text-muted);

    border: 1px solid var(--border);

  }

</style>

