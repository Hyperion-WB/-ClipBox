<script lang="ts">

  import { api } from "$lib/api";

  import {

    clearSourceBadgeCache,

    getCachedAppIcon,

    setCachedAppIcon,

  } from "$lib/sourceBadgeCache";



  interface Props {

    name: string;

  }



  let { name }: Props = $props();



  let iconUrl = $state<string | null>(null);



  const initial = $derived(name.trim().charAt(0).toUpperCase() || "?");

  const hue = $derived(

    [...name].reduce((acc, c) => acc + c.charCodeAt(0), 0) % 360,

  );



  $effect(() => {

    const n = name.trim();

    if (!n) return;



    const cached = getCachedAppIcon(n);

    if (cached !== undefined) {

      iconUrl = cached;

      return;

    }



    let cancelled = false;

    api.getAppIcon(n).then((url) => {

      if (cancelled) return;

      setCachedAppIcon(n, url);

      iconUrl = url;

    });

    return () => {

      cancelled = true;

    };

  });

</script>



<span class="source-badge" title={name}>

  {#if iconUrl}

    <img class="icon" src={iconUrl} alt="" />

  {:else}

    <span class="avatar" style="--hue: {hue}">{initial}</span>

  {/if}

  <span class="label">{name}</span>

</span>



<style>

  .source-badge {

    display: inline-flex;

    align-items: center;

    gap: 4px;

    max-width: 100%;

    min-width: 0;

  }



  .icon {

    width: 14px;

    height: 14px;

    border-radius: 3px;

    flex-shrink: 0;

    object-fit: contain;

    border: 1px solid var(--avatar-border, transparent);

  }



  .avatar {

    width: 14px;

    height: 14px;

    border-radius: 3px;

    flex-shrink: 0;

    display: flex;

    align-items: center;

    justify-content: center;

    font-size: 8px;

    font-weight: 600;

    color: var(--avatar-fg, hsl(var(--hue) 45% 35%));

    background: var(--avatar-bg, hsl(var(--hue) 55% 90%));

    border: 1px solid var(--avatar-border, transparent);

  }



  .label {

    font-size: 10px;

    color: var(--text-muted);

    overflow: hidden;

    text-overflow: ellipsis;

    white-space: nowrap;

    min-width: 0;

  }

</style>

