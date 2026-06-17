---
title: Post Tool Batch
description: Configuration options for postToolBatch
---

# Post Tool Batch

Configuration for post-tool-batch hooks.

Commands run once after every tool call in a batch resolves. Observational.

## Configuration Properties

### `commands`

Commands to execute after each resolved tool batch. They run in order and are observational.

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

## Nested Types

This section uses the following nested type definitions:

### `PostToolBatchCommand` Type

Configuration for individual post-tool-batch commands with optional messages.

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
