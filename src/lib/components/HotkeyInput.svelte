<script lang="ts">
  import { t } from "$lib/i18n.svelte";
  interface Props {
    value: string;
    onChange: (value: string) => void;
  }

  let { value, onChange }: Props = $props();
  let recording = $state(false);

  function capture(e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();

    if (e.key === "Escape") {
      recording = false;
      return;
    }

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    if (e.metaKey) parts.push("Win");

    const key = normalizeKey(e.key);
    if (!["Control", "Shift", "Alt", "Meta"].includes(e.key) && key) {
      parts.push(key);
      onChange(parts.join("+"));
      recording = false;
    }
  }

  function normalizeKey(key: string): string {
    if (key.length === 1) return key.toUpperCase();
    if (key.startsWith("Arrow")) return key.replace("Arrow", "");
    if (key === " ") return "Space";
    return key;
  }
</script>

<div class="hotkey-input">
  <input
    type="text"
    readonly
    value={recording ? t("hotkey.recording") : value}
    class:recording
    onclick={() => (recording = true)}
    onkeydown={capture}
    onblur={() => (recording = false)}
    placeholder={t("hotkey.placeholder")}
  />
  {#if recording}
    <span class="hint">{t("hotkey.escCancel")}</span>
  {/if}
</div>

<style>
  .hotkey-input input {
    width: 100%;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 13px;
    background: var(--surface);
    color: var(--text);
    box-sizing: border-box;
    cursor: pointer;
  }

  .hotkey-input input.recording {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(0, 129, 255, 0.15);
  }

  .hint {
    font-size: 11px;
    color: var(--accent);
    margin-top: 4px;
    display: block;
  }
</style>
