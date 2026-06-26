<script lang="ts">
  import { api } from "$lib/api";
  import { highlightCode, isLikelyCode } from "$lib/codeHighlight";
  import type { ClipItem } from "$lib/types";

  interface Props {
    item: ClipItem | null;
  }

  let { item }: Props = $props();

  let imageUrl = $state<string | null>(null);
  let loadingImage = $state(false);

  $effect(() => {
    const clip = item;
    imageUrl = null;
    if (!clip || clip.content_type !== "image" || !clip.has_blob) return;

    loadingImage = true;
    api.getClipImage(clip.id).then((url) => {
      if (item?.id === clip.id) imageUrl = url;
    }).finally(() => {
      if (item?.id === clip.id) loadingImage = false;
    });
  });

  const isUrl = $derived(
    item
      ? /^https?:\/\//i.test(item.content_text.trim())
      || /^www\./i.test(item.content_text.trim())
      : false,
  );

  const showCode = $derived(
    item
      ? item.content_type === "text" && (isLikelyCode(item.content_text) || false)
      : false,
  );
</script>

{#if item}
  <div class="preview">
    <div class="preview-header">
      <span class="type">{item.content_type}</span>
      {#if item.source_app}
        <span class="source">{item.source_app}</span>
      {/if}
      <span class="time">{new Date(item.created_at).toLocaleString("zh-CN")}</span>
    </div>

    <div class="preview-body">
      {#if item.content_type === "image"}
        {#if loadingImage}
          <div class="placeholder">加载图片…</div>
        {:else if imageUrl}
          <img src={imageUrl} alt="剪贴板图片预览" class="preview-image" />
        {:else}
          <div class="placeholder">无图片数据</div>
        {/if}
      {:else if isUrl}
        <a class="preview-link" href={item.content_text.trim()} target="_blank" rel="noreferrer">
          {item.content_text.trim()}
        </a>
      {:else if showCode}
        <pre class="code"><code>{@html highlightCode(item.content_text)}</code></pre>
      {:else}
        <pre class="plain">{item.content_text}</pre>
      {/if}
    </div>
  </div>
{/if}

<style>
  .preview {
    border-top: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
    max-height: 160px;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .preview-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 10px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
  }

  .type {
    text-transform: uppercase;
    font-weight: 600;
    color: var(--accent);
  }

  .source { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .preview-body {
    flex: 1;
    overflow: auto;
    padding: 8px 10px;
    min-height: 0;
  }

  .preview-image {
    max-width: 100%;
    max-height: 120px;
    object-fit: contain;
    display: block;
    margin: 0 auto;
    border-radius: 6px;
  }

  .placeholder {
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
    padding: 16px;
  }

  .preview-link {
    font-size: 12px;
    color: var(--accent);
    word-break: break-all;
  }

  .plain, .code {
    margin: 0;
    font-size: 11px;
    line-height: 1.45;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: "Cascadia Code", "Fira Code", Consolas, monospace;
  }

  .code :global(.hl-keyword) { color: #7c4dff; }
  .code :global(.hl-string) { color: #2e7d32; }
  .code :global(.hl-number) { color: #f57c00; }
  .code :global(.hl-comment) { color: var(--text-muted); }
</style>
