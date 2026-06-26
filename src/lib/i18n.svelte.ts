import { en } from "./locales/en";
import { zh, type Messages } from "./locales/zh";

export type Locale = "zh" | "en";

const dicts: Record<Locale, Messages> = { zh, en };

const i18nState = $state({ locale: "zh" as Locale });

export function getLocale(): Locale {
  return i18nState.locale;
}

function resolve(obj: Messages, path: string): string | undefined {
  let cur: unknown = obj;
  for (const part of path.split(".")) {
    if (cur == null || typeof cur !== "object") return undefined;
    cur = (cur as Record<string, unknown>)[part];
  }
  return typeof cur === "string" ? cur : undefined;
}

export function t(
  key: string,
  vars?: Record<string, string | number>,
): string {
  const loc = i18nState.locale;
  let text = resolve(dicts[loc], key) ?? resolve(dicts.zh, key) ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replaceAll(`{${k}}`, String(v));
    }
  }
  return text;
}

export function setLocale(l: Locale) {
  i18nState.locale = l;
  if (typeof document !== "undefined") {
    document.documentElement.lang = l === "zh" ? "zh-CN" : "en";
  }
}

export function detectLocale(): Locale {
  if (typeof navigator === "undefined") return "zh";
  return navigator.language.toLowerCase().startsWith("zh") ? "zh" : "en";
}

export function timeBucketLabel(bucket: string): string {
  const map: Record<string, string> = {
    today: t("history.timeToday"),
    yesterday: t("history.timeYesterday"),
    week: t("history.timeWeek"),
    older: t("history.timeOlder"),
  };
  return map[bucket] ?? bucket;
}

export function unknownSourceLabel(): string {
  return t("history.unknownSource");
}

export function pinnedSectionLabel(): string {
  return t("history.pinnedSection");
}
