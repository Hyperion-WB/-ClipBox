<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { open, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { setLocale, t } from "$lib/i18n.svelte";
  import type { AppLocale, AppSettings, HistoryStats, StorageDetails } from "$lib/types";
  import { applyTheme } from "$lib/theme";
  import { checkForAppUpdate, openReleaseDownload } from "$lib/updater";
  import { maskForDisplay } from "$lib/sensitiveMask";
  import CollapsibleSection from "./CollapsibleSection.svelte";
  import HotkeyInput from "./HotkeyInput.svelte";
  import Select from "./Select.svelte";
  import SettingRow from "./SettingRow.svelte";
  import Toggle from "./Toggle.svelte";

  interface Props {
    settings: AppSettings;
    stats: HistoryStats | null;
    diskLabel: string;
    paused: boolean;
    onSave: (settings: AppSettings) => void;
    onTogglePause: (paused: boolean) => void;
    onClose: () => void;
    onExport: (path: string) => Promise<void>;
    onImport: (dest: string, src: string) => Promise<string>;
    onMigrate: (path: string) => Promise<string>;
    onClearHistory: (keepPinned: boolean) => Promise<void>;
    onCleanup: () => Promise<number>;
    onDataChanged?: () => Promise<void>;
    onStatsChange?: (stats: HistoryStats) => void | Promise<void>;
  }

  let {
    settings, stats, diskLabel, paused,
    onSave, onTogglePause, onClose,
    onExport, onImport, onMigrate, onClearHistory, onCleanup, onDataChanged, onStatsChange,
  }: Props = $props();

  let draft = $state<AppSettings>({ ...settings });
  let message = $state("");
  let recentNotifs = $state<{ id: number; preview: string }[]>([]);
  let appVersion = $state("0.1.2");
  let pendingRelease = $state<{ version: string; url: string } | null>(null);
  let updateBusy = $state(false);
  let dbLabel = $state("");
  let mediaLabel = $state("");
  let storageDetails = $state<StorageDetails | null>(null);
  let storageLoaded = $state(false);
  let storageLabels = $state({
    total: "",
    db: "",
    images: "",
    thumbs: "",
    icons: "",
    trash: "",
    orphans: "",
    largest: [] as { preview: string; size: string }[],
  });

  async function loadStorageDetails() {
    storageDetails = await api.getStorageDetails();
    storageLoaded = true;
    if (!storageDetails) return;
    const d = storageDetails;
    const [total, db, images, thumbs, icons, trash, orphans] = await Promise.all([
      api.formatDiskSize(d.total_bytes),
      api.formatDiskSize(d.db_bytes),
      api.formatDiskSize(d.images_bytes),
      api.formatDiskSize(d.thumbs_bytes),
      api.formatDiskSize(d.app_icons_bytes),
      api.formatDiskSize(d.trash_bytes),
      api.formatDiskSize(d.orphan_bytes),
    ]);
    const largest = await Promise.all(
      d.largest.map(async (item) => ({
        preview: item.preview,
        size: await api.formatDiskSize(item.bytes),
      })),
    );
    storageLabels = { total, db, images, thumbs, icons, trash, orphans, largest };
  }

  async function deepCleanStorage() {
    const n = await api.reclaimStorage();
    message = t("settings.reclaimed", { n });
    await loadStorageDetails();
    await onDataChanged?.();
  }

  async function openDataFolder() {
    await api.openDataFolder();
  }

  $effect(() => {
    if (!stats) {
      dbLabel = "";
      mediaLabel = "";
      return;
    }
    api.formatDiskSize(stats.db_bytes).then((v) => { dbLabel = v; });
    api.formatDiskSize(stats.media_bytes).then((v) => { mediaLabel = v; });
  });

  $effect(() => {
    getVersion().then((v) => { appVersion = v; }).catch(() => {});
  });

  $effect(() => { draft = { ...settings }; });

  $effect(() => {
    if (draft.enable_notifications) {
      api.getRecentNotifications().then((items) => {
        recentNotifs = items.map((c) => ({
          id: c.id,
          preview:
            c.content_type === "image"
              ? t("history.imagePreview")
              : maskForDisplay(c.content_text.slice(0, 60), draft.mask_sensitive),
        }));
      });
    } else {
      recentNotifs = [];
    }
  });

  const localeOptions = $derived([
    { value: "zh", label: t("settings.langZh") },
    { value: "en", label: t("settings.langEn") },
  ]);

  const themeOptions = $derived([
    { value: "system", label: t("settings.themeSystem") },
    { value: "light", label: t("settings.themeLight") },
    { value: "dark", label: t("settings.themeDark") },
  ]);

  const filterOptions = $derived([
    { value: "off", label: t("settings.filterOff") },
    { value: "blacklist", label: t("settings.filterBlacklist") },
    { value: "whitelist", label: t("settings.filterWhitelist") },
  ]);

  function parseAppListText(json: string): string {
    try {
      return (JSON.parse(json || "[]") as string[]).join("\n");
    } catch {
      return "";
    }
  }

  function appListToJson(text: string): string {
    return JSON.stringify(text.split("\n").map((s) => s.trim()).filter(Boolean));
  }

  let appListText = $state(parseAppListText(settings.app_filter_list));
  $effect(() => { appListText = parseAppListText(draft.app_filter_list); });

  function onLocaleChange(l: string) {
    draft.locale = l as AppLocale;
    setLocale(draft.locale);
  }

  function onThemeChange() {
    applyTheme(draft.theme);
  }

  function save() {
    draft.app_filter_list = appListToJson(appListText);
    setLocale(draft.locale);
    onSave(draft);
    message = t("settings.saved");
  }

  async function pickStorageDir() {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") draft.storage_path = selected;
  }

  async function pickImageSaveDir() {
    const selected = await open({ directory: true, multiple: false, defaultPath: draft.image_save_dir || undefined });
    if (selected && typeof selected === "string") draft.image_save_dir = selected;
  }

  async function applyStorageMigration() {
    if (!draft.storage_path) return;
    message = await onMigrate(draft.storage_path);
  }

  async function mergeDuplicates() {
    const n = await api.mergeDuplicateClips();
    message = t("settings.mergedDuplicates", { n });
  }

  async function emptyTrashBin() {
    const n = await api.emptyTrash();
    message = t("settings.emptiedTrash", { n });
    const freshStats = await api.getHistoryStats();
    await onStatsChange?.(freshStats);
    await onDataChanged?.();
    if (storageLoaded) await loadStorageDetails();
  }

  async function exportData() {
    const path = await saveDialog({ filters: [{ name: "ClipBox", extensions: ["clipbox"] }] });
    if (path) { await onExport(path); message = t("settings.exportSuccess"); }
  }

  async function importData() {
    const src = await open({ filters: [{ name: "ClipBox", extensions: ["clipbox", "zip"] }] });
    if (src && typeof src === "string") {
      message = await onImport(draft.storage_path || settings.storage_path, src);
    }
  }

  async function checkUpdate() {
    updateBusy = true;
    pendingRelease = null;
    message = t("settings.updateChecking");
    try {
      const result = await checkForAppUpdate(appVersion);
      if (result.status === "latest") {
        message = t("settings.updateLatest");
      } else if (result.status === "available") {
        pendingRelease = { version: result.version, url: result.url };
        message = t("settings.updateAvailable", { version: result.version });
      } else {
        message = t("settings.updateFailed", { error: result.message });
      }
    } catch (err) {
      message = t("settings.updateFailed", {
        error: err instanceof Error ? err.message : String(err),
      });
    } finally {
      updateBusy = false;
    }
  }

  async function openRelease() {
    if (!pendingRelease) return;
    await openReleaseDownload(pendingRelease.url);
    message = t("settings.updateOpened");
  }
</script>

<div class="settings">
  <header class="settings-header">
    <div>
      <h2>{t("settings.title")}</h2>
      {#if stats}
        <p class="stats-line">{t("settings.stats", {
          total: stats.total_clips,
          disk: diskLabel,
          db: dbLabel,
          media: mediaLabel,
          trash: stats.trash_count,
        })}</p>
      {/if}
    </div>
    <button type="button" class="icon-btn" onclick={onClose} aria-label={t("settings.close")}>×</button>
  </header>

  <div class="sections">
    <CollapsibleSection title={t("settings.sections.general")} defaultOpen={true}>
      <div class="field-block">
        <span class="field-label">{t("settings.language")}</span>
        <Select bind:value={draft.locale} options={localeOptions} onchange={onLocaleChange} />
      </div>
      <div class="field-block">
        <span class="field-label">{t("settings.hotkey")}</span>
        <HotkeyInput value={draft.hotkey} onChange={(v) => (draft.hotkey = v)} />
      </div>
      <SettingRow label={t("settings.dismissOnBlur")}>
        {#snippet control()}<Toggle bind:checked={draft.dismiss_on_blur} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.pauseMonitor")}>
        {#snippet control()}<Toggle checked={paused} onchange={onTogglePause} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.simulatePaste")}>
        {#snippet control()}<Toggle bind:checked={draft.simulate_paste} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.startOnBoot")}>
        {#snippet control()}<Toggle bind:checked={draft.start_on_boot} />{/snippet}
      </SettingRow>
    </CollapsibleSection>

    <CollapsibleSection title={t("settings.sections.interface")}>
      <div class="field-block">
        <span class="field-label">{t("settings.theme")}</span>
        <Select bind:value={draft.theme} options={themeOptions} onchange={onThemeChange} />
      </div>
      <SettingRow label={t("settings.minimalMode")} hint={t("settings.minimalHint")}>
        {#snippet control()}<Toggle bind:checked={draft.minimal_mode} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.dragBar")}>
        {#snippet control()}<Toggle bind:checked={draft.window_draggable} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.panelFollowCursor")} hint={t("settings.panelFollowCursorHint")}>
        {#snippet control()}<Toggle bind:checked={draft.panel_follow_cursor} />{/snippet}
      </SettingRow>
      <div class="field-block compact">
        <span class="field-label">{t("settings.pinnedThreshold")}</span>
        <input type="number" class="input num" min="3" max="50" bind:value={draft.pinned_collapse_threshold} />
      </div>
      <SettingRow label={t("settings.groupByTime")}>
        {#snippet control()}<Toggle bind:checked={draft.group_by_time} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.groupBySource")}>
        {#snippet control()}<Toggle bind:checked={draft.group_by_source} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.tagFilters")}>
        {#snippet control()}<Toggle bind:checked={draft.enable_tag_filters} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.smartSearch")}>
        {#snippet control()}<Toggle bind:checked={draft.enable_smart_search} />{/snippet}
      </SettingRow>
    </CollapsibleSection>

    <CollapsibleSection title={t("settings.shortcuts")}>
      <ul class="shortcut-list">
        <li>{t("settings.shortcutNav")}</li>
        <li>{t("settings.shortcutPaste")}</li>
        <li>{t("settings.shortcutPlain")}</li>
        <li>{t("settings.shortcutQuick")}</li>
        <li>{t("settings.shortcutClose")}</li>
        <li>{t("settings.shortcutSearch")}</li>
      </ul>
    </CollapsibleSection>

    <CollapsibleSection title={t("settings.sections.storage")}>
      <div class="storage-panel">
        <button type="button" class="btn-block" onclick={loadStorageDetails}>
          {storageLoaded ? t("settings.storageRefresh") : t("settings.storageView")}
        </button>
        {#if storageDetails}
          <ul class="storage-breakdown">
            <li>{t("settings.storageTotal")}: {storageLabels.total}</li>
            <li>{t("settings.storageDb")}: {storageLabels.db}</li>
            <li>{t("settings.storageImages")}: {storageLabels.images}</li>
            <li>{t("settings.storageThumbs")}: {storageLabels.thumbs}</li>
            <li>{t("settings.storageIcons")}: {storageLabels.icons}</li>
            <li>{t("settings.storageTrash")}: {storageDetails.trash_count} {t("settings.storageItems")} · {storageLabels.trash}</li>
            <li>{t("settings.storageOrphans")}: {storageDetails.orphan_count} {t("settings.storageFiles")} · {storageLabels.orphans}</li>
          </ul>
          {#if storageLabels.largest.length > 0}
            <p class="field-label">{t("settings.storageLargest")}</p>
            <ul class="largest-list">
              {#each storageLabels.largest as item, i (i)}
                <li><span class="largest-preview">{item.preview}</span><span class="largest-size">{item.size}</span></li>
              {/each}
            </ul>
          {/if}
          <button type="button" class="btn-block" onclick={deepCleanStorage}>{t("settings.storageDeepClean")}</button>
          <button type="button" class="link-btn" onclick={openDataFolder}>{t("settings.storageOpenFolder")}</button>
        {/if}
      </div>
      <SettingRow label={t("settings.compressImages")} hint={t("settings.compressImagesHint")}>
        {#snippet control()}<Toggle bind:checked={draft.compress_images} />{/snippet}
      </SettingRow>
      <div class="field-block compact">
        <span class="field-label">{t("settings.imageMaxDimension")}</span>
        <input type="number" class="input num" min="720" max="4096" step="120" bind:value={draft.image_max_dimension} />
      </div>
      <div class="field-block compact">
        <span class="field-label">{t("settings.imageJpegQuality")}</span>
        <input type="number" class="input num" min="50" max="95" bind:value={draft.image_jpeg_quality} />
      </div>
      <div class="field-block compact">
        <span class="field-label">{t("settings.imageCompressMinKb")}</span>
        <input type="number" class="input num" min="64" max="8192" step="64" bind:value={draft.image_compress_min_kb} />
      </div>
      <div class="field-block">
        <span class="field-label">{t("settings.imageSaveDir")}</span>
        <div class="path-row">
          <input type="text" class="input" value={draft.image_save_dir} readonly placeholder={t("settings.imageSavePlaceholder")} />
          <button type="button" class="btn-sm" onclick={pickImageSaveDir}>{t("settings.browse")}</button>
        </div>
        <p class="field-hint">{t("settings.imageSaveHint")}</p>
      </div>
      <div class="field-block compact">
        <span class="field-label">{t("settings.retentionDays")}</span>
        <input type="number" class="input num" min="0" max="365" bind:value={draft.retention_days} />
      </div>
      <div class="field-block compact">
        <span class="field-label">{t("settings.imageRetentionDays")}</span>
        <input type="number" class="input num" min="0" max="365" bind:value={draft.image_retention_days} />
      </div>
      <div class="field-block compact">
        <span class="field-label">{t("settings.maxHistory")}</span>
        <input type="number" class="input num" min="50" max="2000" bind:value={draft.max_history} />
      </div>
      <div class="field-block compact">
        <span class="field-label">{t("settings.trashRetention")}</span>
        <input type="number" class="input num" min="1" max="168" bind:value={draft.trash_retention_hours} />
      </div>
      <SettingRow label={t("settings.autoCleanup")}>
        {#snippet control()}<Toggle bind:checked={draft.auto_cleanup} />{/snippet}
      </SettingRow>
      <div class="field-block">
        <span class="field-label">{t("settings.storagePath")}</span>
        <div class="path-row">
          <input type="text" class="input" bind:value={draft.storage_path} readonly placeholder={t("settings.storageDefault")} />
          <button type="button" class="btn-sm" onclick={pickStorageDir}>{t("settings.browse")}</button>
        </div>
        <button type="button" class="link-btn" onclick={applyStorageMigration}>{t("settings.migrate")}</button>
      </div>
      <button type="button" class="btn-block" onclick={async () => { message = t("settings.cleaned", { n: await onCleanup() }); }}>
        {t("settings.cleanupNow")}
      </button>
    </CollapsibleSection>

    <CollapsibleSection title={t("settings.sections.privacy")}>
      <SettingRow label={t("settings.maskSensitive")} hint={t("settings.maskSensitiveHint")}>
        {#snippet control()}<Toggle bind:checked={draft.mask_sensitive} />{/snippet}
      </SettingRow>
      <div class="field-block">
        <span class="field-label">{t("settings.appFilterMode")}</span>
        <Select bind:value={draft.app_filter_mode} options={filterOptions} />
      </div>
      {#if draft.app_filter_mode !== "off"}
        <div class="field-block">
          <span class="field-label">{t("settings.appList")}</span>
          <textarea class="textarea" bind:value={appListText} rows="3" placeholder={t("settings.appListPlaceholder")}></textarea>
        </div>
      {/if}
      <SettingRow label={t("settings.notifications")}>
        {#snippet control()}<Toggle bind:checked={draft.enable_notifications} />{/snippet}
      </SettingRow>
      {#if draft.enable_notifications && recentNotifs.length > 0}
        <div class="notif-list">
          {#each recentNotifs as n (n.id)}
            <div class="notif-item">{n.preview}</div>
          {/each}
        </div>
      {/if}
      <div class="field-block compact">
        <span class="field-label">{t("settings.pollInterval")}</span>
        <input type="number" class="input num" min="400" max="3000" step="100" bind:value={draft.poll_interval_ms} />
      </div>
      <SettingRow label={t("settings.releaseMemory")}>
        {#snippet control()}<Toggle bind:checked={draft.release_memory_on_close} />{/snippet}
      </SettingRow>
      <SettingRow label={t("settings.dedupe")} hint={t("settings.dedupeHint")}>
        {#snippet control()}<Toggle bind:checked={draft.dedupe} />{/snippet}
      </SettingRow>
    </CollapsibleSection>

    <CollapsibleSection title={t("settings.updateTitle")}>
      <p class="field-hint">{t("settings.versionLine", { version: appVersion })}</p>
      <p class="field-hint">{t("settings.updateManualHint")}</p>
      <button type="button" class="btn-block" disabled={updateBusy} onclick={checkUpdate}>
        {updateBusy ? t("settings.updateChecking") : t("settings.updateCheck")}
      </button>
      {#if pendingRelease}
        <button type="button" class="btn-block accent" onclick={openRelease}>
          {t("settings.updateOpenRelease")} (v{pendingRelease.version})
        </button>
      {/if}
    </CollapsibleSection>

    <CollapsibleSection title={t("settings.sections.data")}>
      {#if stats && stats.trash_count > 0}
        <p class="field-hint">{t("settings.trashCount", { n: stats.trash_count })}</p>
        <button type="button" class="btn-block" onclick={emptyTrashBin}>{t("settings.emptyTrash")}</button>
      {/if}
      <button type="button" class="btn-block" onclick={mergeDuplicates}>{t("settings.mergeDuplicates")}</button>
      <button type="button" class="btn-block" onclick={exportData}>{t("settings.export")}</button>
      <button type="button" class="btn-block" onclick={importData}>{t("settings.import")}</button>
      <button type="button" class="btn-block" onclick={() => onClearHistory(true)}>{t("settings.clearKeepPinned")}</button>
      <button type="button" class="btn-block danger" onclick={() => onClearHistory(false)}>{t("settings.clearAll")}</button>
    </CollapsibleSection>
  </div>

  <footer class="settings-footer">
    {#if message}<p class="message">{message}</p>{/if}
    <button type="button" class="save-btn" onclick={save}>{t("settings.save")}</button>
  </footer>
</div>

<style>
  .settings { flex: 1; display: flex; flex-direction: column; min-height: 0; background: var(--bg); }
  .settings-header { display: flex; justify-content: space-between; align-items: flex-start; padding: 14px 16px 10px; flex-shrink: 0; }
  .settings-header h2 { margin: 0; font-size: 17px; font-weight: 600; }
  .stats-line { margin: 4px 0 0; font-size: 11px; color: var(--text-muted); }
  .icon-btn { border: none; background: var(--surface); color: var(--text-muted); width: 32px; height: 32px; border-radius: 8px; font-size: 18px; cursor: pointer; line-height: 1; }
  .icon-btn:hover { background: var(--hover); color: var(--text); }
  .sections { flex: 1; overflow-y: auto; padding-bottom: 8px; }
  .field-block { padding: 10px 0 4px; }
  .field-block.compact { padding: 8px 0 2px; }
  .field-label { display: block; font-size: 12px; color: var(--text-muted); margin-bottom: 6px; }
  .field-hint { margin: 6px 0 0; font-size: 11px; color: var(--text-muted); line-height: 1.4; }
  .input, .textarea { width: 100%; border: 1px solid var(--border); border-radius: 8px; padding: 8px 10px; font-size: 13px; background: var(--bg); color: var(--text); box-sizing: border-box; }
  .input.num { max-width: 120px; }
  .textarea { resize: vertical; font-family: inherit; min-height: 72px; }
  .path-row { display: flex; gap: 8px; }
  .path-row .input { flex: 1; min-width: 0; }
  .btn-sm { border: 1px solid var(--border); background: var(--bg); color: var(--text); padding: 0 12px; border-radius: 8px; font-size: 12px; cursor: pointer; white-space: nowrap; }
  .btn-sm:hover { background: var(--hover); }
  .btn-block { width: 100%; border: 1px solid var(--border); background: var(--bg); color: var(--text); padding: 9px; border-radius: 8px; font-size: 13px; cursor: pointer; margin-top: 8px; }
  .btn-block:hover { background: var(--hover); }
  .btn-block.danger { color: #e53935; border-color: #ffcdd2; }
  .btn-block.accent { background: var(--accent); color: white; border-color: var(--accent); }
  .btn-block.accent:hover { filter: brightness(1.05); }
  .btn-block:disabled { opacity: 0.55; cursor: not-allowed; }
  .link-btn { border: none; background: transparent; color: var(--accent); font-size: 12px; padding: 6px 0 0; cursor: pointer; }
  .notif-list { margin-top: 8px; }
  .notif-item { padding: 8px 10px; margin-top: 4px; background: var(--hover); border-radius: 8px; font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .shortcut-list { margin: 4px 0 0; padding: 0 0 0 18px; font-size: 12px; color: var(--text-muted); line-height: 1.7; }
  .storage-panel { margin-bottom: 8px; }
  .storage-breakdown, .largest-list { margin: 8px 0 0; padding: 0 0 0 18px; font-size: 12px; color: var(--text-muted); line-height: 1.65; }
  .largest-list { list-style: none; padding: 0; margin-top: 4px; }
  .largest-list li { display: flex; justify-content: space-between; gap: 8px; padding: 4px 0; border-bottom: 1px solid var(--border); }
  .largest-preview { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .largest-size { flex-shrink: 0; color: var(--text); font-variant-numeric: tabular-nums; }
  .settings-footer { padding: 10px 16px 14px; border-top: 1px solid var(--border); background: var(--surface); flex-shrink: 0; }
  .message { margin: 0 0 8px; font-size: 12px; color: var(--accent); text-align: center; }
  .save-btn { width: 100%; border: none; background: var(--accent); color: white; padding: 10px; border-radius: 10px; font-size: 14px; font-weight: 500; cursor: pointer; }
  .save-btn:hover { filter: brightness(1.05); }
</style>
