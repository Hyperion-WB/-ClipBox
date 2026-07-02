const iconCache = new Map<string, string | null>();
const MAX = 32;

export function getCachedAppIcon(name: string): string | null | undefined {
  if (!iconCache.has(name)) return undefined;
  const hit = iconCache.get(name)!;
  iconCache.delete(name);
  iconCache.set(name, hit);
  return hit;
}

export function setCachedAppIcon(name: string, url: string | null) {
  if (iconCache.has(name)) iconCache.delete(name);
  while (iconCache.size >= MAX) {
    const oldest = iconCache.keys().next().value;
    if (oldest === undefined) break;
    iconCache.delete(oldest);
  }
  iconCache.set(name, url);
}

export function clearSourceBadgeCache() {
  iconCache.clear();
}
