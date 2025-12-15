---
title: userPromptSubmit Configuration
description: Configure context injection rules for UserPromptSubmit hook
---

# userPromptSubmit Configuration

Configure context injection rules that automatically prepend context to Claude's system prompt based on pattern matching against user prompts.

The UserPromptSubmit hook fires when you submit a prompt to Claude. Context injection allows you to automatically add relevant guidelines, documentation, or reminders based on what you're asking about.

## Overview

The `userPromptSubmit` section allows you to define rules that inject context when user prompts match certain patterns. This is useful for:
- Automatically loading project-specific guidelines when discussing certain topics
- Reminding Claude about domain-specific conventions based on keywords
- Adding relevant documentation references when working on specific features
- Enforcing workflow patterns without repeating instructions manually

When a pattern matches your prompt, conclaude prepends the configured context to Claude's system prompt, making it aware of relevant guidelines before processing your request.

## Configuration Structure

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "sidebar"
      prompt: |
        Read @.claude/contexts/sidebar.md before making changes.
      enabled: true
      caseInsensitive: false
```

## Fields

### `contextRules`

Array of context injection rules. Each rule defines a pattern to match against user prompts and the context to inject when matched.

| Attribute | Value |
|-----------|-------|
| **Type** | `array` |
| **Default** | `[]` |

**Context Rule Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `pattern` | `string` | (required) | Regular expression pattern to match against user prompts |
| `prompt` | `string` | (required) | Context to prepend to Claude's system prompt when the pattern matches |
| `enabled` | `boolean` | `true` | Whether this rule is active |
| `caseInsensitive` | `boolean` | `false` | Whether pattern matching should be case-insensitive |

**Pattern Matching Behavior:**

- Patterns are evaluated as regular expressions
- Multiple rules can match a single prompt
- When multiple rules match, their contexts are concatenated in configuration order
- If no rules match, the hook returns success with no modifications

**File References:**

Context prompts support file references using the `@` syntax:
- `@path/to/file.md` is expanded to the file's contents
- Paths are relative to the directory containing the configuration file
- If a file doesn't exist, a warning is logged and the reference remains as literal text
- Multiple file references in a single prompt are expanded independently

## Examples

### Basic Pattern Matching

```yaml
userPromptSubmit:
  contextRules:
    # Inject sidebar context when mentioned
    - pattern: "sidebar"
      prompt: |
        Make sure to read @.claude/contexts/sidebar.md before proceeding.
        Follow the component structure guidelines.
```

### Multiple Patterns with OR Logic

```yaml
userPromptSubmit:
  contextRules:
    # Multiple keywords trigger the same context
    - pattern: "auth|login|authentication|password"
      prompt: |
        Review the authentication patterns in @.claude/contexts/auth.md
        Remember:
        - Always use bcrypt for password hashing
        - Implement rate limiting on auth endpoints
        - Use secure session management
```

### Case-Insensitive Matching

```yaml
userPromptSubmit:
  contextRules:
    # Match database-related terms regardless of case
    - pattern: "database|sql|query|postgres"
      prompt: |
        Follow the database conventions in @.claude/contexts/database.md
        Remember to use parameterized queries for security.
      caseInsensitive: true
```

### Multiple Rules Working Together

```yaml
userPromptSubmit:
  contextRules:
    # General React guidelines
    - pattern: "react|component|jsx"
      prompt: |
        Follow React best practices from @.claude/contexts/react.md

    # Specific testing reminder
    - pattern: "test|testing|jest|vitest"
      prompt: |
        Ensure all new code has test coverage.
        See @.claude/contexts/testing-guide.md

    # API development guidelines
    - pattern: "api|endpoint|route"
      prompt: |
        Review API standards: @.claude/contexts/api-standards.md
        - Use proper HTTP status codes
        - Implement error handling
        - Add request validation
```

### Conditionally Disabled Rules

```yaml
userPromptSubmit:
  contextRules:
    # Active rule
    - pattern: "production|deploy"
      prompt: "Review deployment checklist before proceeding."
      enabled: true

    # Temporarily disabled rule (won't match)
    - pattern: "debug|logging"
      prompt: "Enable verbose logging for debugging."
      enabled: false
