# Prompt Context Injection Specification

## Requirements

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

### Requirement: UserPromptSubmit Command Configuration

The system SHALL support a `commands` array in the `userPromptSubmit` configuration section for defining commands to execute when prompts are submitted.

#### Scenario: Valid userPromptSubmit configuration with commands

- **GIVEN** a `.conclaude.yaml` file contains:
  ```yaml
  userPromptSubmit:
    commands:
      - run: "echo prompt received"
  ```
- **WHEN** the configuration is loaded
- **THEN** the configuration SHALL be accepted as valid
- **AND** the commands array SHALL be populated with the specified command

#### Scenario: Commands coexist with contextRules

- **GIVEN** a `.conclaude.yaml` file contains:
  ```yaml
  userPromptSubmit:
    contextRules:
      - pattern: "auth"
        prompt: "Review auth docs"
    commands:
      - run: "echo logging"
  ```
- **WHEN** the configuration is loaded
- **THEN** both contextRules and commands SHALL be available
- **AND** both SHALL be processed when prompts are submitted

#### Scenario: Empty commands array

- **GIVEN** a `.conclaude.yaml` file contains:
  ```yaml
  userPromptSubmit:
    commands: []
  ```
- **WHEN** the configuration is loaded
- **THEN** the configuration SHALL be accepted as valid
- **AND** no commands SHALL execute on UserPromptSubmit events

#### Scenario: Missing commands field uses default

- **GIVEN** a `.conclaude.yaml` file does not contain a `commands` field in userPromptSubmit
- **WHEN** the configuration is loaded
- **THEN** the system SHALL use an empty commands array as default
- **AND** only contextRules processing SHALL occur

### Requirement: UserPromptSubmit Command Structure

The system SHALL support command configuration with pattern filtering and execution options consistent with other hook command types.

#### Scenario: Command with all options specified

- **GIVEN** a userPromptSubmit command configuration:
  ```yaml
  userPromptSubmit:
    commands:
      - run: ".claude/scripts/log.sh"
        pattern: "deploy|release"
        caseInsensitive: true
        showCommand: true
        showStdout: true
        showStderr: false
        maxOutputLines: 100
        timeout: 30
  ```
- **WHEN** the configuration is loaded
- **THEN** all specified options SHALL be preserved
- **AND** the command SHALL only execute for prompts matching the pattern

#### Scenario: Command with minimal options uses defaults

- **GIVEN** a userPromptSubmit command configuration with only `run`:
  ```yaml
  userPromptSubmit:
    commands:
      - run: "echo received"
  ```
- **WHEN** the configuration is loaded
- **THEN** `pattern` SHALL default to matching all prompts
- **AND** `caseInsensitive` SHALL default to `false`
- **AND** `showCommand` SHALL default to `true`
- **AND** `showStdout` SHALL default to `false`
- **AND** `showStderr` SHALL default to `false`
- **AND** `timeout` SHALL default to no timeout
- **AND** `maxOutputLines` SHALL default to no limit

### Requirement: Prompt Pattern Filtering for Commands

The system SHALL filter command execution based on regex pattern matching against the user prompt.

#### Scenario: Regex pattern match

- **GIVEN** a command with `pattern: "deploy|release|ship"`
- **WHEN** a UserPromptSubmit event fires with prompt "let's deploy to production"
- **THEN** the command SHALL execute
- **AND** when a UserPromptSubmit event fires with prompt "fix the bug"
- **THEN** the command SHALL NOT execute

#### Scenario: No pattern matches all prompts

- **GIVEN** a command without a `pattern` field
- **WHEN** a UserPromptSubmit event fires with any prompt
- **THEN** the command SHALL execute

#### Scenario: Case-insensitive pattern matching

- **GIVEN** a command with `pattern: "database"` and `caseInsensitive: true`
- **WHEN** a UserPromptSubmit event fires with prompt "Update DATABASE config"
- **THEN** the command SHALL execute
- **AND** when a UserPromptSubmit event fires with prompt "update database config"
- **THEN** the command SHALL execute

#### Scenario: Multiple commands with different patterns

- **GIVEN** userPromptSubmit configuration:
  ```yaml
  userPromptSubmit:
    commands:
      - pattern: "deploy"
        run: "echo deploy"
      - run: "echo all"
  ```
