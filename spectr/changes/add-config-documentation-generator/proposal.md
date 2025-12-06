# Change: Add Configuration Documentation Generator

## Why

The conclaude project has rich configuration options defined in Rust structs with serde annotations, doc comments, and JSON Schema metadata (`schemars`). Currently, documentation for configuration values must be manually written and maintained separately from the code, leading to potential drift between the actual implementation and documentation.

Generating documentation directly from code ensures:
1. Single source of truth for configuration documentation
2. Automatic synchronization when config structs change
3. Type information, defaults, and constraints are always accurate
4. Reduced maintenance burden

## What Changes

- Add a build-time documentation generator that extracts configuration metadata from Rust code
- Generate Markdown documentation files in `docs/src/content/docs/reference/`
- Include field descriptions, types, defaults, and validation constraints from JSON Schema
- Parse doc comments from Rust source as field descriptions
- Integrate with existing Starlight documentation site

**Key outputs (hybrid structure):**
- `docs/src/content/docs/reference/configuration.md` - Overview with quick reference table
- `docs/src/content/docs/reference/config/stop.md` - Stop hook configuration details
- `docs/src/content/docs/reference/config/subagent-stop.md` - Subagent stop hook details
- `docs/src/content/docs/reference/config/pre-tool-use.md` - PreToolUse hook details
- `docs/src/content/docs/reference/config/notifications.md` - Notifications configuration
- `docs/src/content/docs/reference/config/permission-request.md` - Permission request details

**Prerequisites:**
- Consolidate detailed comments from `default-config.yaml` into Rust doc comments
- This ensures `schemars` generates rich descriptions in JSON Schema

## Impact

- **Affected specs**: `documentation` (ADDED requirements for config docs generation)
- **Affected code**:
  - New generator binary or build script in `src/bin/` or `build.rs`
  - New Markdown files in `docs/src/content/docs/reference/`
  - Possibly extend `FieldList` derive macro to include descriptions
- **No breaking changes**: Documentation generation is additive
