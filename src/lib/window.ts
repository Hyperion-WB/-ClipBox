import { getCurrentWindow } from "@tauri-apps/api/window";

/** Programmatic window drag — avoids blur-dismiss closing the panel on click. */
export async function dragWindow(e: MouseEvent) {
  if (e.button !== 0) return;
  e.preventDefault();
  try {
    await getCurrentWindow().startDragging();
  } catch {
    // non-Tauri dev fallback
  }
}
