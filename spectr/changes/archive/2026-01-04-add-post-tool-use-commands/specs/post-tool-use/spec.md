# post-tool-use Specification

## Purpose

Define the PostToolUse hook command execution system that enables read-only observation of tool results for logging, documentation, and external integration purposes.

## ADDED Requirements

### Requirement: PostToolUse Configuration Section

The system SHALL support a `postToolUse` configuration section in `.conclaude.yaml` for defining commands to execute after tool completion.

#### Scenario: Valid postToolUse configuration with commands

- **GIVEN** a `.conclaude.yaml` file contains:
  ```yaml
  postToolUse:
    commands:
      - run: "echo tool completed"
  ```
- **WHEN** the configuration is loaded
- **THEN** the configuration SHALL be accepted as valid
- **AND** the commands array SHALL be populated with the specified command

#### Scenario: Empty postToolUse configuration

- **GIVEN** a `.conclaude.yaml` file contains:
  ```yaml
  postToolUse:
    commands: []
  ```
- **WHEN** the configuration is loaded
- **THEN** the configuration SHALL be accepted as valid
- **AND** no commands SHALL execute on PostToolUse events

#### Scenario: Missing postToolUse section uses defaults

- **GIVEN** a `.conclaude.yaml` file does not contain a `postToolUse` section
- **WHEN** the configuration is loaded
- **THEN** the system SHALL use an empty commands array as default
- **AND** PostToolUse events SHALL proceed without command execution

### Requirement: PostToolUse Command Structure

The system SHALL support command configuration with tool filtering and execution options consistent with other hook command types.

#### Scenario: Command with all options specified

- **GIVEN** a postToolUse command configuration:
  ```yaml
  postToolUse:
    commands:
      - run: ".claude/scripts/log.sh"
        tool: "AskUserQuestion"
        showCommand: true
        showStdout: true
        showStderr: false
        maxOutputLines: 100
        timeout: 30
  ```
- **WHEN** the configuration is loaded
- **THEN** all specified options SHALL be preserved
- **AND** the command SHALL only execute for AskUserQuestion tool events

#### Scenario: Command with minimal options uses defaults

- **GIVEN** a postToolUse command configuration with only `run`:
  ```yaml
  postToolUse:
    commands:
      - run: "echo completed"
  ```
- **WHEN** the configuration is loaded
- **THEN** `tool` SHALL default to `"*"` (all tools)
- **AND** `showCommand` SHALL default to `true`
- **AND** `showStdout` SHALL default to `false`
- **AND** `showStderr` SHALL default to `false`
- **AND** `timeout` SHALL default to no timeout
- **AND** `maxOutputLines` SHALL default to no limit

### Requirement: Tool Pattern Filtering

The system SHALL filter command execution based on glob pattern matching against the tool name.

#### Scenario: Exact tool name match

- **GIVEN** a command with `tool: "AskUserQuestion"`
- **WHEN** a PostToolUse event fires for tool "AskUserQuestion"
- **THEN** the command SHALL execute
- **AND** when a PostToolUse event fires for tool "Bash"
- **THEN** the command SHALL NOT execute

#### Scenario: Wildcard matches all tools

- **GIVEN** a command with `tool: "*"`
- **WHEN** a PostToolUse event fires for any tool
- **THEN** the command SHALL execute

#### Scenario: Glob pattern with asterisk

- **GIVEN** a command with `tool: "*Search*"`
- **WHEN** a PostToolUse event fires for tool "WebSearch"
- **THEN** the command SHALL execute
- **AND** when a PostToolUse event fires for tool "Grep"
- **THEN** the command SHALL NOT execute

#### Scenario: Multiple commands with different filters

- **GIVEN** postToolUse configuration:
  ```yaml
  postToolUse:
    commands:
      - tool: "AskUserQuestion"
        run: "echo qa"
      - tool: "*"
        run: "echo all"
  ```
- **WHEN** a PostToolUse event fires for tool "AskUserQuestion"
- **THEN** both commands SHALL execute
- **AND** when a PostToolUse event fires for tool "Bash"
- **THEN** only the second command SHALL execute

### Requirement: PostToolUse Environment Variables

The system SHALL expose tool execution data via environment variables to commands.

#### Scenario: CONCLAUDE_TOOL_NAME is set

