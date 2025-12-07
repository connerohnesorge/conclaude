# Change: Add starlight-links-validator Plugin to Docs Site

## Why

The documentation site (`./docs/`) uses Starlight but lacks automated internal link validation. Broken links degrade user experience and are difficult to catch manually. The `starlight-links-validator` plugin automatically validates all internal links during production builds, catching 404s before deployment.

## What Changes

- Install `starlight-links-validator` npm package as a dev dependency
- Configure the plugin in `astro.config.mjs` within the Starlight plugins array
- Update build script to surface validation errors

## Impact

- Affected specs: `documentation`
- Affected code: `docs/package.json`, `docs/astro.config.mjs`
- No breaking changes
- Build time may increase slightly during production builds (validation only runs in production)
