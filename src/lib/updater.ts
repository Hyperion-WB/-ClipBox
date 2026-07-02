import { confirm } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

export type UpdateCheckResult =
  | { status: "latest" }
  | { status: "available"; update: Update }
  | { status: "error"; message: string };

export async function checkForAppUpdate(): Promise<UpdateCheckResult> {
  try {
    const update = await check();
    if (update) {
      return { status: "available", update };
    }
    return { status: "latest" };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { status: "error", message };
  }
}

export async function installAppUpdate(
  update: Update,
  onProgress?: (percent: number) => void,
): Promise<void> {
  let total = 0;
  let downloaded = 0;

  await update.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === "Started") {
      total = event.data.contentLength ?? 0;
      downloaded = 0;
      onProgress?.(0);
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      if (total > 0) {
        onProgress?.(Math.min(99, Math.round((downloaded / total) * 100)));
      }
    } else if (event.event === "Finished") {
      onProgress?.(100);
    }
  });

  await relaunch();
}

export async function confirmAndInstallUpdate(
  update: Update,
  prompt: string,
  onProgress?: (percent: number) => void,
): Promise<"installed" | "cancelled"> {
  const ok = await confirm(prompt, { title: "ClipBox", kind: "info" });
  if (!ok) return "cancelled";
  await installAppUpdate(update, onProgress);
  return "installed";
}