- **WHEN** a postToolUse command executes
- **THEN** `CONCLAUDE_TOOL_NAME` SHALL be set to the name of the tool that completed
- **AND** the value SHALL match the tool_name from the PostToolUse payload

#### Scenario: CONCLAUDE_TOOL_INPUT is set

- **WHEN** a postToolUse command executes
- **THEN** `CONCLAUDE_TOOL_INPUT` SHALL be set to a JSON string
- **AND** the JSON SHALL represent the tool_input HashMap from the payload

#### Scenario: CONCLAUDE_TOOL_OUTPUT is set

- **WHEN** a postToolUse command executes
- **THEN** `CONCLAUDE_TOOL_OUTPUT` SHALL be set to a JSON string
- **AND** the JSON SHALL represent the tool_response from the payload

#### Scenario: CONCLAUDE_TOOL_TIMESTAMP is set

- **WHEN** a postToolUse command executes
- **THEN** `CONCLAUDE_TOOL_TIMESTAMP` SHALL be set to an ISO 8601 formatted timestamp
- **AND** the timestamp SHALL represent the time of command execution

#### Scenario: CONCLAUDE_TOOL_USE_ID is set when available

- **GIVEN** a PostToolUse payload with tool_use_id present
- **WHEN** a postToolUse command executes
- **THEN** `CONCLAUDE_TOOL_USE_ID` SHALL be set to the tool_use_id value
- **AND** this enables correlation with PreToolUse events

#### Scenario: Standard session environment variables are set

- **WHEN** a postToolUse command executes
- **THEN** `CONCLAUDE_SESSION_ID` SHALL be set from payload.base.session_id
- **AND** `CONCLAUDE_CWD` SHALL be set from payload.base.cwd
- **AND** `CONCLAUDE_CONFIG_DIR` SHALL be set to the config file directory

### Requirement: Read-Only Hook Semantics

The system SHALL treat PostToolUse as a read-only observation hook that cannot block or modify tool execution.

#### Scenario: Hook always returns success

- **GIVEN** any postToolUse command configuration
- **WHEN** the PostToolUse hook handler completes
- **THEN** it SHALL return a success HookResult
- **AND** it SHALL NOT return a blocked result

#### Scenario: Command failure does not block

- **GIVEN** a postToolUse command that exits with non-zero status
- **WHEN** the command fails during PostToolUse processing
- **THEN** the failure SHALL be logged to stderr
- **AND** the hook SHALL still return success
- **AND** subsequent commands SHALL continue executing

#### Scenario: Command timeout does not block

- **GIVEN** a postToolUse command with `timeout: 5`
- **WHEN** the command exceeds the 5 second timeout
- **THEN** the command SHALL be terminated
- **AND** a timeout message SHALL be logged to stderr
- **AND** the hook SHALL still return success

### Requirement: Command Execution from Config Directory

The system SHALL execute postToolUse commands with the current working directory set to the configuration file's directory.

#### Scenario: Command executes from config directory

- **GIVEN** `.conclaude.yaml` is located at `/home/user/project/.conclaude.yaml`
- **AND** a postToolUse command `run: "./scripts/log.sh"`
- **WHEN** the command executes
- **THEN** the working directory SHALL be `/home/user/project/`
- **AND** relative paths SHALL resolve from that directory

### Requirement: Configuration Validation

The system SHALL validate postToolUse configuration values.

#### Scenario: Valid timeout value accepted

- **GIVEN** a postToolUse command with `timeout: 300`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL succeed
- **AND** the timeout SHALL be applied during execution

#### Scenario: Invalid timeout value rejected

- **GIVEN** a postToolUse command with `timeout: 5000` (exceeds max 3600)
- **WHEN** the configuration is loaded
- **THEN** validation SHALL fail with an error message
- **AND** the error SHALL indicate the valid range (1-3600)

#### Scenario: Valid maxOutputLines value accepted

- **GIVEN** a postToolUse command with `maxOutputLines: 500`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL succeed

#### Scenario: Invalid maxOutputLines value rejected

- **GIVEN** a postToolUse command with `maxOutputLines: 0`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL fail with an error message
- **AND** the error SHALL indicate the valid range (1-10000)

#### Scenario: Invalid tool pattern rejected

- **GIVEN** a postToolUse command with `tool: "[invalid"`
- **WHEN** the configuration is loaded
- **THEN** validation SHALL fail with an error message
- **AND** the error SHALL indicate the glob pattern syntax issue
