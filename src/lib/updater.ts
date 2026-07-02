import { api } from "$lib/api";

const RELEASES_REPO = "Hyperion-WB/-ClipBox";
export const RELEASE_PAGE = `https://github.com/${RELEASES_REPO}/releases/latest`;

export type UpdateCheckResult =
  | { status: "latest" }
  | { status: "available"; version: string; url: string }
  | { status: "error"; message: string };

function parseVersion(v: string): number[] {
  return v
    .replace(/^v/i, "")
    .split(".")
    .map((part) => parseInt(part, 10) || 0);
}

function isNewer(latest: string, current: string): boolean {
  const a = parseVersion(latest);
  const b = parseVersion(current);
  const len = Math.max(a.length, b.length);
  for (let i = 0; i < len; i++) {
    const x = a[i] ?? 0;
    const y = b[i] ?? 0;
    if (x > y) return true;
    if (x < y) return false;
  }
  return false;
}

export async function checkForAppUpdate(currentVersion: string): Promise<UpdateCheckResult> {
  try {
    const res = await fetch(
      `https://api.github.com/repos/${RELEASES_REPO}/releases/latest`,
      { headers: { Accept: "application/vnd.github+json" } },
    );
    if (!res.ok) {
      throw new Error(`GitHub API ${res.status}`);
    }
    const data = (await res.json()) as { tag_name?: string; html_url?: string };
    const tag = data.tag_name?.trim();
    if (!tag) {
      throw new Error("invalid release response");
    }
    if (isNewer(tag, currentVersion)) {
      return {
        status: "available",
        version: tag.replace(/^v/i, ""),
        url: data.html_url ?? RELEASE_PAGE,
      };
    }
    return { status: "latest" };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { status: "error", message };
  }
}

export async function openReleaseDownload(url?: string): Promise<void> {
  await api.openUrl(url ?? RELEASE_PAGE);
}
