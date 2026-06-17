---
title: Message Display
description: Configuration options for messageDisplay
---

# Message Display

Configuration for message-display hooks.

`MessageDisplay` fires per streamed line-batch (potentially many times per message), so by default commands run only on the final flush of each message. Observational.

## Configuration Properties

### `commands`

Commands to execute on a message-display event. Observational.

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

### `onlyFinal`

Run commands only on the final flush of each message (recommended; avoids running per streamed line-batch). Default: true

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `true` |

## Nested Types

This section uses the following nested type definitions:

### `MessageDisplayCommand` Type

Configuration for individual message-display commands with optional messages.

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `maxOutputLines` | `integer | null` | `null` | Maximum number of output lines to display (limits both stdout and stderr) |
| `message` | `string | null` | `null` | Custom error message to display when the command fails (exits with non-zero status) |
| `notifyPerCommand` | `boolean | null` | `null` | Whether to send individual notifications for this command |
| `run` | `string` | - | The shell command to execute |
| `showCommand` | `boolean | null` | `true` | Whether to show the command being executed to the user and Claude |
| `showStderr` | `boolean | null` | `null` | Whether to show the command's standard error output to the user and Claude |
| `showStdout` | `boolean | null` | `null` | Whether to show the command's standard output to the user and Claude |
| `timeout` | `integer | null` | `null` | Optional command timeout in seconds |

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
