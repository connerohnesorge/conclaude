# Change: Add preventRootAdditionsMessage Configuration

## Why

Users need the ability to customize the error message shown when `preventRootAdditions` blocks file creation at the repository root. This allows teams to provide more context-specific guidance (e.g., "Files should be placed in src/ or docs/") rather than the generic system message.

This follows the established pattern of `generatedFileMessage` which already provides customizable messages for another file protection feature.

## What Changes

- Add `preventRootAdditionsMessage` optional string field to `preToolUse` configuration
- Support `{file_path}` and `{tool}` template variables for message customization
- Default to `null` which uses the existing hardcoded message

## Impact

- Affected specs: preToolUse
- Affected code: `src/config.rs`, `src/hooks.rs`, `src/default-config.yaml`, `src/config_test.rs`
- Affected docs: `docs/src/content/docs/reference/config/pre-tool-use.md`
- No breaking changes - purely additive feature
