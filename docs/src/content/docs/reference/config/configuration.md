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
| [Notifications](./notifications) | Configuration for system notifications | `enabled`, `hooks`, `showErrors` |
| [Permission Request](./permission-request) | Configuration for permission request hooks that control tool permission decisions | `allow`, `default`, `deny` |
| [Pre Tool Use](./pre-tool-use) | Configuration for pre-tool-use hooks that run before tools are executed | `generatedFileMessage`, `preventAdditions`, `preventGeneratedFileEdits` |
| [Stop](./stop) | Configuration for stop hook commands that run when Claude is about to stop | `commands`, `infinite`, `infiniteMessage` |
| [Subagent Stop](./subagent-stop) | Configuration for subagent stop hooks with pattern-based command execution | `commands` |

## Configuration Sections

Detailed documentation for each configuration section:

### [Notifications](./notifications)

Configuration for system notifications.

### [Permission Request](./permission-request)

Configuration for permission request hooks that control tool permission decisions.

### [Pre Tool Use](./pre-tool-use)

Configuration for pre-tool-use hooks that run before tools are executed.

### [Stop](./stop)

Configuration for stop hook commands that run when Claude is about to stop

### [Subagent Stop](./subagent-stop)

Configuration for subagent stop hooks with pattern-based command execution.

