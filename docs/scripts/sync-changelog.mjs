import fs from "node:fs";
import path from "node:path";

const src = path.resolve("../CHANGELOG.md");
const dest = path.resolve("src/content/docs/changelog.md");

const content = fs.readFileSync(src, "utf-8");

// Add Starlight frontmatter
const frontmatter = `---
title: Changelog
description: All notable changes to conclaude
---

`;

fs.writeFileSync(dest, frontmatter + content);

console.log("✓ Changelog synced");
