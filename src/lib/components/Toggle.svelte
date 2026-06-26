<script lang="ts">
  import { t } from "$lib/i18n.svelte";
  interface Props {
    checked: boolean;
    onchange?: (checked: boolean) => void;
    disabled?: boolean;
    id?: string;
  }

  let { checked = $bindable(false), onchange, disabled = false, id }: Props = $props();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={t("settings.toggle")}
  {id}
  class="toggle"
  class:on={checked}
  class:disabled
  {disabled}
  onclick={toggle}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    width: 40px;
    height: 22px;
    border-radius: 11px;
    border: none;
    padding: 0;
    background: var(--border);
    cursor: pointer;
    position: relative;
    flex-shrink: 0;
    transition: background 0.2s ease;
  }

  .toggle.on {
    background: var(--accent);
  }

  .toggle.disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    transition: transform 0.2s ease;
  }

  .toggle.on .knob {
    transform: translateX(18px);
  }
</style>
