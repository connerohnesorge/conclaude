---
title: User Prompt Submit
description: Configuration options for userPromptSubmit
---

# User Prompt Submit

Configuration for user prompt submit hook with context injection rules and command execution.

This hook allows automatic injection of context or instructions into Claude's system prompt based on pattern matching against user-submitted prompts, as well as running shell commands when prompts match patterns.

## Configuration Properties

### `commands`

List of commands to execute when user prompts are submitted.

Commands run after contextRules are evaluated. They are observational (read-only) and cannot block prompt processing. Use them for logging, notifications, or triggering external integrations.

Each command supports: - `run`: (required) Shell command to execute - `pattern`: (optional) Regex pattern to filter prompts. Default: runs for all - `caseInsensitive`: (optional) Case-insensitive pattern matching. Default: false - `showCommand`: (optional) Show command being executed. Default: true - `showStdout`: (optional) Show stdout. Default: false - `showStderr`: (optional) Show stderr. Default: false - `maxOutputLines`: (optional) Limit output lines. Range: 1-10000 - `timeout`: (optional) Command timeout in seconds. Range: 1-3600

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

### `contextRules`

List of context injection rules that match user prompts and inject context.

Rules are evaluated in order. Multiple rules can match a single prompt, and their contexts will be concatenated in configuration order.

Each rule supports: - `pattern`: (required) Regex pattern to match user prompt - `prompt`: (required) Context to inject when pattern matches - `enabled`: (optional) Whether rule is active. Default: true - `caseInsensitive`: (optional) Case-insensitive matching. Default: false

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

### `slashCommands`

Configuration for slash command hooks that trigger when users invoke slash commands.

Allows running custom commands when specific slash commands are detected in user prompts. Slash commands are detected from the prompt text (e.g., `/commit`, `/deploy`).

Commands are executed after contextRules and regular commands processing. Unlike regular commands, slash command hooks CAN block prompt processing if a command exits with code 2.

| Attribute | Value |
|-----------|-------|
| **Type** | `unknown` |
| **Default** | `null` |

**Examples:**

```yaml
userPromptSubmit: slashCommands: commands: # Run for /commit command "/commit": - run: ".claude/scripts/pre-commit.sh" showStdout: true

# Run for any /test* command "/test*": - run: ".claude/scripts/test-setup.sh"

# Run for all slash commands "*": - run: ".claude/scripts/log-command.sh"
```

## Nested Types

This section uses the following nested type definitions:

### `UserPromptSubmitCommand` Type

Configuration for individual user prompt submit commands.

These commands run when a user submits a prompt to Claude. Commands are observational (read-only) and cannot block prompt processing.

# Environment Variables

The following environment variables are available in commands: - `CONCLAUDE_USER_PROMPT` - The user's input text - `CONCLAUDE_SESSION_ID` - Current session ID - `CONCLAUDE_CWD` - Current working directory - `CONCLAUDE_CONFIG_DIR` - Directory containing .conclaude.yaml - `CONCLAUDE_HOOK_EVENT` - Always "UserPromptSubmit"

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `caseInsensitive` | `boolean | null` | `null` | Use case-insensitive pattern matching |
| `maxOutputLines` | `integer | null` | `null` | Maximum number of output lines to display (limits both stdout and stderr) |
| `notifyPerCommand` | `boolean | null` | `null` | Whether to send individual notifications for this command (start and completion) |
| `pattern` | `string | null` | `null` | Regex pattern to filter which prompts trigger this command |
| `run` | `string` | - | The shell command to execute |
| `showCommand` | `boolean | null` | `true` | Whether to show the command being executed to the user and Claude |
| `showStderr` | `boolean | null` | `null` | Whether to show the command's standard error output to the user and Claude |
| `showStdout` | `boolean | null` | `null` | Whether to show the command's standard output to the user and Claude |
| `timeout` | `integer | null` | `null` | Optional command timeout in seconds |

### `ContextInjectionRule` Type

Configuration for a single context injection rule

Rules define patterns to match against user prompts and context to inject when matches occur.

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `caseInsensitive` | `boolean | null` | `null` | Use case-insensitive pattern matching |
| `enabled` | `boolean | null` | `true` | Whether this rule is active |
| `pattern` | `string` | - | Regex pattern to match against user prompt text |
| `prompt` | `string` | - | Context or instructions to prepend to Claude's system prompt when pattern matches |

## Complete Examples

Here are complete configuration examples for the `userPromptSubmit` section:

```yaml
userPromptSubmit: contextRules: # Basic pattern matching - pattern: "sidebar" prompt: | Make sure to read @.claude/contexts/sidebar.md before proceeding.

# Multiple patterns with logical OR - pattern: "auth|login|authentication" prompt: | Review the authentication patterns in @.claude/contexts/auth.md

# Case-insensitive matching - pattern: "(?i)database|sql|query" prompt: | Follow the database conventions in @.claude/contexts/database.md

# Optional rule that can be disabled - pattern: "performance" prompt: "Consider performance implications" enabled: false

# Command execution (runs after contextRules processing) commands: # Run for all prompts - run: ".claude/scripts/log-prompt.sh"

# Run only for prompts matching pattern - pattern: "deploy|release" run: ".claude/scripts/notify-deploy.sh"
```

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
