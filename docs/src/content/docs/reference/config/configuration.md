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
| [Notifications](/conclaude/reference/config/notifications) | Configuration for system notifications | `enabled`, `hooks`, `showErrors` |
| [Permission Request](/conclaude/reference/config/permission-request) | Configuration for permission request hooks that control tool permission decisions | `allow`, `default`, `deny` |
| [Pre Tool Use](/conclaude/reference/config/pre-tool-use) | Configuration for pre-tool-use hooks that run before tools are executed | `preventAdditions`, `preventRootAdditions`, `preventRootAdditionsMessage` |
| [Stop](/conclaude/reference/config/stop) | Configuration for stop hook commands that run when Claude is about to stop | `commands`, `infinite`, `infiniteMessage` |
| [Subagent Stop](/conclaude/reference/config/subagent-stop) | Configuration for subagent stop hooks with pattern-based command execution | `commands` |
| [User Prompt Submit](/conclaude/reference/config/user-prompt-submit) | Configuration for user prompt submit hook with context injection rules | `contextRules` |

## Configuration Sections

Detailed documentation for each configuration section:

### [Notifications](/conclaude/reference/config/notifications)

Configuration for system notifications.

### [Permission Request](/conclaude/reference/config/permission-request)

Configuration for permission request hooks that control tool permission decisions.

### [Pre Tool Use](/conclaude/reference/config/pre-tool-use)

Configuration for pre-tool-use hooks that run before tools are executed.

### [Stop](/conclaude/reference/config/stop)

Configuration for stop hook commands that run when Claude is about to stop

### [Subagent Stop](/conclaude/reference/config/subagent-stop)

Configuration for subagent stop hooks with pattern-based command execution.

### [User Prompt Submit](/conclaude/reference/config/user-prompt-submit)

Configuration for user prompt submit hook with context injection rules

