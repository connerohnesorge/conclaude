---
title: Stop
description: Configuration options for stop
---

# Stop

Configuration for stop hook commands that run when Claude is about to stop

## Configuration Properties

### `commands`

List of commands to execute when Claude is about to stop. Commands run in order and can provide custom error messages and control output display.

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

### `infinite`

Infinite mode - when enabled, allows Claude to continue automatically instead of ending the session after stop hook commands succeed. Default: false

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `false` |

### `infiniteMessage`

Message to send to Claude when infinite mode is enabled and stop hook commands succeed. Claude receives this message to continue working.

| Attribute | Value |
|-----------|-------|
| **Type** | `string | null` |
| **Default** | `null` |

## Per-Command Notifications

The `notifyPerCommand` field allows you to receive individual desktop notifications when specific commands start and complete, providing granular feedback during stop hook execution.

### When to Use Per-Command Notifications

Per-command notifications are useful when:

- **Long-running commands**: You have commands that take significant time to complete (tests, builds, etc.) and want to know when they start and finish
- **Multiple commands**: Your stop hook runs several commands and you want to track progress through each one
- **Failure debugging**: You need to quickly identify which specific command in a sequence failed
- **Background awareness**: You're working on other tasks while stop hook commands execute and want to stay informed

### How It Works

When `notifyPerCommand: true` is set on a command:

1. **Start notification**: Sent when the command begins execution (if notifications are enabled)
2. **Completion notification**: Sent when the command finishes (success or failure)
3. **Command context**: The notification includes the command name (if `showCommand: true`) or a generic message
4. **Filter respect**: All existing notification filters (`hooks`, `showErrors`, `showSuccess`) continue to apply

### Requirements

- Global notifications must be enabled: `notifications.enabled: true`
- The Stop hook must be in the notification hooks list: `notifications.hooks` includes `"Stop"` or `"*"`
- Notification preferences are respected: `showErrors` and `showSuccess` control what gets notified

### Example Usage

```yaml
# Enable notifications globally
notifications:
  enabled: true
  hooks: ["Stop"]
  showErrors: true
  showSuccess: true

stop:
  commands:
    # Get notified for long-running test command
    - run: npm test
      notifyPerCommand: true
      showCommand: true
      message: "Tests failed - fix before continuing"

    # Get notified for build command
    - run: npm run build
      notifyPerCommand: true
      showCommand: true

    # No notifications for quick check
    - run: git status
      notifyPerCommand: false
```

With this configuration:
- You'll receive a notification when `npm test` starts: "Running: npm test"
- You'll receive a notification when `npm test` completes: "Command completed: npm test"
- Same for `npm run build`
- No per-command notifications for `git status` (though hook-level notifications still apply)

## Nested Types

This section uses the following nested type definitions:

### `StopCommand` Type

Configuration for individual stop commands with optional messages

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

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
