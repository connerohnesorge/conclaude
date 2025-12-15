# Conclaude Documentation Site

This directory contains the official documentation site for **conclaude**, a Rust-based guardrail CLI for Claude Code sessions. The documentation is built using [Astro](https://astro.build/) with the [Starlight](https://starlight.astro.build/) documentation theme.

## Overview

The documentation site provides comprehensive guides, reference material, and configuration documentation for conclaude users and contributors. It includes:

- Getting started guides
- Installation instructions
- Hook configuration reference
- CLI reference
- Changelog

## Prerequisites

Before working on the documentation, ensure you have:

- **Node.js** (v18 or later recommended)
- **npm** (comes with Node.js)

## Installation

Install all dependencies:

```bash
npm install
```

## Development

### Running the Dev Server

Start the development server with hot-reload:

```bash
npm run dev
```

This will start the Astro dev server at `http://localhost:4321`. The site will automatically reload when you make changes to the documentation files.

Alternatively, you can use:

```bash
npm start
```

### Generating Configuration Documentation

Conclaude includes auto-generated configuration documentation from the Rust codebase. To regenerate this documentation:

```bash
npm run generate:config-docs
```

This command runs the Rust `generate-docs` binary which creates configuration reference files in `src/content/docs/reference/config/`.

## Building

### Production Build

Build the static site for production:

```bash
npm run build
```

The built site will be output to the `dist/` directory.

### Build with Config Docs

To regenerate configuration documentation and build in one step:

```bash
npm run docs:build
```

This is equivalent to running `npm run generate:config-docs && npm run build`.

### Preview Production Build

Preview the production build locally:

```bash
npm run preview
```

## Project Structure

```
docs/
├── src/
│   ├── content/           # Documentation content
│   │   └── docs/          # Markdown documentation files
│   │       ├── guides/    # User guides
│   │       └── reference/ # Reference documentation
│   ├── assets/            # Images and static assets
│   └── content.config.ts  # Content configuration
├── public/                # Static files copied to output
├── astro.config.mjs       # Astro configuration
├── package.json           # Node dependencies and scripts
├── tsconfig.json          # TypeScript configuration
└── README.md              # This file
```

## Configuration

The site is configured in `astro.config.mjs` and includes several Starlight plugins:

- **starlight-contextual-menu**: Adds contextual actions (copy, view, AI chat)
- **starlight-changelogs**: Changelog integration
- **starlight-links-validator**: Validates internal links
- **starlight-llms-txt**: LLM-friendly documentation format
- **starlight-site-graph**: Site structure visualization
- **star-warp**: Enhanced routing capabilities

## Writing Documentation

### Adding New Pages

1. Create a new `.md` or `.mdx` file in `src/content/docs/`
2. Add frontmatter with title and description:

```md
---
title: Page Title
description: Brief description of the page
---

# Content goes here
```

3. Update the sidebar in `astro.config.mjs` if needed

### Markdown Features

Starlight supports:

- Standard Markdown syntax
- Code blocks with syntax highlighting
- Admonitions (notes, warnings, tips)
- Tables
- And more! See [Starlight documentation](https://starlight.astro.build/)

## Contributing to Documentation

When contributing to the documentation:

1. Follow the existing structure and formatting
2. Run `npm run dev` to preview changes locally
3. Ensure all links are valid (the links validator will check this)
4. If adding new configuration reference, regenerate docs with `npm run generate:config-docs`
5. Build the site to ensure there are no errors: `npm run build`

## Deployment

The documentation site is configured to deploy to GitHub Pages with the base path `/conclaude`. The site URL is: `https://connerohnesorge.github.io/conclaude`

## Testing

The docs directory includes Playwright for end-to-end testing:

```bash
npx playwright test
```

## Support

For issues or questions about the documentation site itself, please open an issue in the [conclaude repository](https://github.com/connerohnesorge/conclaude).
