---
title: CLI Reference
description: Complete reference for all conclaude CLI commands and options.
---

conclaude provides a command-line interface for initializing configuration, validating settings, and handling Claude Code lifecycle hooks.

## Global Options

These options apply to all commands:

| Option | Description |
|--------|-------------|
| `--version` | Print version information |
| `--help` | Print help information |

## Commands

### `init`

Initialize conclaude configuration and Claude Code hooks in the current directory.

```bash
conclaude init [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--config-path <PATH>` | Custom path for `.conclaude.yaml` file |
| `--claude-path <PATH>` | Custom path for `.claude` directory |
| `-f, --force` | Overwrite existing configuration files |
| `--schema-url <URL>` | Custom schema URL for YAML language server header |

**Examples:**

```bash
# Initialize with defaults
conclaude init

# Force overwrite existing config
conclaude init --force

# Custom configuration path
conclaude init --config-path ./config/conclaude.yaml
```

**Created Files:**

- `.conclaude.yaml` — Project configuration with YAML language server support
- `.claude/settings.json` — Claude Code hook configuration (created or updated)

---

### `validate`

Validate conclaude configuration file syntax and schema compliance.

```bash
conclaude validate [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--config-path <PATH>` | Path to configuration file or directory to validate |

**Examples:**

```bash
# Validate default configuration
conclaude validate

# Validate specific file
conclaude validate --config-path ./production.yaml

# Use in CI scripts
conclaude validate && echo "Config valid" || exit 1
```

**Exit Codes:**

| Code | Meaning |
|------|---------|
| `0` | Configuration is valid |
| `1` | Validation failed (syntax error, schema violation, file not found) |

---

### `visualize`

Display file and directory protection settings from configuration.

```bash
conclaude visualize [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-r, --rule <RULE>` | Specific rule to visualize |
| `--show-matches` | Show files that match the rule patterns |

**Available Rules:**

- `uneditableFiles` — Files protected from editing
- `preventRootAdditions` — Root directory protection status
- `toolUsageValidation` — Tool-specific validation rules

**Examples:**

```bash
# Show all rules overview
conclaude visualize

# Show specific rule details
conclaude visualize --rule uneditableFiles

# Show matching files
conclaude visualize --rule uneditableFiles --show-matches
```

---

## Hook Commands

These commands are called by Claude Code during session lifecycle events. They read JSON payloads from stdin and output results.

All hook commands now support an optional `--agent <name>` flag for agent-aware execution. When provided, the agent name is available to hook handlers via the `CONCLAUDE_AGENT_NAME` environment variable.

### `Hooks`

New unified hook command structure that supports all hook types with optional agent awareness.

```bash
# Standard hook execution
echo '{"session_id":"...","tool_name":"Write",...}' | conclaude Hooks PreToolUse

# Agent-aware hook execution
conclaude Hooks PreToolUse --agent coder
conclaude Hooks Stop --agent tester
```

**Options:**

| Option | Description |
|--------|-------------|
| `--agent <name>` | Agent name for agent-aware hook execution |

**Note:** Agent frontmatter hooks (`.claude/agents/*.md`) automatically use the `--agent` flag. Run `conclaude init` to inject hooks into agent files.

---

### `PreToolUse`

Fired before Claude uses any tool (Write, Bash, Read, etc.).

```bash
echo '{"session_id":"...","tool_name":"Write",...}' | conclaude Hooks PreToolUse
# or legacy:
echo '{"session_id":"...","tool_name":"Write",...}' | conclaude PreToolUse
```

**Use Cases:**
- Block file creation at project root
- Protect files from editing
- Validate tool inputs

---

### `PostToolUse`

Fired after a tool operation completes.

```bash
echo '{"session_id":"...","tool_name":"Write",...}' | conclaude Hooks PostToolUse
# or legacy:
echo '{"session_id":"...","tool_name":"Write",...}' | conclaude PostToolUse
```

