#!/usr/bin/env node
/**
 * Extract release notes for a version from CHANGELOG.md
 * Usage: node scripts/release-notes.mjs v0.1.1
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const tag = process.argv[2] ?? "v0.1.0";
const version = tag.replace(/^v/, "");
const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const changelog = readFileSync(join(root, "CHANGELOG.md"), "utf8");

const header = `## [${version}]`;
const start = changelog.indexOf(header);
if (start === -1) {
  console.error(`No section ${header} in CHANGELOG.md`);
  process.exit(1);
}

let section = changelog.slice(start);
const afterHeader = section.slice(header.length);
const next = afterHeader.search(/\n## \[/);
if (next !== -1) section = section.slice(0, header.length + next);
const body = section.replace(/^## \[[^\]]+\][^\n]*\n?/, "").trim();

const assets = `

---

| 平台 | 安装包 |
|------|--------|
| Windows | \`.exe\` (NSIS) / \`.msi\` |
| Linux | \`.deb\` / AppImage |

详见 GitHub Release Assets。`;

console.log(`# ClipBox ${tag}\n\n${body}${assets}`);
