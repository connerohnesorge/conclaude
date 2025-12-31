---
title: Notifications
description: Configuration options for notifications
---

# Notifications

Configuration for system notifications.

Controls desktop notifications for hook execution, errors, successes, and system events. Notifications help you stay informed about what conclaude is doing in the background.

## Configuration Properties

### `enabled`

Enable system notifications for hook execution.

When enabled, conclaude will send desktop notifications based on the configured notification types (errors, successes, system events) and hook filters.

Default: `false`

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `false` |

### `hooks`

List of hook names that should trigger notifications.

Use `["*"]` to receive notifications for all hooks, or specify individual hook names to filter which hooks generate notifications.

Common hook names: - `"Stop"` - When Claude is about to stop - `"PreToolUse"` - Before tools are executed - `"PostToolUse"` - After tools are executed - `"SessionStart"` - When a session starts - `"UserPromptSubmit"` - When user submits a prompt - `"Notification"` - General notifications - `"SubagentStop"` - When subagents stop - `"PreCompact"` - Before transcript compaction

Examples: - `["*"]` - All hooks - `["Stop", "PreToolUse"]` - Only specific hooks - `["Stop"]` - Only stop hook notifications

Default: `[]` (no hooks)

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

### `showErrors`

Show error notifications (hook failures, system errors).

When enabled, you'll receive desktop notifications when hooks fail or system errors occur. Useful for catching issues early.

Default: `false`

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `false` |

### `showSuccess`

Show success notifications (hook completion, successful operations).

When enabled, you'll receive desktop notifications when hooks complete successfully and operations finish without errors.

Default: `false`

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `false` |

### `showSystemEvents`

Show system event notifications (session start/end, configuration loaded).

When enabled, you'll receive desktop notifications for system-level events like session initialization, configuration loading, and session termination.

Default: `true`

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `true` |

## Complete Examples

Here are complete configuration examples for the `notifications` section:

### Example 1

```yaml
# Enable notifications for all hooks notifications: enabled: true hooks: ["*"] showErrors: true showSuccess: true showSystemEvents: true
```

### Example 2

```yaml
# Enable notifications only for Stop hook notifications: enabled: true hooks: ["Stop"] showErrors: true showSuccess: false showSystemEvents: false
```

### Example 3

```yaml
# Enable notifications for specific hooks notifications: enabled: true hooks: ["Stop", "PreToolUse"] showErrors: true showSuccess: true showSystemEvents: true
```

### Example 4: Per-Command Notifications

```yaml
# Enable per-command notifications for granular feedback
notifications:
  enabled: true
  hooks: ["Stop", "SubagentStop"]
  showErrors: true
  showSuccess: true
  showSystemEvents: false

stop:
  commands:
    # Get notified for each command
    - run: npm test
      notifyPerCommand: true
      showCommand: true
      message: "Tests failed"

    - run: npm run build
      notifyPerCommand: true
      showCommand: true
      message: "Build failed"

    # Skip notifications for quick commands
    - run: git status
      notifyPerCommand: false

subagentStop:
  commands:
    coder:
      - run: npm run lint
        notifyPerCommand: true
        showCommand: true
        message: "Linting failed after coder"
```

This configuration will send notifications:
- When `npm test` starts: "Running: npm test"
- When `npm test` completes: "Command completed: npm test" (or error if failed)
- When `npm run build` starts: "Running: npm run build"
- When `npm run build` completes: "Command completed: npm run build" (or error if failed)
- When coder subagent triggers linting: Start and completion notifications
- No per-command notifications for `git status` (but hook-level notifications still apply)

### Example 5: Mixed Notification Strategy

```yaml
# Selective per-command notifications
notifications:
  enabled: true
  hooks: ["*"]
  showErrors: true      # Always show errors
  showSuccess: false    # Don't show success for every command
  showSystemEvents: true

stop:
  commands:
    # Only notify for long-running commands
    - run: cargo test --all
      notifyPerCommand: true
      showCommand: true
      timeout: 600

    - run: cargo build --release
      notifyPerCommand: true
      showCommand: true
      timeout: 300

    # Quick checks - no per-command notifications
    - run: cargo fmt --check
      notifyPerCommand: false

    - run: cargo clippy
      notifyPerCommand: false
```

This configuration:
- Sends per-command notifications only for long-running test and build commands
- Shows errors for all commands (including quick checks)
- Skips success notifications for quick checks
- Provides focused notifications where they're most valuable

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
