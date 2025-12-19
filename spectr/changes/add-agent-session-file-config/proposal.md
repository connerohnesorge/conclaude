# Change: Add Agent Session File Configuration

## Why

Agent session files (`/tmp/conclaude-agent-{session_id}.json`) are currently deleted unconditionally during SubagentStop with errors silently ignored. This creates two issues:

1. **No debugging capability**: Users cannot inspect session files after subagent completion to debug agent-aware file protection issues
2. **Silent failures**: If cleanup fails (permissions, file locked, etc.), errors are silently swallowed, hiding potential issues

## What Changes

- Add `preserveAgentSessionFiles` boolean configuration option under `subagentStop` section
- When enabled, agent session files are retained after SubagentStop for debugging purposes
- Properly log cleanup errors instead of silently ignoring them
- Add cleanup error handling with configurable behavior

## Impact

- Affected specs: `subagent-hooks`
- Affected code: `src/hooks.rs` (cleanup_agent_session_file function), `src/config.rs` (SubagentStopConfig struct)
- Non-breaking: Default behavior remains unchanged (files are deleted, but errors are now logged)
