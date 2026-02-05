---
title: Pre Tool Use
description: Configuration options for preToolUse
---

# Pre Tool Use

Configuration for pre-tool-use hooks that run before tools are executed.

All file protection rules are consolidated in this section to prevent Claude from making unintended modifications to protected files, directories, or executing dangerous commands.

## Configuration Properties

### `preventAdditions`

Directories where file additions are prevented (in addition to root if `preventRootAdditions` is enabled).

List of directory paths where new files cannot be created. Useful for protecting build output directories or other generated content.

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

**Examples:**

```yaml
preventAdditions: - "dist" - "build" - "node_modules"
```

### `preventRootAdditions`

Prevent Claude from creating or modifying files at the repository root.

Helps maintain clean project structure by preventing clutter at the root level. This is a security best practice to avoid accidental modification of important configuration files.

Default: `true`

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `true` |

### `preventRootAdditionsMessage`

Custom message when blocking file creation at repository root.

Available placeholders: - `{file_path}` - The path to the file being blocked - `{tool}` - The tool name that attempted the operation (e.g., "Write")

# Example

```yaml preventRootAdditionsMessage: "Files must go in src/. Cannot create {file_path} using {tool}." ```

Default: `null` (uses a generic error message)

| Attribute | Value |
|-----------|-------|
| **Type** | `string | null` |
| **Default** | `null` |

**Examples:**

```yaml
preventRootAdditionsMessage: "Files must go in src/. Cannot create {file_path} using {tool}."
```

### `preventUpdateGitIgnored`

Block Claude from modifying or creating files that match .gitignore patterns.

When enabled, files matching patterns in .gitignore will be protected. Uses your existing .gitignore as the source of truth for file protection.

Default: `false`

| Attribute | Value |
|-----------|-------|
| **Type** | `boolean` |
| **Default** | `false` |

### `toolUsageValidation`

Tool usage validation rules for fine-grained control over tool usage.

Allows controlling which tools can be used on which files or with which command patterns. Rules are evaluated in order.

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

**Examples:**

```yaml
toolUsageValidation: # Allow writing to JavaScript files - tool: "Write" pattern: "**/*.js" action: "allow"

# Block environment file modifications - tool: "*" pattern: ".env*" action: "block" message: "Environment files cannot be modified"

# Block dangerous git operations - tool: "Bash" commandPattern: "git push --force*" action: "block" message: "Force push is not allowed"
```

### `uneditableFiles`

Files that Claude cannot edit, using glob patterns.

Supports various glob patterns for flexible file protection. By default, conclaude's own config files are protected to prevent the AI from modifying guardrail settings - this is a security best practice.

Supports two formats: 1. Simple string patterns: `"*.lock"` 2. Detailed objects with custom messages: `{pattern: "*.lock", message: "..."}` 3. Detailed objects with agent scoping: `{pattern: "*.lock", agent: "coder"}`

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

**Examples:**

```yaml
uneditableFiles: - ".conclaude.yml"    # Protect config - ".conclaude.yaml"   # Alternative extension - "*.lock"            # Lock files - pattern: ".env*" message: "Environment files contain secrets. Use .env.example instead." - pattern: "src/**/*.test.ts" agent: "coder" message: "The coder agent should not modify test files."
```

## Nested Types

This section uses the following nested type definitions:

### `ToolUsageRule` Type

Tool usage validation rule for fine-grained control over tool usage based on file patterns.

Allows controlling which tools can be used on which files or with which command patterns. Rules are evaluated in order and the first matching rule determines the action. Supports optional agent scoping to apply rules only to specific agents.

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `action` | `string` | - | Action to take when the rule matches: "allow" or "block" |
| `agent` | `string | null` | `null` | Optional agent pattern to scope this rule to specific agents (e |
| `commandPattern` | `string | null` | - | Optional command pattern to match for Bash tool |
| `matchMode` | `string | null` | - | Optional match mode for pattern matching (reserved for future use) |
| `message` | `string | null` | - | Optional custom message to display when the rule blocks an action |
| `pattern` | `string` | - | File path pattern to match |
| `skill` | `string | null` | `null` | Optional skill pattern to scope this rule to specific skills (e |
| `tool` | `string` | - | The tool name to match against |

### `UnEditableFileRule` Type

Configuration for an uneditable file rule.

Files that Claude cannot edit, using glob patterns. Supports various glob patterns for flexible file protection.

# Formats

Two formats are supported for backward compatibility:

1. **Simple string patterns**: `"*.lock"` - Just the glob pattern as a string - Uses a generic error message when blocking

2. **Detailed objects with custom messages**: `{pattern: "*.lock", message: "..."}` - Allows specifying a custom error message - More descriptive feedback when files are blocked

**Variants:**

1. **object**: Detailed format with pattern and optional custom message.

Allows providing a custom error message that will be shown when Claude attempts to edit a file matching this pattern.

   Properties:
   - `agent` (string | null): Optional agent pattern to scope this rule to specific agents (e.g., "coder", "tester", "main", or glob patterns like "code*")
   - `message` (string | null): Optional custom message to display when blocking edits to matching files
   - `pattern` (string): Glob pattern matching files to protect (e.g., "*.lock", ".env*", "src/**/*.ts")
   - `skill` (string | null): Optional skill pattern to scope this rule to specific skills (e.g., "tester", "commit", or glob patterns like "test*")

2. **string**: Simple format: just a glob pattern string.

Uses a generic error message when blocking file edits. Backward compatible with existing configurations.

## Complete Examples

Here are complete configuration examples for the `preToolUse` section:

```yaml
preToolUse: # Prevent root-level file creation preventRootAdditions: true

# Protect specific files with glob patterns uneditableFiles: - ".conclaude.yml" - "*.lock" - pattern: ".env*" message: "Environment files contain secrets"

# Prevent modifications to git-ignored files preventUpdateGitIgnored: false

# Fine-grained tool control toolUsageValidation: - tool: "Bash" commandPattern: "git push --force*" action: "block" message: "Force push is not allowed"

# Block additions to specific directories preventAdditions: - "dist" - "build"
```

## See Also

- [Configuration Overview](/conclaude/reference/config/configuration) - Complete reference for all configuration options
