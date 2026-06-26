<script lang="ts">
  interface Props {
    value: string;
    placeholder?: string;
    onInput: (value: string) => void;
    inputRef?: HTMLInputElement | null;
  }

  let {
    value,
    placeholder = "搜索剪贴板历史...",
    onInput,
    inputRef = $bindable(null),
  }: Props = $props();
</script>

<div class="search-bar">
  <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="11" cy="11" r="7" />
    <path d="M20 20l-4-4" />
  </svg>
  <input
    bind:this={inputRef}
    type="text"
    {placeholder}
    {value}
    oninput={(e) => onInput(e.currentTarget.value)}
  />
</div>

<style>
  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .icon {
    width: 16px;
    height: 16px;
    opacity: 0.5;
    flex-shrink: 0;
  }

  input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    outline: none;
  }

  input::placeholder {
    color: var(--text-muted);
  }
</style>
