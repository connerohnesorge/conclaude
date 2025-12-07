# Change: Add Starlight Site Graph Plugin

## Why

The documentation site currently lacks a visual way to explore the relationships and connections between different documentation pages. The starlight-site-graph plugin provides an interactive graph visualization that helps users understand the structure and interconnections within the documentation, improving navigation and discoverability.

## What Changes

- Add `starlight-site-graph` npm dependency to docs/package.json
- Integrate the plugin into the Starlight configuration in docs/astro.config.mjs
- Use minimal/default configuration to enable the feature

## Impact

- **Affected specs**: `specs/documentation/spec.md`
- **Affected code**:
  - `docs/package.json` - new dependency
  - `docs/astro.config.mjs` - plugin integration
- **User-facing**: Adds interactive graph visualization feature to the documentation site
- **Breaking changes**: None
