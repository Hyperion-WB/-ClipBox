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
      s = s.replace(
        /(\/\/.*$|#.*$)/,
        '<span class="hl-comment">$1</span>',
      );
      s = s.replace(
        /(&quot;[^&]*?&quot;|'[^']*?'|`[^`]*?`)/g,
        '<span class="hl-string">$1</span>',
      );
      s = s.replace(
        /\b(const|let|var|function|class|import|export|from|return|if|else|for|while|async|await|def|fn|pub|use|struct|impl|new|true|false|null|undefined)\b/g,
        '<span class="hl-keyword">$1</span>',
      );
      s = s.replace(
        /\b(\d+\.?\d*)\b/g,
        '<span class="hl-number">$1</span>',
      );
      return s;
    })
    .join("\n");
}

export function isLikelyCode(text: string): boolean {
  const t = text.trim();
  if (t.length < 4) return false;
  const markers = [
    "function ", "const ", "let ", "import ", "class ", "def ", "fn ", "public ",
    "=>", "();", "{\n", "}\n",
  ];
  return markers.some((m) => t.includes(m)) || t.split("\n").length > 3;
}
