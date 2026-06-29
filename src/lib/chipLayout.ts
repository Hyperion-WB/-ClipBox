import type { Highlight, HighlightKind } from "./extractHighlights";

export function isLinkChip(kind: HighlightKind): boolean {
  return kind === "url" || kind === "path";
}

export function partitionHighlights(items: Highlight[]) {
  const links: Highlight[] = [];
  const extras: Highlight[] = [];
  for (const h of items) {
    if (isLinkChip(h.kind)) links.push(h);
    else extras.push(h);
  }
  return { links, extras };
}

/**
 * Hide optional chips when they would get in the way of copying the main text.
 * Links/paths always stay visible.
 */
export function shouldCollapseChips(items: Highlight[]): boolean {
  if (items.length <= 1) return false;
  const { links, extras } = partitionHighlights(items);
  if (extras.length === 0) return false;
  if (links.length > 0) return true;
  return extras.length > 1;
}

export function clipRowHeight(highlights: Highlight[]): number {
  if (highlights.length === 0) return 44;
  if (shouldCollapseChips(highlights)) {
    const { links } = partitionHighlights(highlights);
    return links.length > 0 ? 56 : 50;
  }
  return highlights.length > 2 ? 64 : 56;
}
