---
title: Configuration Reference
description: Complete reference for Conclaude configuration options
---

# Configuration Reference

Conclaude uses YAML configuration files to define lifecycle hooks, file protection rules, and workflow policies for Claude Code sessions. The configuration system is discovered via cosmiconfig and validated against a JSON Schema to ensure correctness.

**IDE Support**: Conclaude provides a JSON Schema for configuration validation and autocomplete in your editor. The schema is available at `conclaude-schema.json` and can be referenced in your YAML files for enhanced editing support.

## Quick Reference

| Section | Description | Key Options |
|---------|-------------|-------------|
| [Config Change](/conclaude/reference/config/config-change) | Configuration for config change hooks with source-based command execution | `commands` |
| [Notifications](/conclaude/reference/config/notifications) | Configuration for system notifications | `enabled`, `hooks`, `showErrors` |
| [Permission Request](/conclaude/reference/config/permission-request) | Configuration for permission request hooks that control tool permission decisions | `allow`, `default`, `deny` |
| [Pre Tool Use](/conclaude/reference/config/pre-tool-use) | Configuration for pre-tool-use hooks that run before tools are executed | `preventAdditions`, `preventRootAdditions`, `preventRootAdditionsMessage` |
| [Skill Start](/conclaude/reference/config/skill-start) | Configuration for skill start hooks that trigger when subagents (skills) start | `commands` |
| [Stop](/conclaude/reference/config/stop) | Configuration for stop hook commands that run when Claude is about to stop | `commands`, `infinite`, `infiniteMessage` |
| [Subagent Stop](/conclaude/reference/config/subagent-stop) | Configuration for subagent stop hooks with pattern-based command execution | `commands` |
| [Task Completed](/conclaude/reference/config/task-completed) | Configuration for task completed hooks with pattern-based command execution | `commands` |
| [Teammate Idle](/conclaude/reference/config/teammate-idle) | Configuration for teammate idle hooks with pattern-based command execution | `commands` |
| [User Prompt Submit](/conclaude/reference/config/user-prompt-submit) | Configuration for user prompt submit hook with context injection rules and command execution | `commands`, `contextRules`, `slashCommands` |
| [Worktree Create](/conclaude/reference/config/worktree-create) | Configuration for worktree create hook | `command`, `timeout` |

## Configuration Sections

Detailed documentation for each configuration section:

### [Config Change](/conclaude/reference/config/config-change)

Configuration for config change hooks with source-based command execution.

### [Notifications](/conclaude/reference/config/notifications)

Configuration for system notifications.

### [Permission Request](/conclaude/reference/config/permission-request)

Configuration for permission request hooks that control tool permission decisions.

### [Pre Tool Use](/conclaude/reference/config/pre-tool-use)

Configuration for pre-tool-use hooks that run before tools are executed.

### [Skill Start](/conclaude/reference/config/skill-start)

Configuration for skill start hooks that trigger when subagents (skills) start.

### [Stop](/conclaude/reference/config/stop)

Configuration for stop hook commands that run when Claude is about to stop

### [Subagent Stop](/conclaude/reference/config/subagent-stop)

Configuration for subagent stop hooks with pattern-based command execution.

### [Task Completed](/conclaude/reference/config/task-completed)

Configuration for task completed hooks with pattern-based command execution.

### [Teammate Idle](/conclaude/reference/config/teammate-idle)

Configuration for teammate idle hooks with pattern-based command execution.

### [User Prompt Submit](/conclaude/reference/config/user-prompt-submit)

Configuration for user prompt submit hook with context injection rules and command execution.

### [Worktree Create](/conclaude/reference/config/worktree-create)

Configuration for worktree create hook.

