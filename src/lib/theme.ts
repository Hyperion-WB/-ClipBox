import type { ThemeMode } from "./types";

export function applyTheme(theme: ThemeMode) {
  const root = document.documentElement;
  root.classList.remove("theme-light", "theme-dark");

  if (theme === "light") {
    root.classList.add("theme-light");
    root.style.colorScheme = "light";
  } else if (theme === "dark") {
    root.classList.add("theme-dark");
    root.style.colorScheme = "dark";
  } else {
    root.style.colorScheme = "";
    const dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (dark) root.classList.add("theme-dark");
    else root.classList.add("theme-light");
  }
}

export function watchSystemTheme(onChange: () => void) {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = () => onChange();
  mq.addEventListener("change", handler);
  return () => mq.removeEventListener("change", handler);
}
