<script lang="ts">
  import { api } from "$lib/api";
  import { getCachedThumb, setCachedThumb } from "$lib/thumbCache";

  interface Props {
    clipId: number;
    alt?: string;
    large?: boolean;
  }

  let { clipId, alt = "剪贴板图片", large = false }: Props = $props();
  let root = $state<HTMLElement | null>(null);
  let src = $state<string | null>(null);
  let loading = $state(false);
  let visible = $state(false);

  $effect(() => {
    const el = root;
    if (!el) return;
    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry?.isIntersecting) visible = true;
      },
      { rootMargin: "64px" },
    );
    io.observe(el);
    return () => io.disconnect();
  });

  $effect(() => {
    if (!visible) return;
    const cached = getCachedThumb(clipId);
    if (cached) {
      src = cached;
      loading = false;
      return;
    }
    let cancelled = false;
    loading = true;
    api.getClipThumbnail(clipId).then((url) => {
      if (cancelled) return;
      if (url) setCachedThumb(clipId, url);
      src = url;
      loading = false;
    });
    return () => {
      cancelled = true;
    };
  });
</script>

<div class="thumb-wrap" class:large bind:this={root}>
  {#if src}
    <img {src} {alt} class="thumb" class:large loading="lazy" decoding="async" />
  {:else if loading}
    <span class="thumb-placeholder" class:large>…</span>
  {:else}
    <span class="thumb-placeholder" class:large>图</span>
  {/if}
</div>

<style>
  .thumb-wrap {
    flex-shrink: 0;
  }

  .thumb {
    height: 32px;
    width: 48px;
    object-fit: cover;
    border-radius: 4px;
    display: block;
  }

  .thumb-placeholder {
    width: 48px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--badge-bg);
    border-radius: 4px;
    font-size: 11px;
    color: var(--text-muted);
  }
  .thumb.large {
    height: 120px;
    width: 180px;
    object-fit: contain;
  }

  .thumb-placeholder.large {
    width: 180px;
    height: 120px;
  }
</style>
