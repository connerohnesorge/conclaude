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

## See Also

- [Configuration Overview](/reference/config/configuration) - Complete reference for all configuration options