- **WHEN** a UserPromptSubmit event fires with prompt "deploy now"
- **THEN** both commands SHALL execute
- **AND** when a UserPromptSubmit event fires with prompt "fix bug"
- **THEN** only the second command SHALL execute

### Requirement: UserPromptSubmit Environment Variables

The system SHALL expose prompt data via environment variables to commands.

#### Scenario: CONCLAUDE_USER_PROMPT is set

- **WHEN** a userPromptSubmit command executes
- **THEN** `CONCLAUDE_USER_PROMPT` SHALL be set to the user's input text
- **AND** the value SHALL match the prompt from the UserPromptSubmit payload

#### Scenario: Standard session environment variables are set

- **WHEN** a userPromptSubmit command executes
- **THEN** `CONCLAUDE_SESSION_ID` SHALL be set from payload.base.session_id
- **AND** `CONCLAUDE_CWD` SHALL be set from payload.base.cwd
- **AND** `CONCLAUDE_CONFIG_DIR` SHALL be set to the config file directory
- **AND** `CONCLAUDE_HOOK_EVENT` SHALL be set to "UserPromptSubmit"

### Requirement: Read-Only Command Semantics

The system SHALL treat UserPromptSubmit commands as read-only observations that cannot block prompt processing.

#### Scenario: Hook returns success regardless of command results

- **GIVEN** any userPromptSubmit command configuration
- **WHEN** the UserPromptSubmit hook handler completes
- **THEN** it SHALL return a success HookResult (with or without context injection)
- **AND** it SHALL NOT return a blocked result due to command execution

#### Scenario: Command failure does not block

- **GIVEN** a userPromptSubmit command that exits with non-zero status
- **WHEN** the command fails during UserPromptSubmit processing
- **THEN** the failure SHALL be logged to stderr
- **AND** the hook SHALL still return success
- **AND** subsequent commands SHALL continue executing
- **AND** context injection SHALL still be returned if contextRules matched

#### Scenario: Command timeout does not block

- **GIVEN** a userPromptSubmit command with `timeout: 5`
- **WHEN** the command exceeds the 5 second timeout
- **THEN** the command SHALL be terminated
- **AND** a timeout message SHALL be logged to stderr
- **AND** the hook SHALL still return success

### Requirement: Command Execution Order

The system SHALL execute userPromptSubmit commands after contextRules processing.

#### Scenario: Commands execute after context rules

- **WHEN** a user submits a prompt
- **AND** both contextRules and commands are configured
- **THEN** contextRules SHALL be evaluated first
- **AND** context injection result SHALL be determined
- **AND** commands SHALL execute after contextRules processing
- **AND** context injection result SHALL be returned regardless of command outcomes

### Requirement: Command Execution from Config Directory

The system SHALL execute userPromptSubmit commands with the current working directory set to the configuration file's directory.

#### Scenario: Command executes from config directory

- **GIVEN** `.conclaude.yaml` is located at `/home/user/project/.conclaude.yaml`
- **AND** a userPromptSubmit command `run: "./scripts/log.sh"`
- **WHEN** the command executes
- **THEN** the working directory SHALL be `/home/user/project/`
- **AND** relative paths SHALL resolve from that directory

### Requirement: Command Configuration Validation

The system SHALL validate userPromptSubmit command configuration values.

#### Scenario: Valid pattern regex accepted

- **GIVEN** a userPromptSubmit command with `pattern: "deploy|release"`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL succeed

#### Scenario: Invalid pattern regex rejected

- **GIVEN** a userPromptSubmit command with `pattern: "[invalid"`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL fail with an error message
- **AND** the error SHALL indicate the regex syntax issue

#### Scenario: Valid timeout value accepted

- **GIVEN** a userPromptSubmit command with `timeout: 300`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL succeed
- **AND** the timeout SHALL be applied during execution

#### Scenario: Invalid timeout value rejected

- **GIVEN** a userPromptSubmit command with `timeout: 5000` (exceeds max 3600)
- **WHEN** the configuration is loaded
- **THEN** validation SHALL fail with an error message
- **AND** the error SHALL indicate the valid range (1-3600)

#### Scenario: Valid maxOutputLines value accepted

- **GIVEN** a userPromptSubmit command with `maxOutputLines: 500`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL succeed

#### Scenario: Invalid maxOutputLines value rejected

- **GIVEN** a userPromptSubmit command with `maxOutputLines: 0`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL fail with an error message
- **AND** the error SHALL indicate the valid range (1-10000)
