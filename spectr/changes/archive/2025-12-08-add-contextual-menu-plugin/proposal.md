# Change: Add Contextual Menu Plugin to Docs Site

## Why
The documentation site lacks convenient content interaction features. Users would benefit from quick actions like copying page content, viewing raw markdown, and opening content in AI assistants directly from the documentation interface.

## What Changes
- Add `starlight-contextual-menu` npm package as a dependency to `docs/package.json`
- Configure the plugin in `docs/astro.config.mjs` with contextual menu actions
- Enable copy, view markdown, ChatGPT, and Claude actions in the contextual menu

## Impact
- Affected specs: `documentation`
- Affected code: `docs/package.json`, `docs/astro.config.mjs`
- User experience: Enhanced with right-click contextual menu for content interaction
