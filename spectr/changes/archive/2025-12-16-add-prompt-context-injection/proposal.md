# Change: Add Prompt Context Injection for UserPromptSubmit Hook

## Why

Currently, the `UserPromptSubmit` hook only validates and logs user prompts without providing any mechanism to conditionally inject context or guidance based on prompt content. Users need a way to automatically prepend context or instructions to Claude when their prompts match certain patterns - for example, automatically reminding Claude to read specific context files when discussing certain topics.

This feature enables workflow-aware context injection, allowing teams to enforce project-specific guidelines, reference documentation, or add domain-specific reminders based on what the user is asking about.

**Inspiration**: [YouTube video demonstrating the concept](https://youtu.be/iKwRWwabkEc?si=5QjMF1Mm0jRifgsQ)

## What Changes

- **ADDED**: New `userPromptSubmit` configuration section for defining context injection rules
- **ADDED**: Pattern matching on user prompt text (supports regex patterns)
- **ADDED**: Conditional prompt/context prepending when patterns match
- **ADDED**: Support for referencing context files via `@` syntax (e.g., `@.claude/contexts/sidebar.md`)
- **MODIFIED**: `handle_user_prompt_submit()` function to process context injection rules
- **MODIFIED**: Hook result to support returning modified/augmented prompts

## Impact

- Affected specs: `hooks-system`, new `prompt-context-injection` capability
- Affected code:
  - `src/config.rs` - New configuration structs
  - `src/hooks.rs` - Updated `handle_user_prompt_submit()` handler
  - `src/types.rs` - Extended `HookResult` if needed
  - `conclaude-schema.json` - Schema for new config section
  - `src/default-config.yaml` - Default config template

## Example Configuration

```yaml
userPromptSubmit:
  contextRules:
    # Pattern-based context injection
    - pattern: "sidebar"
      prompt: |
        Make sure to read @.claude/contexts/sidebar.md before proceeding.

    # Multiple patterns can trigger the same context
    - pattern: "auth|login|authentication"
      prompt: |
        Review the authentication patterns in @.claude/contexts/auth.md

    # Case-insensitive matching with custom message
    - pattern: "(?i)database|sql|query"
      prompt: |
        Follow the database conventions in @.claude/contexts/database.md
        Remember to use parameterized queries for security.
```

## Non-Goals

- This change does NOT block or reject prompts (that would be a separate feature)
- This change does NOT modify the original prompt, only prepends context
- This change does NOT support async context file loading (files must exist at config time)
