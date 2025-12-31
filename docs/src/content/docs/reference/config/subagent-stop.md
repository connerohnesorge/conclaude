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

The following environment variables are available in subagent stop commands: - `CONCLAUDE_AGENT_ID` - The subagent's identifier - `CONCLAUDE_AGENT_TRANSCRIPT_PATH` - Path to subagent's transcript - `CONCLAUDE_SESSION_ID` - Current session ID - `CONCLAUDE_TRANSCRIPT_PATH` - Main transcript file path - `CONCLAUDE_HOOK_EVENT` - Always "SubagentStop" - `CONCLAUDE_CWD` - Current working directory

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

## Per-Command Notifications

The `notifyPerCommand` field allows you to receive individual desktop notifications when specific subagent stop commands start and complete, providing granular feedback during subagent completion.

### When to Use Per-Command Notifications

Per-command notifications are particularly useful for subagent stop hooks when:

- **Monitoring subagent workflows**: You want to track what validation or cleanup happens when specific subagents complete (e.g., coder, tester, stuck)
- **Debugging subagent issues**: You need to identify which validation command failed when a subagent completes
- **Long-running validations**: Your subagent stop commands perform time-consuming operations (linting, testing, builds)
- **Multi-subagent projects**: You have different commands for different subagent types and want granular tracking

### How It Works

When `notifyPerCommand: true` is set on a subagent stop command:

1. **Start notification**: Sent when the command begins execution (if notifications are enabled)
2. **Completion notification**: Sent when the command finishes (success or failure)
3. **Command context**: The notification includes the command name (if `showCommand: true`) or a generic message
4. **Subagent context**: The notification indicates which subagent triggered the command
5. **Filter respect**: All existing notification filters (`hooks`, `showErrors`, `showSuccess`) continue to apply

### Requirements

- Global notifications must be enabled: `notifications.enabled: true`
- The SubagentStop hook must be in the notification hooks list: `notifications.hooks` includes `"SubagentStop"` or `"*"`
- Notification preferences are respected: `showErrors` and `showSuccess` control what gets notified

### Example Usage

```yaml
# Enable notifications globally
notifications:
  enabled: true
  hooks: ["SubagentStop"]
  showErrors: true
  showSuccess: true

subagentStop:
  commands:
    # Get notified when coder subagent completes
    coder:
      - run: npm run lint
        notifyPerCommand: true
        showCommand: true
        message: "Linting failed after coder subagent"

      - run: npm test
        notifyPerCommand: true
        showCommand: true
        message: "Tests failed after coder subagent"

    # Get notified for tester subagent validation
    tester:
      - run: npm run validate-tests
        notifyPerCommand: true
        showCommand: true

    # Wildcard fallback without per-command notifications
    "*":
      - run: echo 'Subagent completed'
        notifyPerCommand: false
```

With this configuration:
- When the "coder" subagent completes:
  - Notification: "Running: npm run lint"
  - Notification: "Command completed: npm run lint"
  - Notification: "Running: npm test"
  - Notification: "Command completed: npm test"
- When the "tester" subagent completes:
  - Notification: "Running: npm run validate-tests"
  - Notification: "Command completed: npm run validate-tests"
- Other subagents: No per-command notifications (though hook-level notifications still apply)

## Nested Types

This section uses the following nested type definitions:

### `SubagentStopCommand` Type

Configuration for individual subagent stop commands with optional messages

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `maxOutputLines` | `integer | null` | `null` | Maximum number of output lines to display (limits both stdout and stderr) |
| `message` | `string | null` | `null` | Custom error message to display when the command fails (exits with non-zero status) |
| `notifyPerCommand` | `boolean | null` | `false` | When true, sends individual notifications for this command's start and completion. Requires notifications to be enabled globally. Respects notification filters (showErrors, showSuccess). |
| `run` | `string` | - | The shell command to execute |
| `showCommand` | `boolean | null` | `true` | Whether to show the command being executed to the user and Claude |
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

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
