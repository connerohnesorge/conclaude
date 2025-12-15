# Change: Add timeout field to generated Claude Code hooks

## Why

The `conclaude init` command generates Claude Code hook configurations in `.claude/settings.json`, but these hooks do not include a `timeout` field. According to the Claude Code hooks reference, hooks support an optional `timeout` field (in seconds) that specifies how long a hook should run before being cancelled. Without a timeout, long-running hooks could potentially hang indefinitely, blocking Claude Code sessions.

Setting a default timeout of 10 minutes (600 seconds) provides:
- Protection against runaway hook processes
- Predictable behavior for users
- Alignment with Claude Code's documented hook configuration options

## What Changes

- **Modify `handle_init` function**: Update the `ClaudeHookConfig` struct and hook generation logic to include a `timeout: 600` field (10 minutes) for all generated hooks
- **Update `ClaudeHookConfig` struct**: Add optional `timeout` field to match Claude Code's hook schema

## Impact

- Affected specs: initialization (new spec)
- Affected code: `src/main.rs` (handle_init function, ClaudeHookConfig struct)
- User impact: Newly initialized projects will have hooks with 10-minute timeouts; existing projects are unaffected unless they re-run `conclaude init --force`
