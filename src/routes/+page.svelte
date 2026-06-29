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
    --chip-url-bg: #e3f2fd;
    --chip-url-bg-hover: #bbdefb;
    --chip-url-fg: #1565c0;
    --chip-url-border: rgba(21, 101, 192, 0.15);
    --chip-path-bg: #f3e5f5;
    --chip-path-bg-hover: #e1bee7;
    --chip-path-fg: #6a1b9a;
    --chip-path-border: rgba(106, 27, 154, 0.15);
    --chip-otp-bg: #fff3e0;
    --chip-otp-bg-hover: #ffe0b2;
    --chip-otp-fg: #e65100;
    --chip-otp-border: rgba(230, 81, 0, 0.15);
    --chip-tracking-bg: #e8f5e9;
    --chip-tracking-bg-hover: #c8e6c9;
    --chip-tracking-fg: #2e7d32;
    --chip-tracking-border: rgba(46, 125, 50, 0.15);
    --chip-code-bg: #eceff1;
    --chip-code-bg-hover: #cfd8dc;
    --chip-code-fg: #37474f;
    --chip-code-border: rgba(55, 71, 79, 0.12);
    --avatar-bg: hsl(var(--hue, 200) 55% 90%);
    --avatar-fg: hsl(var(--hue, 200) 45% 35%);
    --avatar-border: rgba(0, 0, 0, 0.06);
  }

  :global(:root.theme-dark) {
    --bg: rgba(40, 40, 40, 0.96);
    --surface: #333;
    --text: #f0f0f0;
    --text-muted: #aaa;
    --border: #444;
    --hover: rgba(255, 255, 255, 0.06);
    --hover-strong: rgba(255, 255, 255, 0.1);
    --accent: #5eb3ff;
    --badge-bg: #444;
    --badge-text: #ddd;
    --chip-url-bg: rgba(77, 171, 247, 0.22);
    --chip-url-bg-hover: rgba(77, 171, 247, 0.34);
    --chip-url-fg: #90caf9;
    --chip-url-border: rgba(144, 202, 249, 0.25);
    --chip-path-bg: rgba(186, 104, 200, 0.2);
    --chip-path-bg-hover: rgba(186, 104, 200, 0.32);
    --chip-path-fg: #ce93d8;
    --chip-path-border: rgba(206, 147, 216, 0.22);
    --chip-otp-bg: rgba(255, 183, 77, 0.2);
    --chip-otp-bg-hover: rgba(255, 183, 77, 0.32);
    --chip-otp-fg: #ffcc80;
    --chip-otp-border: rgba(255, 204, 128, 0.22);
    --chip-tracking-bg: rgba(129, 199, 132, 0.2);
    --chip-tracking-bg-hover: rgba(129, 199, 132, 0.32);
    --chip-tracking-fg: #a5d6a7;
    --chip-tracking-border: rgba(165, 214, 167, 0.22);
    --chip-code-bg: rgba(176, 190, 197, 0.18);
    --chip-code-bg-hover: rgba(176, 190, 197, 0.28);
    --chip-code-fg: #cfd8dc;
    --chip-code-border: rgba(207, 216, 220, 0.18);
    --avatar-bg: hsl(var(--hue, 200) 35% 28%);
    --avatar-fg: hsl(var(--hue, 200) 70% 82%);
    --avatar-border: rgba(255, 255, 255, 0.12);
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
    --chip-url-bg: #e3f2fd;
    --chip-url-bg-hover: #bbdefb;
    --chip-url-fg: #1565c0;
    --chip-url-border: rgba(21, 101, 192, 0.15);
    --chip-path-bg: #f3e5f5;
    --chip-path-bg-hover: #e1bee7;
    --chip-path-fg: #6a1b9a;
    --chip-path-border: rgba(106, 27, 154, 0.15);
    --chip-otp-bg: #fff3e0;
    --chip-otp-bg-hover: #ffe0b2;
    --chip-otp-fg: #e65100;
    --chip-otp-border: rgba(230, 81, 0, 0.15);
    --chip-tracking-bg: #e8f5e9;
    --chip-tracking-bg-hover: #c8e6c9;
    --chip-tracking-fg: #2e7d32;
    --chip-tracking-border: rgba(46, 125, 50, 0.15);
    --chip-code-bg: #eceff1;
    --chip-code-bg-hover: #cfd8dc;
    --chip-code-fg: #37474f;
    --chip-code-border: rgba(55, 71, 79, 0.12);
    --avatar-bg: hsl(var(--hue, 200) 55% 90%);
    --avatar-fg: hsl(var(--hue, 200) 45% 35%);
    --avatar-border: rgba(0, 0, 0, 0.06);
  }

  @media (prefers-color-scheme: dark) {
    :global(:root:not(.theme-light):not(.theme-dark)) {
      --bg: rgba(40, 40, 40, 0.96);
      --surface: #333;
      --text: #f0f0f0;
      --text-muted: #aaa;
      --border: #444;
      --hover: rgba(255, 255, 255, 0.06);
      --hover-strong: rgba(255, 255, 255, 0.1);
      --accent: #5eb3ff;
      --badge-bg: #444;
      --badge-text: #ddd;
      --chip-url-bg: rgba(77, 171, 247, 0.22);
      --chip-url-bg-hover: rgba(77, 171, 247, 0.34);
      --chip-url-fg: #90caf9;
      --chip-url-border: rgba(144, 202, 249, 0.25);
      --chip-path-bg: rgba(186, 104, 200, 0.2);
      --chip-path-bg-hover: rgba(186, 104, 200, 0.32);
      --chip-path-fg: #ce93d8;
      --chip-path-border: rgba(206, 147, 216, 0.22);
      --chip-otp-bg: rgba(255, 183, 77, 0.2);
      --chip-otp-bg-hover: rgba(255, 183, 77, 0.32);
      --chip-otp-fg: #ffcc80;
      --chip-otp-border: rgba(255, 204, 128, 0.22);
      --chip-tracking-bg: rgba(129, 199, 132, 0.2);
      --chip-tracking-bg-hover: rgba(129, 199, 132, 0.32);
      --chip-tracking-fg: #a5d6a7;
      --chip-tracking-border: rgba(165, 214, 167, 0.22);
      --chip-code-bg: rgba(176, 190, 197, 0.18);
      --chip-code-bg-hover: rgba(176, 190, 197, 0.28);
      --chip-code-fg: #cfd8dc;
      --chip-code-border: rgba(207, 216, 220, 0.18);
      --avatar-bg: hsl(var(--hue, 200) 35% 28%);
      --avatar-fg: hsl(var(--hue, 200) 70% 82%);
      --avatar-border: rgba(255, 255, 255, 0.12);
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

  :global(*) {
    -webkit-font-smoothing: antialiased;
  }

  :global(.virtual-list) {
    scroll-behavior: auto;
    overscroll-behavior: contain;
    -webkit-overflow-scrolling: touch;
  }

  :global(.virtual-list .window) {
    transform: translateZ(0);
  }
</style>
