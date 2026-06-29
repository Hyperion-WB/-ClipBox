<script lang="ts">
  import { open, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { setLocale, t } from "$lib/i18n.svelte";
  import type { AppLocale, AppSettings, HistoryStats } from "$lib/types";
  import { applyTheme } from "$lib/theme";
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
  }

  let {
    settings, stats, diskLabel, paused,
    onSave, onTogglePause, onClose,
    onExport, onImport, onMigrate, onClearHistory, onCleanup,
  }: Props = $props();

  let draft = $state<AppSettings>({ ...settings });
  let message = $state("");
  let recentNotifs = $state<{ id: number; preview: string }[]>([]);

  $effect(() => { draft = { ...settings }; });

  $effect(() => {
    if (draft.enable_notifications) {
      api.getRecentNotifications().then((items) => {
        recentNotifs = items.map((c) => ({
          id: c.id,
          preview: c.content_type === "image" ? t("history.imagePreview") : c.content_text.slice(0, 60),
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
</script>

<div class="settings">
  <header class="settings-header">
    <div>
      <h2>{t("settings.title")}</h2>
      {#if stats}
        <p class="stats-line">{t("settings.stats", { total: stats.total_clips, disk: diskLabel })}</p>
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

    <CollapsibleSection title={t("settings.sections.storage")}>
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

    <CollapsibleSection title={t("settings.sections.data")}>
      <button type="button" class="btn-block" onclick={exportData}>{t("settings.export")}</button>
      <button type="button" class="btn-block" onclick={importData}>{t("settings.import")}</button>
      <button type="button" class="btn-block" onclick={() => onClearHistory(true)}>{t("settings.clearKeepPinned")}</button>
      <button type="button" class="btn-block danger" onclick={() => onClearHistory(false)}>{t("settings.clearAll")}</button>
      <p class="version">{t("settings.version")}</p>
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
  .link-btn { border: none; background: transparent; color: var(--accent); font-size: 12px; padding: 6px 0 0; cursor: pointer; }
  .notif-list { margin-top: 8px; }
  .notif-item { padding: 8px 10px; margin-top: 4px; background: var(--hover); border-radius: 8px; font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .version { font-size: 11px; color: var(--text-muted); margin: 12px 0 0; text-align: center; }
  .settings-footer { padding: 10px 16px 14px; border-top: 1px solid var(--border); background: var(--surface); flex-shrink: 0; }
  .message { margin: 0 0 8px; font-size: 12px; color: var(--accent); text-align: center; }
  .save-btn { width: 100%; border: none; background: var(--accent); color: white; padding: 10px; border-radius: 10px; font-size: 14px; font-weight: 500; cursor: pointer; }
  .save-btn:hover { filter: brightness(1.05); }
</style>
