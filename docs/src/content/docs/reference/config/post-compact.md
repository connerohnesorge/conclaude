---
title: Post Compact
description: Configuration options for postCompact
---

# Post Compact

Configuration for post-compact hooks with trigger-based command execution.

Commands run after transcript compaction completes. Observational - the trigger value ("manual" or "auto") is used as the match query.

## Configuration Properties

### `commands`

Map of trigger patterns to command configurations. Keys are glob patterns matching the compaction trigger (e.g., "manual", "auto", "*").

| Attribute | Value |
|-----------|-------|
| **Type** | `object` |
| **Default** | `{}` |

## Nested Types

This section uses the following nested type definitions:

### `PostCompactCommand` Type

Configuration for individual post-compact commands with optional messages.

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
