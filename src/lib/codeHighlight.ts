export function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** Lightweight regex highlight — no extra deps */
export function highlightCode(text: string): string {
  const lines = text.split("\n");
  return lines
    .map((line) => {
      let s = escapeHtml(line);
      s = s.replace(/(\/\/.*$|#.*$)/, '<span class="hl-comment">$1</span>');
      s = s.replace(
        /(&quot;[^&]*?&quot;|'[^']*?'|`[^`]*?`)/g,
        '<span class="hl-string">$1</span>',
      );
      s = s.replace(
        /\b(const|let|var|function|class|import|export|from|return|if|else|for|while|async|await|def|fn|pub|use|struct|impl|new|true|false|null|undefined)\b/g,
        '<span class="hl-keyword">$1</span>',
      );
      s = s.replace(/\b(\d+\.?\d*)\b/g, '<span class="hl-number">$1</span>');
      return s;
    })
    .join("\n");
}

function looksLikeJson(text: string): boolean {
  const t = text.trim();
  if (!(t.startsWith("{") || t.startsWith("["))) return false;
  try {
    JSON.parse(t);
    return true;
  } catch {
    return false;
  }
}

/** True only for JSON or text that looks like source code — not long prose. */
export function isLikelyCode(text: string): boolean {
  const t = text.trim();
  if (t.length < 8) return false;
  if (looksLikeJson(t)) return true;

  const cjk = (t.match(/[\u4e00-\u9fff]/g) || []).length;
  const cjkRatio = cjk / Math.max(t.length, 1);
  const hasCodeKeyword =
    /\b(function|const|let|var|import|export|class|def|fn|public|private|async|await)\b/.test(
      t,
    );
  if (cjkRatio > 0.12 && !hasCodeKeyword) return false;

  const strongMarkers = [
    /\bfunction\s+[\w$]/,
    /\b(const|let|var)\s+[\w$]+\s*=/,
    /\bimport\s+[\w{*]/,
    /\bexport\s+(default\s+)?(function|class|const|let)/,
    /\bclass\s+[\w$]/,
    /\bdef\s+[\w$]+\s*\(/,
    /\bfn\s+[\w$]+/,
    /\bpublic\s+(static\s+)?(void|class|interface|enum)\b/,
    /^\s*#include\s+[<"]/m,
    /=>\s*[({]/,
    /\bpackage\s+[\w.]+;/,
    /\busing\s+namespace\s+/,
  ];
  if (strongMarkers.some((re) => re.test(t))) return true;

  const lines = t.split("\n").filter((l) => l.trim().length > 0);
  if (lines.length < 3) return false;

  const codeLikeLines = lines.filter((line) => {
    const s = line.trim();
    return (
      /^(const|let|var|import|export|function|class|def |fn |if |for |while |return |public |private )/.test(
        s,
      ) ||
      (/[;{}]$/.test(s) && /[a-zA-Z_$][\w$]*\s*[=(]/.test(s))
    );
  });
  return codeLikeLines.length >= 2 && codeLikeLines.length / lines.length >= 0.4;
}
