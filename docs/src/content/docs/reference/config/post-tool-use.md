---
title: Post Tool Use
description: Configuration options for postToolUse
---

# Post Tool Use

Configuration for post-tool-use hook with command execution.

This hook allows running commands after tools complete execution for logging, documentation, and integration purposes. Commands are observational (read-only) and cannot block or modify tool execution.

## Configuration Properties

### `commands`

List of commands to execute when tools complete.

Commands run after tool execution completes. They are observational (read-only) and cannot block or modify tool execution. Use them for logging, notifications, or triggering external integrations.

Each command supports: - `run`: (required) Shell command to execute - `tool`: (optional) Glob pattern to filter tools. Default: "*" (all tools) - `showCommand`: (optional) Show command being executed. Default: true - `showStdout`: (optional) Show stdout. Default: false - `showStderr`: (optional) Show stderr. Default: false - `maxOutputLines`: (optional) Limit output lines. Range: 1-10000 - `timeout`: (optional) Command timeout in seconds. Range: 1-3600 - `notifyPerCommand`: (optional) Send individual notifications. Default: false

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

## Nested Types

This section uses the following nested type definitions:

### `PostToolUseCommand` Type

Configuration for individual post-tool-use commands.

These commands run after a tool completes execution. Commands are observational (read-only) and cannot block tool execution.

# Environment Variables

The following environment variables are available in commands: - `CONCLAUDE_TOOL_NAME` - The name of the tool that was executed - `CONCLAUDE_TOOL_INPUT` - JSON string of tool input parameters - `CONCLAUDE_TOOL_OUTPUT` - JSON string of tool response/result - `CONCLAUDE_TOOL_TIMESTAMP` - ISO 8601 timestamp of completion - `CONCLAUDE_TOOL_USE_ID` - Unique identifier for correlation with preToolUse - `CONCLAUDE_SESSION_ID` - Current session ID - `CONCLAUDE_CWD` - Current working directory - `CONCLAUDE_CONFIG_DIR` - Directory containing .conclaude.yaml

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `maxOutputLines` | `integer | null` | `null` | Maximum number of output lines to display (limits both stdout and stderr) |
| `notifyPerCommand` | `boolean | null` | `null` | Whether to send individual notifications for this command (start and completion) |
| `run` | `string` | - | The shell command to execute |
| `showCommand` | `boolean | null` | `true` | Whether to show the command being executed to the user and Claude |
| `showStderr` | `boolean | null` | `null` | Whether to show the command's standard error output to the user and Claude |
| `showStdout` | `boolean | null` | `null` | Whether to show the command's standard output to the user and Claude |
| `timeout` | `integer | null` | `null` | Optional command timeout in seconds |
| `tool` | `string | null` | `null` | Glob pattern to filter which tools trigger this command |

## Complete Examples

Here are complete configuration examples for the `postToolUse` section:

```yaml
postToolUse: commands: # Run for all tools - run: ".claude/scripts/log-tool.sh"

# Run only for specific tools (glob patterns supported) - tool: "AskUserQuestion" run: ".claude/scripts/log-qa.sh"

# Run for multiple tool patterns - tool: "*Search*" run: ".claude/scripts/log-search.sh" showStdout: false
```

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
