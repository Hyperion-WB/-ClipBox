const iconCache = new Map<string, string | null>();

export function getCachedAppIcon(name: string): string | null | undefined {
  if (iconCache.has(name)) return iconCache.get(name);
  return undefined;
}

export function setCachedAppIcon(name: string, url: string | null) {
  iconCache.set(name, url);
}

export function clearSourceBadgeCache() {
  iconCache.clear();
}
