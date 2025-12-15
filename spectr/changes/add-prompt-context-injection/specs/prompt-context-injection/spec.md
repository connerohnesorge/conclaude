# Prompt Context Injection Specification

## Purpose

Defines the configuration and behavior for conditionally injecting context or instructions into Claude's system prompt based on pattern matching against user-submitted prompts.

## ADDED Requirements

### Requirement: Context Injection Rule Configuration

The system SHALL support configuring context injection rules that match user prompts and prepend context to Claude's system prompt.

#### Scenario: Basic pattern matching

- **WHEN** a user submits a prompt containing "sidebar"
- **AND** a context rule exists with pattern `sidebar`
- **THEN** the configured prompt/context SHALL be prepended to Claude's response context

#### Scenario: Regex pattern matching

- **WHEN** a user submits a prompt containing "authentication"
- **AND** a context rule exists with pattern `auth|login|authentication`
- **THEN** the configured prompt/context SHALL be prepended to Claude's response context

#### Scenario: Case-insensitive matching

- **WHEN** a context rule has `caseInsensitive: true`
- **AND** the pattern is `database`
- **AND** the user prompt contains "DATABASE" or "Database"
- **THEN** the rule SHALL match and inject the configured context

#### Scenario: Disabled rule

- **WHEN** a context rule has `enabled: false`
- **AND** the user prompt matches the pattern
- **THEN** the rule SHALL NOT inject any context

### Requirement: Multiple Pattern Matches

The system SHALL support multiple context rules matching a single prompt and concatenate their contexts in configuration order.

#### Scenario: Two rules match same prompt

- **WHEN** a user submits a prompt "update the auth sidebar"
- **AND** rule A with pattern `sidebar` is configured first
- **AND** rule B with pattern `auth` is configured second
- **THEN** rule A's context SHALL be prepended first
- **AND** rule B's context SHALL be prepended second

#### Scenario: No rules match

- **WHEN** a user submits a prompt
- **AND** no context rules match the prompt
- **THEN** the hook SHALL return success with no system_prompt modification

### Requirement: File Reference Expansion

The system SHALL support expanding `@path/to/file` references in context prompts to the file's contents.

#### Scenario: Valid file reference

- **WHEN** a context rule contains `@.claude/contexts/sidebar.md` in the prompt
- **AND** the file `.claude/contexts/sidebar.md` exists relative to the config file
- **THEN** the `@.claude/contexts/sidebar.md` reference SHALL be replaced with the file's contents

#### Scenario: Missing file reference

- **WHEN** a context rule contains `@missing-file.md` in the prompt
- **AND** the file does not exist
- **THEN** the system SHALL log a warning
- **AND** the reference SHALL remain as literal text `@missing-file.md`

#### Scenario: Multiple file references

- **WHEN** a context rule contains multiple `@file` references
- **THEN** each reference SHALL be expanded independently

### Requirement: Configuration Schema

The system SHALL define a JSON Schema for the `userPromptSubmit` configuration section.

#### Scenario: Valid configuration

- **WHEN** a configuration includes:
  ```yaml
  userPromptSubmit:
    contextRules:
      - pattern: "sidebar"
        prompt: "Read the sidebar docs"
        enabled: true
        caseInsensitive: false
  ```
- **THEN** the configuration SHALL parse successfully
- **AND** the rule SHALL be available for prompt matching

#### Scenario: Invalid regex pattern

- **WHEN** a configuration includes a context rule with invalid regex pattern `[invalid`
- **THEN** configuration loading SHALL fail with a descriptive error message
- **AND** the error SHALL indicate which pattern is invalid

#### Scenario: Missing required fields

- **WHEN** a context rule is missing the required `pattern` field
- **THEN** configuration loading SHALL fail with a descriptive error message

### Requirement: Hook Result Extension

The system SHALL extend the `HookResult` struct to support returning system prompt context.

#### Scenario: Context injection via HookResult

- **WHEN** `handle_user_prompt_submit()` processes matching rules
- **THEN** it SHALL return a `HookResult` with `system_prompt` field set
- **AND** the `blocked` field SHALL be `false`
- **AND** the `message` field SHALL remain unset (or contain log info)

#### Scenario: HookResult backward compatibility

- **WHEN** existing code uses `HookResult::success()` or `HookResult::blocked()`
- **THEN** the existing behavior SHALL be unchanged
- **AND** `system_prompt` SHALL default to `None`

### Requirement: Notification Integration

The system SHALL send notifications for context injection when notifications are enabled.

#### Scenario: Context injected notification

- **WHEN** one or more context rules match a user prompt
- **AND** notifications are enabled for `UserPromptSubmit` hook
- **THEN** a notification SHALL be sent indicating context was injected
- **AND** the notification SHALL list which rules matched

#### Scenario: No match notification

- **WHEN** no context rules match a user prompt
- **AND** notifications are enabled with `showSuccess: true`
- **THEN** a standard success notification SHALL be sent
