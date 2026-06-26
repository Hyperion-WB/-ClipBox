import type { ClipItem } from "./types";
import { timeBucketLabel, unknownSourceLabel, pinnedSectionLabel } from "$lib/i18n.svelte";

export type ListRow =
  | { kind: "header"; id: string; title: string }
  | { kind: "clip"; id: number; item: ClipItem; clipIndex: number };

export const HEADER_HEIGHT = 28;
export const CLIP_HEIGHT = 44;

export interface ListLayout {
  rows: ListRow[];
  heights: number[];
  offsets: number[];
  totalHeight: number;
}

interface GroupOptions {
  groupByTime: boolean;
  groupBySource: boolean;
}

function startOfDay(d: Date): Date {
  const x = new Date(d);
  x.setHours(0, 0, 0, 0);
  return x;
}

function timeBucket(date: Date, now: Date): "today" | "yesterday" | "week" | "older" {
  const today = startOfDay(now).getTime();
  const day = startOfDay(date).getTime();
  if (day === today) return "today";
  if (day === today - 86_400_000) return "yesterday";
  const weekday = (now.getDay() + 6) % 7;
  const weekStart = today - weekday * 86_400_000;
  if (day >= weekStart) return "week";
  return "older";
}

function sourceLabel(app: string | null): string {
  if (!app?.trim()) return unknownSourceLabel();
  return app;
}

function pushHeader(rows: ListRow[], heights: number[], title: string) {
  const id = `h-${title}-${rows.length}`;
  rows.push({ kind: "header", id, title });
  heights.push(HEADER_HEIGHT);
}

function pushClip(
  rows: ListRow[],
  heights: number[],
  item: ClipItem,
  clipIndex: number,
) {
  rows.push({ kind: "clip", id: item.id, item, clipIndex });
  heights.push(CLIP_HEIGHT);
}

function groupBySourceOnly(
  items: ClipItem[],
  rows: ListRow[],
  heights: number[],
  startClipIndex: number,
): number {
  let clipIndex = startClipIndex;
  const map = new Map<string, ClipItem[]>();
  for (const item of items) {
    const key = sourceLabel(item.source_app);
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(item);
  }
  const keys = [...map.keys()].sort((a, b) => a.localeCompare(b, "zh-CN"));
  for (const key of keys) {
    pushHeader(rows, heights, key);
    for (const item of map.get(key)!) {
      pushClip(rows, heights, item, clipIndex++);
    }
  }
  return clipIndex;
}

function groupByTimeOnly(
  items: ClipItem[],
  rows: ListRow[],
  heights: number[],
  startClipIndex: number,
): number {
  let clipIndex = startClipIndex;
  const now = new Date();
  const buckets: Record<string, ClipItem[]> = {
    today: [],
    yesterday: [],
    week: [],
    older: [],
  };
  for (const item of items) {
    const bucket = timeBucket(new Date(item.created_at), now);
    buckets[bucket].push(item);
  }
  for (const key of ["today", "yesterday", "week", "older"] as const) {
    const group = buckets[key];
    if (group.length === 0) continue;
    pushHeader(rows, heights, timeBucketLabel(key));
    for (const item of group) {
      pushClip(rows, heights, item, clipIndex++);
    }
  }
  return clipIndex;
}

function groupByTimeAndSource(
  items: ClipItem[],
  rows: ListRow[],
  heights: number[],
  startClipIndex: number,
): number {
  let clipIndex = startClipIndex;
  const now = new Date();
  const timeMap = new Map<string, ClipItem[]>();
  for (const item of items) {
    const bucket = timeBucket(new Date(item.created_at), now);
    if (!timeMap.has(bucket)) timeMap.set(bucket, []);
    timeMap.get(bucket)!.push(item);
  }
  for (const key of ["today", "yesterday", "week", "older"] as const) {
    const group = timeMap.get(key);
    if (!group?.length) continue;
    pushHeader(rows, heights, timeBucketLabel(key));
    clipIndex = groupBySourceOnly(group, rows, heights, clipIndex);
  }
  return clipIndex;
}

export function buildListLayout(
  clips: ClipItem[],
  opts: GroupOptions,
): ListLayout {
  const rows: ListRow[] = [];
  const heights: number[] = [];
  let clipIndex = 0;

  const pinned = clips.filter((c) => c.pinned);
  const history = clips.filter((c) => !c.pinned);

  if (pinned.length > 0) {
    if (opts.groupByTime || opts.groupBySource) {
      pushHeader(rows, heights, pinnedSectionLabel());
    }
    for (const item of pinned) {
      pushClip(rows, heights, item, clipIndex++);
    }
  }

  if (!opts.groupByTime && !opts.groupBySource) {
    for (const item of history) {
      pushClip(rows, heights, item, clipIndex++);
    }
  } else if (opts.groupByTime && opts.groupBySource) {
    clipIndex = groupByTimeAndSource(history, rows, heights, clipIndex);
  } else if (opts.groupByTime) {
    clipIndex = groupByTimeOnly(history, rows, heights, clipIndex);
  } else {
    clipIndex = groupBySourceOnly(history, rows, heights, clipIndex);
  }

  const offsets: number[] = [];
  let total = 0;
  for (const h of heights) {
    offsets.push(total);
    total += h;
  }

  return { rows, heights, offsets, totalHeight: total };
}

export function clipAtRowIndex(layout: ListLayout, rowIndex: number): ClipItem | null {
  const row = layout.rows[rowIndex];
  return row?.kind === "clip" ? row.item : null;
}

export function rowIndexForClipIndex(layout: ListLayout, clipIndex: number): number {
  return layout.rows.findIndex(
    (r) => r.kind === "clip" && r.clipIndex === clipIndex,
  );
}

export function nextClipRowIndex(layout: ListLayout, fromRow: number, dir: 1 | -1): number {
  let i = fromRow + dir;
  while (i >= 0 && i < layout.rows.length) {
    if (layout.rows[i].kind === "clip") return i;
    i += dir;
  }
  return fromRow;
}
