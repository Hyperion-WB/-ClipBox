export type SensitiveKind =
  | "id_card"
  | "bank_card"
  | "phone"
  | "otp"
  | "token"
  | "email_password";

export interface SensitiveAnalysis {
  sensitive: boolean;
  kinds: SensitiveKind[];
}

const CN_ID =
  /\b[1-9]\d{5}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx]\b/g;

const CN_PHONE = /(?<!\d)1[3-9]\d{9}(?!\d)/g;

const BANK_CARD = /\b(?:\d[ -]?){13,18}\d\b/g;

const OTP_CODE = /\b\d{4,8}\b/g;

const TOKEN_PATTERNS = [
  /\b(?:sk|pk)[-_][a-zA-Z0-9]{16,}\b/g,
  /\bghp_[a-zA-Z0-9]{20,}\b/g,
  /\bAKIA[0-9A-Z]{16}\b/g,
  /\bBearer\s+[a-zA-Z0-9._\-]{20,}\b/gi,
];

const PASSWORD_KV =
  /(?:password|passwd|pwd|密码)\s*[:=]\s*\S+/gi;

function luhnValid(digits: string): boolean {
  if (digits.length < 13 || digits.length > 19) return false;
  let sum = 0;
  let alt = false;
  for (let i = digits.length - 1; i >= 0; i--) {
    let n = parseInt(digits[i]!, 10);
    if (alt) {
      n *= 2;
      if (n > 9) n -= 9;
    }
    sum += n;
    alt = !alt;
  }
  return sum % 10 === 0;
}

function maskDigits(digits: string, keepEnd = 4): string {
  const show = Math.min(keepEnd, digits.length);
  const hidden = Math.max(0, digits.length - show);
  return "•".repeat(hidden) + digits.slice(-show);
}

function maskIdCard(raw: string): string {
  if (raw.length < 8) return "•".repeat(raw.length);
  return raw.slice(0, 3) + "•".repeat(raw.length - 7) + raw.slice(-4);
}

function maskPhone(raw: string): string {
  if (raw.length < 7) return maskDigits(raw, 2);
  return raw.slice(0, 3) + "••••" + raw.slice(-4);
}

function maskToken(raw: string): string {
  if (raw.length <= 8) return "•".repeat(raw.length);
  return raw.slice(0, 4) + "•".repeat(Math.min(12, raw.length - 8)) + raw.slice(-4);
}

function maskPasswordLine(raw: string): string {
  const m = raw.match(/^([^:=]+[:=]\s*)(\S+)$/i);
  if (!m) return maskToken(raw);
  return m[1] + maskToken(m[2]!);
}

export function analyzeSensitive(text: string): SensitiveAnalysis {
  const kinds = new Set<SensitiveKind>();
  if (!text || text.length < 4) {
    return { sensitive: false, kinds: [] };
  }

  if (CN_ID.test(text)) kinds.add("id_card");
  CN_ID.lastIndex = 0;

  if (CN_PHONE.test(text)) kinds.add("phone");
  CN_PHONE.lastIndex = 0;

  for (const m of text.matchAll(BANK_CARD)) {
    const digits = m[0].replace(/\D/g, "");
    if (digits.length >= 13 && digits.length <= 19 && luhnValid(digits)) {
      kinds.add("bank_card");
      break;
    }
  }

  if (PASSWORD_KV.test(text)) kinds.add("email_password");
  PASSWORD_KV.lastIndex = 0;

  for (const re of TOKEN_PATTERNS) {
    if (re.test(text)) {
      kinds.add("token");
      break;
    }
  }

  // Standalone 4–8 digit codes when line is mostly numeric (likely OTP)
  const trimmed = text.trim();
  if (/^\d{4,8}$/.test(trimmed)) kinds.add("otp");

  return {
    sensitive: kinds.size > 0,
    kinds: [...kinds],
  };
}

export function maskForDisplay(text: string, enabled = true): string {
  if (!enabled || !text) return text;
  const { sensitive } = analyzeSensitive(text);
  if (!sensitive) return text;

  let out = text;

  out = out.replace(CN_ID, (m) => maskIdCard(m));
  CN_ID.lastIndex = 0;

  out = out.replace(CN_PHONE, (m) => maskPhone(m));
  CN_PHONE.lastIndex = 0;

  out = out.replace(BANK_CARD, (m) => {
    const digits = m.replace(/\D/g, "");
    if (digits.length >= 13 && luhnValid(digits)) {
      return maskDigits(digits, 4);
    }
    return m;
  });
  BANK_CARD.lastIndex = 0;

  out = out.replace(PASSWORD_KV, (m) => maskPasswordLine(m));
  PASSWORD_KV.lastIndex = 0;

  for (const re of TOKEN_PATTERNS) {
    out = out.replace(re, (m) => maskToken(m));
  }

  if (/^\d{4,8}$/.test(text.trim())) {
    out = maskDigits(text.trim(), 2);
  }

  return out;
}

export function sensitiveKindLabel(kind: SensitiveKind, t: (k: string) => string): string {
  const key = `sensitive.${kind}` as const;
  return t(key);
}
