---
title: User Prompt Submit
description: Configuration options for userPromptSubmit
---

# User Prompt Submit

Configuration for user prompt submit hook with context injection rules

This hook allows automatic injection of context or instructions into Claude's system prompt based on pattern matching against user-submitted prompts.

## Configuration Properties

### `contextRules`

List of context injection rules that match user prompts and inject context.

Rules are evaluated in order. Multiple rules can match a single prompt, and their contexts will be concatenated in configuration order.

Each rule supports: - `pattern`: (required) Regex pattern to match user prompt - `prompt`: (required) Context to inject when pattern matches - `enabled`: (optional) Whether rule is active. Default: true - `caseInsensitive`: (optional) Case-insensitive matching. Default: false

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

## Nested Types

This section uses the following nested type definitions:

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
```

## See Also

- [Configuration Overview](configuration) - Complete reference for all configuration options
