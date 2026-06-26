<script lang="ts">
  import HistoryPanel from "$lib/components/HistoryPanel.svelte";
</script>

<HistoryPanel />

<style>
  :global(:root) {
    --bg: rgba(255, 255, 255, 0.96);
    --surface: #f8f9fa;
    --text: #1a1a1a;
    --text-muted: #888;
    --border: #e8e8e8;
    --hover: rgba(0, 0, 0, 0.04);
    --hover-strong: rgba(0, 0, 0, 0.08);
    --accent: #0081ff;
    --badge-bg: #f0f0f0;
    --badge-text: #666;
  }

  :global(:root.theme-dark) {
    --bg: rgba(40, 40, 40, 0.96);
    --surface: #333;
    --text: #f0f0f0;
    --text-muted: #999;
    --border: #444;
    --hover: rgba(255, 255, 255, 0.06);
    --hover-strong: rgba(255, 255, 255, 0.1);
    --accent: #4dabf7;
    --badge-bg: #444;
    --badge-text: #ccc;
  }

  :global(:root.theme-light) {
    --bg: rgba(255, 255, 255, 0.96);
    --surface: #f8f9fa;
    --text: #1a1a1a;
    --text-muted: #888;
    --border: #e8e8e8;
    --hover: rgba(0, 0, 0, 0.04);
    --hover-strong: rgba(0, 0, 0, 0.08);
    --accent: #0081ff;
    --badge-bg: #f0f0f0;
    --badge-text: #666;
  }

  @media (prefers-color-scheme: dark) {
    :global(:root:not(.theme-light):not(.theme-dark)) {
      --bg: rgba(40, 40, 40, 0.96);
      --surface: #333;
      --text: #f0f0f0;
      --text-muted: #999;
      --border: #444;
      --hover: rgba(255, 255, 255, 0.06);
      --hover-strong: rgba(255, 255, 255, 0.1);
      --accent: #4dabf7;
      --badge-bg: #444;
      --badge-text: #ccc;
    }
  }

  :global(html) {
    color-scheme: light dark;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
  }
</style>
