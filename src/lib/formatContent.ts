import type { Highlight } from "./extractHighlights";
import { isLikelyCode } from "./codeHighlight";

export function isJsonText(text: string): boolean {
  const t = text.trim();
  if (!(t.startsWith("{") || t.startsWith("["))) return false;
  try {
    JSON.parse(t);
    return true;
  } catch {
    return false;
  }
}

export function formatJson(text: string): string {
  const parsed = JSON.parse(text.trim());
  return JSON.stringify(parsed, null, 2);
}

/** Normalize indentation and trailing whitespace for code-like text. */
export function formatCode(text: string): string {
  const t = text.replace(/\r\n/g, "\n").trimEnd();
  const lines = t.split("\n");
  const indents = lines
    .filter((l) => l.trim().length > 0)
    .map((l) => l.match(/^(\s*)/)?.[1].length ?? 0);
  const min = indents.length ? Math.min(...indents) : 0;
  if (min > 0) {
    return lines.map((l) => (l.length >= min ? l.slice(min) : l)).join("\n").trimEnd();
  }
  return t;
}

export function formatHighlight(h: Highlight): string {
  switch (h.kind) {
    case "json":
      return formatJson(h.value);
    case "code":
      if (isJsonText(h.value)) return formatJson(h.value);
      return formatCode(h.value);
    default:
      return h.value;
  }
}

export function itemHasFormatAction(text: string, contentType?: string): boolean {
  if (contentType === "image") return false;
  const t = text.trim();
  return isJsonText(t) || isLikelyCode(t);
}
