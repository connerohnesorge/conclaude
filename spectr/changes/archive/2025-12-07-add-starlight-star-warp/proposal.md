# Change: Add Starlight Star Warp Plugin

## Why

The documentation site currently lacks fast, client-side search functionality. The star-warp plugin integrates Pagefind search engine with Astro/Starlight, providing users with instant, offline-capable search across all documentation pages without requiring a backend service or external search provider.

## What Changes

- Add `@inox-tools/star-warp` npm dependency to docs/package.json
- Integrate the plugin into the Astro configuration in docs/astro.config.mjs
- Use minimal/default configuration to enable the feature

## Impact

- **Affected specs**: `specs/documentation/spec.md`
- **Affected code**:
  - `docs/package.json` - new dependency
  - `docs/astro.config.mjs` - plugin integration
- **User-facing**: Adds fast client-side search functionality to the documentation site
- **Breaking changes**: None