**Use Cases:**
- Audit logging
- Performance monitoring
- Post-processing validation

---

### `Stop`

Fired when Claude finishes a task and the session is about to end.

```bash
echo '{"session_id":"...","stop_hook_active":true,...}' | conclaude Hooks Stop
# or legacy:
echo '{"session_id":"...","stop_hook_active":true,...}' | conclaude Stop
```

**Use Cases:**
- Run linting and formatting checks
- Execute test suites
- Verify build succeeds
- Continuous validation (infinite mode)

---

### `SessionStart`

Fired when a Claude Code session begins.

```bash
echo '{"session_id":"...",...}' | conclaude SessionStart
```

**Use Cases:**
- Initialize session logging
- Set up monitoring
- Prepare workspace

---

### `UserPromptSubmit`

Fired when the user submits input to Claude.

```bash
echo '{"session_id":"...","prompt":"...",...}' | conclaude UserPromptSubmit
```

**Use Cases:**
- Log user prompts
- Pre-process input

---

### `SubagentStart`

Fired when a Claude subagent (coder, tester, etc.) begins work.

```bash
echo '{"session_id":"...","agent_id":"coder",...}' | conclaude SubagentStart
```

**Use Cases:**
- Track subagent initialization
- Resource allocation
- Monitoring setup

---

### `SubagentStop`

Fired when a Claude subagent completes its work.

```bash
echo '{"session_id":"...","agent_id":"coder",...}' | conclaude SubagentStop
```

**Use Cases:**
- Log subagent completion
- Cleanup operations
- Metrics collection

---

### `Notification`

Fired for system notifications.

```bash
echo '{"session_id":"...",...}' | conclaude Notification
```

---

### `PreCompact`

Fired before transcript compaction.

```bash
echo '{"session_id":"...",...}' | conclaude PreCompact
```

---

### `PermissionRequest`

Fired when a tool requests permission.

```bash
echo '{"session_id":"...",...}' | conclaude PermissionRequest
```

---

### `SessionEnd`

Fired when a session terminates.

```bash
echo '{"session_id":"...",...}' | conclaude SessionEnd
```

---

## Exit Codes

All hook commands use consistent exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success — operation allowed to proceed |
| `1` | Error — validation failure, parsing error, or handler crash |
| `2` | Blocked — hook explicitly blocked the operation |

## Environment Variables

### Configuration

| Variable | Description |
|----------|-------------|
| `CONCLAUDE_LOG_LEVEL` | Log level: `debug`, `info`, `warn`, `error` |
| `CONCLAUDE_DISABLE_FILE_LOGGING` | Disable logging to temporary files |

### Hook Context

Available to commands executed by hooks:

| Variable | Description |
|----------|-------------|
| `CONCLAUDE_SESSION_ID` | Unique session identifier |
| `CONCLAUDE_TRANSCRIPT_PATH` | Path to session transcript file |
| `CONCLAUDE_CWD` | Current working directory |
| `CONCLAUDE_HOOK_EVENT` | Name of executing hook |
| `CONCLAUDE_CONFIG_DIR` | Directory containing config file |

### Agent-Aware Hooks

When using the `--agent` flag (automatically set in agent frontmatter hooks):

| Variable | Description |
|----------|-------------|
| `CONCLAUDE_AGENT_NAME` | Agent name passed via `--agent` flag (e.g., "coder", "tester", "stuck") |

### Subagent Hooks

Additional variables for SubagentStart and SubagentStop (payload-based):

| Variable | Description |
|----------|-------------|
| `CONCLAUDE_AGENT_ID` | Agent identifier from payload (e.g., "adb0a8b") |
| `CONCLAUDE_SUBAGENT_TYPE` | Subagent type (e.g., "implementation") (SubagentStart only) |
| `CONCLAUDE_AGENT_TRANSCRIPT_PATH` | Path to subagent transcript |
| `CONCLAUDE_PERMISSION_MODE` | Permission mode (SubagentStart only) |
