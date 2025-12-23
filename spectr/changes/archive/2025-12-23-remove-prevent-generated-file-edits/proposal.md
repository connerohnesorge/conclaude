# Remove preventGeneratedFileEdits Feature

## Why

The `preventGeneratedFileEdits` feature was never formally specified and is being removed as a hard breaking change. The feature automatically blocks edits to files containing markers like "DO NOT EDIT", "@generated", etc., but is overly opinionated, creates false positives, and duplicates functionality available via `uneditableFiles`.

## What Changes

- **BREAKING**: Remove `preventGeneratedFileEdits` config field from `preToolUse` section
- **BREAKING**: Remove `generatedFileMessage` config field from `preToolUse` section
- Remove `check_auto_generated_file` function from hooks implementation
- Remove `check_generated_file_markers` helper function
- Remove all related tests
- Update documentation to remove references

## Impact

- Affected specs: None (feature was never formally specified in `spectr/specs/preToolUse/spec.md`)
- Affected code: `src/config.rs`, `src/hooks.rs`, `src/default-config.yaml`, `conclaude-schema.json`
- Affected tests: `src/config_test.rs`, `tests/hooks_tests.rs`, `tests/integration_tests.rs`, `tests/output_limiting_tests.rs`
- Affected docs: `docs/src/content/docs/reference/config/pre-tool-use.md`

## Migration

Users relying on this feature can achieve similar protection by adding specific generated files to `uneditableFiles`:

```yaml
preToolUse:
  uneditableFiles:
    - "path/to/generated-file.ts"
    - "*.generated.ts"
```

This provides explicit, user-controlled protection without content scanning.