```

## Pattern Syntax

Patterns use regular expression syntax. Here are common patterns:

| Pattern | Matches | Example |
|---------|---------|---------|
| `sidebar` | Exact word "sidebar" | "update the sidebar" |
| `auth\|login` | Either "auth" OR "login" | "fix auth bug" or "login page" |
| `(?i)database` | Case-insensitive "database" | "DATABASE", "Database", "database" |
| `^update` | Prompts starting with "update" | "update the docs" |
| `test.*file` | "test" followed by "file" | "test the file handler" |
| `\bapi\b` | Word boundary match for "api" | "api endpoint" (not "rapid") |

**Note:** Remember to escape special regex characters in YAML strings if needed.

## File References

The `@` syntax in prompt text triggers file content expansion:

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "style|css|design"
      prompt: |
        Follow the design system: @.claude/contexts/design-system.md

        Use the color palette: @.claude/contexts/colors.md
```

**File Resolution:**
- Paths are relative to the configuration file location
- Both absolute and relative paths are supported
- If a file is missing, the literal text (e.g., `@missing.md`) remains in the prompt
- A warning is logged for missing files

**Best Practices:**
- Store context files in `.claude/contexts/` directory
- Use descriptive filenames
- Keep context files focused on specific topics
- Version control your context files

## Multiple Matches

When multiple rules match a single prompt, all matching contexts are concatenated:

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "sidebar"
      prompt: "Context A: Sidebar guidelines"

    - pattern: "react"
      prompt: "Context B: React best practices"
```

**User prompt:** "update the react sidebar component"

**Result:** Both rules match, so Claude receives:
```
Context A: Sidebar guidelines
Context B: React best practices
```

The contexts are prepended in the order they appear in the configuration.

## Notifications

When notifications are enabled for the `UserPromptSubmit` hook, you'll receive alerts when context is injected:

```yaml
notifications:
  enabled: true
  hooks:
    - "UserPromptSubmit"
  showSuccess: true
```

**Notification behavior:**
- When rules match: Notification shows which rules matched and injected context
- When no rules match: Standard success notification (if `showSuccess: true`)

## Use Cases

### Enforce Code Style Guidelines

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "(?i)typescript|javascript|\.ts|\.js"
      prompt: |
        Follow the code style guide: @.claude/contexts/style-guide.md
        - Use functional components
        - Prefer composition over inheritance
        - Add JSDoc comments for exported functions
      caseInsensitive: true
```

### Security Reminders

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "security|auth|password|token|api.?key"
      prompt: |
        SECURITY CHECKLIST:
        - Never commit secrets or API keys
        - Use environment variables for sensitive data
        - Implement proper input validation
        - Review: @.claude/contexts/security-guidelines.md
```

### Domain-Specific Knowledge

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "payment|stripe|checkout|billing"
      prompt: |
        Payment processing guidelines: @.claude/contexts/payments.md

        Remember:
        - Always use idempotency keys
        - Handle webhooks properly
        - Test in sandbox mode first
```

### Documentation Enforcement

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "(?i)function|class|export"
      prompt: |
        Document all exported functions and classes.
        See documentation standards: @.claude/contexts/docs.md
      caseInsensitive: true
```

## Complete Example

Here's a complete configuration for a React project with TypeScript:

```yaml
userPromptSubmit:
  contextRules:
    # Component development
    - pattern: "component|jsx|tsx"
      prompt: |
        React component guidelines: @.claude/contexts/components.md
        - Use TypeScript for all components
        - Include PropTypes or interface definitions
        - Add unit tests for interactive components
      caseInsensitive: true

    # State management
    - pattern: "state|redux|zustand|context"
      prompt: |
        State management patterns: @.claude/contexts/state.md
      caseInsensitive: true

    # API integration
    - pattern: "api|fetch|axios|endpoint"
      prompt: |
        API integration guide: @.claude/contexts/api.md
        - Use React Query for data fetching
        - Handle loading and error states
        - Implement proper error boundaries
      caseInsensitive: true

    # Testing
    - pattern: "test|spec|jest|vitest"
      prompt: |
        Testing standards: @.claude/contexts/testing.md
        - Aim for 80%+ code coverage
        - Write integration tests for user flows
        - Mock external API calls
      caseInsensitive: true

    # Accessibility
    - pattern: "(?i)a11y|accessibility|aria|wcag"
      prompt: |
        Accessibility requirements: @.claude/contexts/a11y.md
        - Use semantic HTML
        - Include ARIA labels
        - Test with screen readers
      caseInsensitive: true

# Enable notifications to see when context is injected
notifications:
  enabled: true
  hooks:
    - "UserPromptSubmit"
  showSuccess: true
```

## See Also

- [Configuration Overview](/reference/config/configuration/) - Complete reference for all configuration options
- [Hooks Overview](/guides/hooks/) - Understanding the conclaude hook system
