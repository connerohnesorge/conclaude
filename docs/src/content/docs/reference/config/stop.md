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

## Nested Types

This section uses the following nested type definitions:

### `StopCommand` Type

Configuration for individual stop commands with optional messages

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `maxOutputLines` | `integer | null` | `null` | Maximum number of output lines to display (limits both stdout and stderr) |
| `message` | `string | null` | `null` | Custom error message to display when the command fails (exits with non-zero status) |
| `notifyPerCommand` | `boolean | null` | `null` | Whether to send individual notifications for this command (start and completion) |
| `run` | `string` | - | The shell command to execute |
| `showCommand` | `boolean | null` | `true` | Whether to show the command being executed to the user and Claude |
| `showStderr` | `boolean | null` | `null` | Whether to show the command's standard error output to the user and Claude |
| `showStdout` | `boolean | null` | `null` | Whether to show the command's standard output to the user and Claude |
| `skill` | `string | null` | `null` | Optional skill pattern to scope this command to specific skills (e |
| `timeout` | `integer | null` | `null` | Optional command timeout in seconds |

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
