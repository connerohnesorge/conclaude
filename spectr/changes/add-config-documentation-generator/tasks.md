## 1. Comment Consolidation (Prerequisite)

- [x] 1.1 Migrate `stop` section comments from `default-config.yaml` to `StopConfig` and `StopCommand` doc comments in `src/config.rs`
- [x] 1.2 Migrate `subagentStop` section comments to `SubagentStopConfig` and `SubagentStopCommand` doc comments
- [x] 1.3 Migrate `preToolUse` section comments to `PreToolUseConfig`, `ToolUsageRule`, and `UnEditableFileRule` doc comments
- [x] 1.4 Migrate `notifications` section comments to `NotificationsConfig` doc comments
- [x] 1.5 Migrate `permissionRequest` section comments to `PermissionRequestConfig` doc comments
- [x] 1.6 Regenerate `conclaude-schema.json` to verify descriptions appear in schema
- [x] 1.7 Simplify `default-config.yaml` comments to avoid duplication (keep minimal examples only)

## 2. Generator Scaffolding

- [x] 2.1 Create generator binary scaffold at `src/bin/generate-docs.rs`
- [x] 2.2 Add serde_json dependency for schema parsing
- [x] 2.3 Define output directory structure: `docs/src/content/docs/reference/config/`

## 3. Core Generator Implementation

- [x] 3.1 Parse `conclaude-schema.json` and extract type definitions
- [x] 3.2 Implement Markdown generation with Starlight frontmatter
- [x] 3.3 Generate overview page (`reference/configuration.md`) with quick reference table
- [x] 3.4 Generate per-section detail pages:
  - `reference/config/stop.md`
  - `reference/config/subagent-stop.md`
  - `reference/config/pre-tool-use.md`
  - `reference/config/notifications.md`
  - `reference/config/permission-request.md`
- [x] 3.5 Include inline YAML examples extracted from default-config.yaml
- [x] 3.6 Add links between overview and detail pages

## 4. Nested Type Documentation

- [x] 4.1 Generate subsections for `StopCommand` fields
- [x] 4.2 Generate subsections for `SubagentStopCommand` fields
- [x] 4.3 Generate subsections for `ToolUsageRule` fields
- [x] 4.4 Generate subsections for `UnEditableFileRule` variants

## 5. Integration

- [x] 5.1 Update Starlight sidebar in `astro.config.mjs` to include new reference pages
- [x] 5.2 Add npm script or Makefile target for doc generation
- [x] 5.3 Add CI workflow step to verify docs are up-to-date

## 6. Validation & Testing

- [x] 6.1 Verify all schema fields appear in generated docs
- [x] 6.2 Verify Starlight builds successfully with generated content
- [x] 6.3 Verify idempotent output (multiple runs produce identical results)
- [x] 6.4 Manual review of generated documentation for clarity
