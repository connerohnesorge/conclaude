# Change: Remove generatedFileMessage Configuration Field

## Why

The `generatedFileMessage` field adds complexity for minimal value. It allows users to customize the error message when `preventGeneratedFileEdits` blocks a file operation, but the default message is already clear and informative. Removing this field simplifies the configuration surface and reduces maintenance overhead.

## What Changes

- **BREAKING**: Remove `generatedFileMessage` field from `preToolUse` configuration
- Remove field from `PreToolUseConfig` struct in Rust code
- Remove field from JSON schema
- Remove field from default configuration
- Update documentation to remove references
- Simplify hook logic to always use the default message

## Impact

- Affected specs: `preToolUse`
- Affected code:
  - `src/config.rs` - Remove field from `PreToolUseConfig` struct and documentation
  - `src/hooks.rs` - Remove custom message logic, always use default message
  - `src/default-config.yaml` - Remove field
  - `conclaude-schema.json` - Remove field from schema
  - `docs/src/content/docs/reference/config/pre-tool-use.md` - Remove documentation
  - `.conclaude.yaml` - Remove field from example config
