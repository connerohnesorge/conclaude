# Change: Add Starlight Changelogs Plugin with GitHub Integration

## Why

The documentation site currently lacks an integrated changelog display that automatically pulls from GitHub releases. Adding the starlight-changelogs plugin with GitHub provider integration will surface release history directly in the docs, providing users with version information and release notes without leaving the documentation site.

## What Changes

- Add `starlight-changelogs` npm dependency to docs/package.json
- Configure starlight-changelogs plugin in docs/astro.config.mjs with GitHub provider
- Set up content collections in docs/src/content.config.ts to support changelogs
- Configure GitHub provider to track releases from the conclaude repository
- Add changelog navigation entry to the Starlight sidebar

## Impact

- Affected specs: `documentation`
- Affected code:
  - `docs/package.json` - new dependency
  - `docs/astro.config.mjs` - plugin configuration
  - `docs/src/content.config.ts` - new file for content collections
  - Sidebar configuration in astro.config.mjs
