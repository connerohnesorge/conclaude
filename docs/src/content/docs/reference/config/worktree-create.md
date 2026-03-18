---
title: Worktree Create
description: Configuration options for worktreeCreate
---

# Worktree Create

Configuration for worktree create hook.

When configured, this command runs instead of the default `git worktree add`. The command must output the worktree path on stdout.

## Configuration Properties

### `command`

The shell command to execute to create a worktree. Must output the worktree path to stdout. Environment variable CONCLAUDE_WORKTREE_NAME is set to the requested name.

| Attribute | Value |
|-----------|-------|
| **Type** | `string | null` |
| **Default** | `null` |

### `timeout`

Optional command timeout in seconds. Range: 1-3600.

| Attribute | Value |
|-----------|-------|
| **Type** | `integer | null` |
| **Default** | `null` |

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
