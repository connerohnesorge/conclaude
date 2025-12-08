# Change: Reorganize Hook Subcommands Under `hooks` Parent Command

## Why

Currently, all hook subcommands (`PreToolUse`, `PostToolUse`, `SubagentStop`, etc.) are top-level CLI commands. This creates a flat CLI structure that:
- Pollutes the top-level command namespace with 11 hook-specific commands
- Makes it harder to distinguish between user-facing commands (`Init`, `Validate`, `Visualize`) and Claude Code integration hooks
- Prevents future hook-related subcommands from being grouped logically

Reorganizing hooks under a parent `hooks` subcommand improves CLI organization and aligns with the principle that hooks are an internal integration mechanism.

## What Changes

- **BREAKING**: All hook commands move under `hooks` parent subcommand:
  - `conclaude PreToolUse` → `conclaude hooks PreToolUse`
  - `conclaude PostToolUse` → `conclaude hooks PostToolUse`
  - `conclaude PermissionRequest` → `conclaude hooks PermissionRequest`
  - `conclaude Notification` → `conclaude hooks Notification`
  - `conclaude UserPromptSubmit` → `conclaude hooks UserPromptSubmit`
  - `conclaude SessionStart` → `conclaude hooks SessionStart`
  - `conclaude SessionEnd` → `conclaude hooks SessionEnd`
  - `conclaude Stop` → `conclaude hooks Stop`
  - `conclaude SubagentStart` → `conclaude hooks SubagentStart`
  - `conclaude SubagentStop` → `conclaude hooks SubagentStop`
  - `conclaude PreCompact` → `conclaude hooks PreCompact`

- `Init` command must be updated to generate hook configurations with the new command format (`conclaude hooks <HookName>`)

- Top-level commands after change: `Init`, `Validate`, `Visualize`, `hooks`

## Impact

- Affected specs: `cli-structure` (new capability)
- Affected code:
  - `src/main.rs` - CLI structure, Commands enum, Init handler
  - `.claude/settings.json` - Generated hook command paths (via Init)
  - Documentation/README
