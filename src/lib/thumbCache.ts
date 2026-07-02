const cache = new Map<number, string>();
const MAX = 36;

export function getCachedThumb(id: number): string | undefined {
  const hit = cache.get(id);
  if (hit === undefined) return undefined;
  // LRU touch
  cache.delete(id);
  cache.set(id, hit);
  return hit;
}

export function setCachedThumb(id: number, url: string) {
  if (cache.has(id)) cache.delete(id);
  while (cache.size >= MAX) {
    const oldest = cache.keys().next().value;
    if (oldest === undefined) break;
    cache.delete(oldest);
  }
  cache.set(id, url);
}

export function clearThumbCache() {
  cache.clear();
}
