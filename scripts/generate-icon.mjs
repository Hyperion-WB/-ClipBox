import sharp from "sharp";
import { mkdir, writeFile } from "fs/promises";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const assetsDir = join(root, "assets");

await mkdir(assetsDir, { recursive: true });

const size = 1024;
const svg = `<svg width="${size}" height="${size}" viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#0081ff"/>
      <stop offset="100%" stop-color="#0066cc"/>
    </linearGradient>
  </defs>
  <rect width="1024" height="1024" rx="220" fill="url(#bg)"/>
  <rect x="250" y="180" width="524" height="664" rx="72" fill="#ffffff" opacity="0.95"/>
  <rect x="330" y="120" width="364" height="120" rx="60" fill="#ffffff" opacity="0.95"/>
  <rect x="330" y="320" width="364" height="36" rx="18" fill="#0081ff" opacity="0.35"/>
  <rect x="330" y="410" width="300" height="36" rx="18" fill="#0081ff" opacity="0.25"/>
  <rect x="330" y="500" width="240" height="36" rx="18" fill="#0081ff" opacity="0.18"/>
  <circle cx="760" cy="760" r="120" fill="#4dabf7"/>
  <path d="M720 760 L748 788 L808 712" stroke="#fff" stroke-width="36" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
</svg>`;

const pngPath = join(assetsDir, "clipbox-icon.png");
await sharp(Buffer.from(svg)).png().toFile(pngPath);
console.log("Wrote", pngPath);
