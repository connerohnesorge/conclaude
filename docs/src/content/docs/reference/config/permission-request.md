---
title: Permission Request
description: Configuration options for permissionRequest
---

# Permission Request

Configuration for permission request hooks that control tool permission decisions.

This hook is fired when Claude requests permission to use a tool. Use this to automatically approve or deny tool usage based on configurable rules.

# Pattern Matching

Both `allow` and `deny` fields support glob patterns for flexible tool matching: - `"Bash"` - Exact match (only "Bash") - `"*"` - Wildcard (matches any tool) - `"Edit*"` - Prefix match (matches "Edit", "EditFile", etc.) - `"*Read"` - Suffix match (matches "Read", "FileRead", etc.)

**Important**: Deny patterns take precedence over allow patterns.

# Security Recommendations

- **Whitelist approach (recommended)**: Set `default: "deny"` and explicitly list allowed tools - **Blacklist approach (more permissive)**: Set `default: "allow"` and explicitly list denied tools

## Configuration Properties

### `allow`

Tools to explicitly allow using glob patterns.

These patterns are checked AFTER deny patterns. If a tool matches both an allow and a deny pattern, the deny pattern takes precedence.

# Pattern Examples

- `"Read"` - Exact match for the Read tool - `"*"` - Match all tools (use with caution) - `"Edit*"` - Match any tool starting with "Edit" - `"*Read"` - Match any tool ending with "Read"

# Common Tools

- `"Read"` - Read files - `"Write"` - Write files - `"Edit"` - Edit files - `"Bash"` - Execute bash commands - `"Glob"` - File pattern matching - `"Grep"` - Content search - `"Task"` - Subagent tasks

Default: `None` (no tools explicitly allowed)

| Attribute | Value |
|-----------|-------|
| **Type** | `array | null` |
| **Default** | `null` |

### `default`

Default decision when a tool doesn't match any allow or deny rule.

Valid values: - `"allow"` - Permit tools by default (blacklist approach) - `"deny"` - Block tools by default (whitelist approach, recommended for security)

The default action is applied when a tool is requested that doesn't match any patterns in the `allow` or `deny` lists.

| Attribute | Value |
|-----------|-------|
| **Type** | `string` |

### `deny`

Tools to explicitly deny using glob patterns.

Deny patterns take precedence over allow patterns. If a tool matches both an allow and a deny pattern, it will be denied.

# Pattern Examples

- `"BashOutput"` - Block reading background process output - `"KillShell"` - Block terminating background shells - `"Bash"` - Block all bash command execution - `"*"` - Block all tools (use with specific allow rules)

Default: `None` (no tools explicitly denied)

| Attribute | Value |
|-----------|-------|
| **Type** | `array | null` |
| **Default** | `null` |

## Complete Examples

Here are complete configuration examples for the `permissionRequest` section:

### Example 1

```yaml
permissionRequest: default: deny allow: - "Read"       # Allow reading files - "Glob"       # Allow file pattern matching - "Grep"       # Allow content search - "Edit"       # Allow file editing - "Write"      # Allow file writing - "Task"       # Allow subagent tasks - "Bash"       # Allow bash commands
```

### Example 2

```yaml
permissionRequest: default: allow deny: - "BashOutput"   # Block reading background process output - "KillShell"    # Block terminating background shells
```

### Example 3

```yaml
permissionRequest: default: deny allow: - "Read" - "Write" - "Edit*"      # Allow all Edit-based tools deny: - "Bash"       # Explicitly deny even though default is deny
```

## See Also

- [Configuration Overview](./configuration) - Complete reference for all configuration options
