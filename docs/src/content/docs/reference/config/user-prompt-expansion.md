---
title: User Prompt Expansion
description: Configuration options for userPromptExpansion
---

# User Prompt Expansion

Configuration for user-prompt-expansion hooks with command-name-based command execution.

Commands run when a slash command or MCP prompt is expanded. Observational.

## Configuration Properties

### `commands`

Map of command-name patterns to command configurations. Keys are glob patterns matched against the expanded command name (e.g., "commit", "*").

| Attribute | Value |
|-----------|-------|
| **Type** | `object` |
| **Default** | `{}` |

## Nested Types

This section uses the following nested type definitions:

### `UserPromptExpansionCommand` Type

Configuration for individual user-prompt-expansion commands with optional messages.

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
