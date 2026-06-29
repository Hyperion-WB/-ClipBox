import { isJsonText } from "./formatContent";
import { isLikelyCode } from "./codeHighlight";

export type HighlightKind =
  | "url"
  | "email"
  | "ip"
  | "path"
  | "phone"
  | "otp"
  | "tracking"
  | "json"
  | "code"
  | "token";

export interface Highlight {
  kind: HighlightKind;
  value: string;
  label: string;
}

const URL_RE = /https?:\/\/[^\s<>"')\]]+/gi;
const EMAIL_RE = /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g;
const IP_RE = /\b(?:\d{1,3}\.){3}\d{1,3}\b/g;
const WIN_PATH_RE = /[A-Za-z]:\\(?:[^\\/:*?"<>|\r\n]+\\?)+/g;
const PHONE_RE = /\b1[3-9]\d{9}\b/g;
const OTP_CONTEXT_RE = /验证码[：:\s]*(\d{4,8})/gi;
const OTP_STANDALONE_RE = /(?<![0-9])(\d{6})(?![0-9])/g;
const TRACKING_PREFIX_RE =
  /\b(?:SF|YT|JD|EMS|STO|ZTO|YTO|HTKY|DBL|UC|ANE)[A-Z0-9]{8,22}\b/gi;
const TRACKING_GENERIC_RE = /\b[A-Z]{2}\d{12,14}\b/g;

function shortLabel(value: string, max = 22): string {
  const one = value.replace(/\s+/g, " ").trim();
  if (one.length <= max) return one;
  return one.slice(0, max - 1) + "…";
}

function addUnique(
  out: Highlight[],
  seen: Set<string>,
  kind: HighlightKind,
  value: string,
  label?: string,
) {
  const key = `${kind}:${value.toLowerCase()}`;
  if (seen.has(key) || value.length < 2) return;
  seen.add(key);
  out.push({ kind, value, label: label ?? shortLabel(value) });
}

export function extractHighlights(text: string, contentType?: string): Highlight[] {
  const raw = text.replace(/\s+/g, " ").trim();
  if (!raw || contentType === "image") return [];

  const out: Highlight[] = [];
  const seen = new Set<string>();

  for (const m of raw.matchAll(URL_RE)) {
    addUnique(out, seen, "url", m[0]);
  }
  for (const m of raw.matchAll(EMAIL_RE)) {
    addUnique(out, seen, "email", m[0]);
  }
  for (const m of raw.matchAll(IP_RE)) {
    addUnique(out, seen, "ip", m[0]);
  }
  if (contentType === "file" || /[A-Za-z]:\\/.test(raw) || raw.startsWith("/")) {
    for (const m of raw.matchAll(WIN_PATH_RE)) {
      addUnique(out, seen, "path", m[0]);
    }
    if (!out.some((h) => h.kind === "path") && (raw.includes("\\") || raw.includes("/"))) {
      addUnique(out, seen, "path", raw);
    }
  }
  for (const m of raw.matchAll(PHONE_RE)) {
    addUnique(out, seen, "phone", m[0]);
  }

  for (const m of raw.matchAll(OTP_CONTEXT_RE)) {
    addUnique(out, seen, "otp", m[1], `验证码 ${m[1]}`);
  }
  if (!out.some((h) => h.kind === "otp")) {
    for (const m of raw.matchAll(OTP_STANDALONE_RE)) {
      addUnique(out, seen, "otp", m[1], m[1]);
    }
  }

  for (const m of raw.matchAll(TRACKING_PREFIX_RE)) {
    addUnique(out, seen, "tracking", m[0], shortLabel(m[0], 18));
  }
  for (const m of raw.matchAll(TRACKING_GENERIC_RE)) {
    addUnique(out, seen, "tracking", m[0], shortLabel(m[0], 18));
  }

  const fullText = text.trim();
  if (isJsonText(fullText)) {
    addUnique(out, seen, "json", fullText, "JSON");
  } else if (isLikelyCode(fullText)) {
    addUnique(out, seen, "code", fullText, "代码");
  }

  return out.slice(0, 6);
}

export function clipRowHeight(highlights: Highlight[]): number {
  // Re-exported via chipLayout for row sizing; keep for backward compat.
  return highlights.length > 0 ? 60 : 44;
}

export function highlightIcon(kind: HighlightKind): string {
  switch (kind) {
    case "url":
      return "🔗";
    case "email":
      return "✉";
    case "ip":
      return "◎";
    case "path":
      return "📁";
    case "phone":
      return "☎";
    case "otp":
      return "🔢";
    case "tracking":
      return "📦";
    case "json":
      return "{ }";
    case "code":
      return "</>";
    default:
      return "·";
  }
}
