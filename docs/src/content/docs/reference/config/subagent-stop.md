---
title: Subagent Stop
description: Configuration options for subagentStop
---

# Subagent Stop

Configuration for subagent stop hooks with pattern-based command execution.

This hook allows configuring different commands for different subagent names using pattern matching. Commands run when a subagent finishes its work.

# Pattern Matching Rules

- Patterns are matched in the order they appear in the configuration - First matching pattern's commands are executed - Use "*" to match all subagents (put last as fallback) - Glob patterns support: *, ?, \[abc\], \[a-z\], {foo,bar}

# Environment Variables

The following environment variables are available in subagent stop commands:
- `CONCLAUDE_AGENT_ID` - The raw agent identifier (e.g., "adb0a8b")
- `CONCLAUDE_AGENT_NAME` - The semantic agent name (e.g., "coder", "tester", "stuck") extracted from the main transcript. Falls back to `CONCLAUDE_AGENT_ID` if extraction fails.
- `CONCLAUDE_AGENT_TRANSCRIPT_PATH` - Path to subagent's transcript
- `CONCLAUDE_SESSION_ID` - Current session ID
- `CONCLAUDE_TRANSCRIPT_PATH` - Main transcript file path
- `CONCLAUDE_HOOK_EVENT` - Always "SubagentStop"
- `CONCLAUDE_CWD` - Current working directory

## Configuration Properties

### `commands`

Map of subagent name patterns to command configurations.

Each key is a glob pattern that matches against the subagent name. Commands are executed in the order they appear when the pattern matches.

Pattern examples: - `"*"` - Matches all subagents (wildcard) - `"coder"` - Exact match for subagent named "coder" - `"test*"` - Matches any subagent name starting with "test" - `"*coder"` - Matches any subagent name ending with "coder"

Command options (same as stop hook): - `run`: (required) Command to execute - `showStdout`: (optional) Show stdout to user/Claude. Default: false - `showStderr`: (optional) Show stderr to user/Claude. Default: false - `message`: (optional) Custom error message on non-zero exit - `maxOutputLines`: (optional) Limit output lines. Range: 1-10000 - `timeout`: (optional) Command timeout in seconds. Range: 1-3600 (1 second to 1 hour). When timeout occurs, command is terminated and hook is blocked.

| Attribute | Value |
|-----------|-------|
| **Type** | `object` |
| **Default** | `{}` |

## Nested Types

This section uses the following nested type definitions:

### `SubagentStopCommand` Type

Configuration for individual subagent stop commands with optional messages

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `maxOutputLines` | `integer | null` | `null` | Maximum number of output lines to display (limits both stdout and stderr) |
| `message` | `string | null` | `null` | Custom error message to display when the command fails (exits with non-zero status) |
| `run` | `string` | - | The shell command to execute |
| `showStderr` | `boolean | null` | `null` | Whether to show the command's standard error output to the user and Claude |
| `showStdout` | `boolean | null` | `null` | Whether to show the command's standard output to the user and Claude |
| `timeout` | `integer | null` | `null` | Optional command timeout in seconds |

## Complete Examples

Here are complete configuration examples for the `subagentStop` section:

```yaml
subagentStop: commands: # Exact match - only runs for subagent named "coder" coder: - run: "npm run lint" showStdout: true message: "Linting failed"

# Glob pattern - runs for any subagent name starting with "test" test*: - run: "npm test" timeout: 600

# Wildcard - runs for ALL subagents "*": - run: "echo 'Subagent completed'"
```

## See Also

- [Configuration Overview](/reference/config/configuration/) - Complete reference for all configuration options
