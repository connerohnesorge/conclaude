---
title: Skill Start
description: Configuration options for skillStart
---

# Skill Start

Configuration for skill start hooks that trigger when subagents (skills) start.

This allows configuring different commands for different skill types using pattern matching. Commands run when a skill/subagent begins execution.

# Pattern Matching Rules

- Patterns are matched in the order they appear in the configuration - First matching pattern's commands are executed - Use "*" to match all skills (put last as fallback) - Glob patterns support: *, ?, \[abc\], \[a-z\], {foo,bar}

# Environment Variables

The following environment variables are available in skill start hooks: - `CONCLAUDE_SKILL_NAME` - The skill/agent type name - `CONCLAUDE_AGENT_ID` - Unique agent identifier - `CONCLAUDE_AGENT_TRANSCRIPT_PATH` - Path to subagent's transcript - `CONCLAUDE_SESSION_ID` - Current session ID - `CONCLAUDE_TRANSCRIPT_PATH` - Main transcript file path - `CONCLAUDE_HOOK_EVENT` - Always "SubagentStart" - `CONCLAUDE_CWD` - Current working directory

## Configuration Properties

### `commands`

Map of skill name patterns to command configurations.

Each key is a glob pattern that matches against the skill/agent type. Commands are executed in the order they appear when the pattern matches.

Pattern examples: - `"*"` - Matches all skills (wildcard) - `"coder"` - Exact match for coder skill - `"test*"` - Matches any skill starting with "test" - `"*coder"` - Matches any skill ending with "coder"

Command options: - `run`: (required) Command to execute - `showStdout`: (optional) Show stdout to user/Claude. Default: false - `showStderr`: (optional) Show stderr to user/Claude. Default: false - `message`: (optional) Custom error message on non-zero exit - `maxOutputLines`: (optional) Limit output lines. Range: 1-10000 - `timeout`: (optional) Command timeout in seconds. Range: 1-3600

| Attribute | Value |
|-----------|-------|
| **Type** | `object` |
| **Default** | `{}` |

## Nested Types

This section uses the following nested type definitions:

### `SkillStartCommand` Type

Configuration for individual skill start commands with optional messages

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
| `timeout` | `integer | null` | `null` | Optional command timeout in seconds |

## Complete Examples

Here are complete configuration examples for the `skillStart` section:

```yaml
skillStart: commands: # Exact match - only runs when "coder" skill starts "coder": - run: ".claude/scripts/coder-init.sh" showStdout: true

# Glob pattern - runs for any skill starting with "test" "test*": - run: ".claude/scripts/test-env-check.sh"

# Wildcard - runs for ALL skills "*": - run: ".claude/scripts/log-skill.sh" showCommand: false
```

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
