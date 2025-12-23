# Change: Add showCommand Configuration Option

## Why

When running stop hook commands, conclaude currently always prints "Executing command X/Y: <command>" to stdout. Some users want to suppress this output for cleaner logs when the command itself is not informative (e.g., `npm test` is self-explanatory) or when running many commands where the output becomes noisy.

## What Changes

- Add `showCommand` boolean field to `StopCommand` struct (default: `true`)
- Add `showCommand` boolean field to `SubagentStopCommand` struct (default: `true`)
- When `showCommand: false`, suppress the "Executing command X/Y: <command>" output line
- When `showCommand: true` (default), preserve current behavior
- Update JSON schema to include the new field

## Impact

- Affected specs: `execution`
- Affected code:
  - `src/config.rs` - Add `showCommand` field to `StopCommand` and `SubagentStopCommand` structs
  - `src/hooks.rs` - Conditionally print command execution line based on `showCommand` value
  - `conclaude-schema.json` - Auto-generated, will include new field
- Backward compatible: Existing configs work unchanged (default `true` preserves current behavior)
