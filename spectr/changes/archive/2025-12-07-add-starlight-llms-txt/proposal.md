# Change: Add starlight-llms-txt Plugin to Docs Site

## Why

The documentation site (`./docs/`) should generate machine-readable context files (`llms.txt`, `llms-full.txt`, `llms-small.txt`) that enable AI systems to efficiently learn from the documentation. The `starlight-llms-txt` plugin automatically generates these files during builds, making the documentation more accessible to LLMs and AI-assisted development tools.

## What Changes

- Install `starlight-llms-txt` npm package as a dependency
- Configure the plugin in `astro.config.mjs` within the Starlight plugins array
- Add required `site` URL to astro config if not already present
- Verify generated files at `/llms.txt`, `/llms-full.txt`, and `/llms-small.txt` after build

## Impact

- Affected specs: `documentation`
- Affected code: `docs/package.json`, `docs/astro.config.mjs`
- No breaking changes
- Generates three additional output files during builds
- Minimal build time increase
