# execution Specification

## Purpose
TBD - created by archiving change add-output-limiting. Update Purpose after archive.
## Requirements

### Requirement: Command Output Line Limiting Configuration
The system SHALL provide an optional maxOutputLines field for individual stop commands in the configuration to limit stdout and stderr output.

#### Scenario: Command with maxOutputLines configured
- **WHEN** a stop command includes maxOutputLines field set to 10
- **THEN** stdout and stderr SHALL each be limited to 10 lines maximum
- **AND** output exceeding the limit SHALL be truncated with a clear indicator

#### Scenario: Command without maxOutputLines configured
- **WHEN** a stop command does not include a maxOutputLines field
- **THEN** the command output SHALL be displayed in full without truncation
- **AND** existing behavior SHALL be preserved for backward compatibility

### Requirement: Output Truncation Enforcement
The system SHALL truncate command output when it exceeds the configured line limit and provide clear indication of truncation.

#### Scenario: Stdout exceeds line limit
- **WHEN** a command produces stdout exceeding the maxOutputLines limit
- **THEN** only the first N lines SHALL be displayed
- **AND** a truncation indicator SHALL be appended (e.g., "... (123 lines omitted)")
- **AND** the omitted line count SHALL be accurate

#### Scenario: Stderr exceeds line limit
- **WHEN** a command produces stderr exceeding the maxOutputLines limit
- **THEN** only the first N lines SHALL be displayed
- **AND** a truncation indicator SHALL be appended (e.g., "... (45 lines omitted)")
- **AND** the omitted line count SHALL be accurate

#### Scenario: Output within line limit
- **WHEN** a command produces output within the maxOutputLines limit
- **THEN** all output lines SHALL be displayed
- **AND** no truncation indicator SHALL be shown

#### Scenario: Independent stdout and stderr limiting
- **WHEN** both stdout and stderr are produced by a command with maxOutputLines configured
- **THEN** stdout SHALL be limited to maxOutputLines independently
- **AND** stderr SHALL be limited to maxOutputLines independently
- **AND** truncation SHALL be applied separately to each stream

### Requirement: Configuration Validation
The system SHALL validate maxOutputLines values in the configuration to ensure they are properly formatted and reasonable.

#### Scenario: Valid maxOutputLines value
- **WHEN** maxOutputLines field contains a positive integer value
- **THEN** the configuration SHALL be accepted
- **AND** the limit SHALL be applied during command execution

#### Scenario: Invalid maxOutputLines value
- **WHEN** maxOutputLines field contains a non-numeric value, zero, or negative number
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the maxOutputLines value format issue

#### Scenario: Reasonable value range
- **WHEN** maxOutputLines field contains a value between 1 and 10000
- **THEN** the configuration SHALL be accepted
- **AND** values outside this range SHALL be rejected with a helpful error message

### Requirement: Output Limiting with showStdout and showStderr
The system SHALL apply output limiting only when the respective output stream is being shown to the user or Claude. The system SHALL NOT output command stdout/stderr to the console (via eprintln or similar) when the respective show flags are disabled.

#### Scenario: maxOutputLines with showStdout disabled
- **WHEN** maxOutputLines is configured but showStdout is false
- **THEN** stdout SHALL NOT be shown to the user or Claude in the error message
- **AND** stdout SHALL NOT be printed to the console in diagnostic logging
- **AND** maxOutputLines SHALL have no effect on stdout

#### Scenario: maxOutputLines with showStderr disabled
- **WHEN** maxOutputLines is configured but showStderr is false
- **THEN** stderr SHALL NOT be shown to the user or Claude in the error message
- **AND** stderr SHALL NOT be printed to the console in diagnostic logging
- **AND** maxOutputLines SHALL have no effect on stderr

#### Scenario: maxOutputLines with both outputs enabled
- **WHEN** maxOutputLines is configured with showStdout true and showStderr true
- **THEN** both stdout and stderr SHALL be limited to maxOutputLines in the error message
- **AND** both stdout and stderr MAY be included in diagnostic logging to the console
- **AND** truncation indicators SHALL be shown for each stream if exceeded

#### Scenario: Diagnostic logging respects output flags
- **WHEN** a stop command fails and writes diagnostic information via eprintln
- **THEN** stdout SHALL only be included in the diagnostic output if showStdout is true
- **AND** stderr SHALL only be included in the diagnostic output if showStderr is true
- **AND** command name and exit code SHALL always be included regardless of flags
- **AND** placeholder text like "(output hidden by configuration)" MAY be shown when flags are false

### Requirement: Command Working Directory

The system SHALL execute all configured commands with the current working directory set to the directory containing the configuration file, not the directory from which Claude Code was invoked.

#### Scenario: Stop command executes from config directory

- **WHEN** a stop command is configured in `.conclaude.yaml` located at `/home/user/project/.conclaude.yaml`
- **AND** Claude Code session runs from `/home/user/project/src/nested`
- **THEN** the stop command SHALL execute with cwd set to `/home/user/project/`
- **AND** relative paths in commands like `npm test` SHALL resolve from `/home/user/project/`

#### Scenario: Subagent stop command executes from config directory

- **WHEN** a subagent stop command is configured in `.conclaude.yaml` located at `/home/user/project/.conclaude.yaml`
- **AND** Claude Code session runs from `/home/user/project/deep/nested/dir`
- **THEN** the subagent stop command SHALL execute with cwd set to `/home/user/project/`
- **AND** the command SHALL have access to project files relative to the config location

#### Scenario: Config in current directory

- **WHEN** the configuration file is in the same directory as the Claude Code session
- **THEN** commands SHALL execute from that directory
- **AND** behavior SHALL be identical to previous implementation for this case

#### Scenario: Commands with absolute paths

- **WHEN** a command uses an absolute path (e.g., `/usr/bin/custom-linter`)
- **THEN** the command SHALL execute successfully regardless of working directory
- **AND** the working directory SHALL still be set to the config directory

### Requirement: Config Directory Environment Variable

The system SHALL expose the configuration file's parent directory as an environment variable to all executed commands.

#### Scenario: CONCLAUDE_CONFIG_DIR available to stop commands

- **WHEN** a stop command executes
- **THEN** the environment variable `CONCLAUDE_CONFIG_DIR` SHALL be set
- **AND** its value SHALL be the absolute path to the directory containing `.conclaude.yaml`

#### Scenario: CONCLAUDE_CONFIG_DIR available to subagent stop commands

- **WHEN** a subagent stop command executes
- **THEN** the environment variable `CONCLAUDE_CONFIG_DIR` SHALL be set
- **AND** its value SHALL be the absolute path to the directory containing `.conclaude.yaml`

#### Scenario: Script uses CONCLAUDE_CONFIG_DIR

- **WHEN** a command script references `$CONCLAUDE_CONFIG_DIR`
- **THEN** it SHALL resolve to the config file's parent directory
- **AND** the script MAY use it for explicit path construction (e.g., `$CONCLAUDE_CONFIG_DIR/scripts/lint.sh`)

### Requirement: Command Timeout Configuration
The system SHALL provide an optional timeout field for individual stop commands in the configuration.

#### Scenario: Command with timeout configured
- **WHEN** a stop command includes a timeout field set to 30 seconds
- **THEN** the command execution SHALL terminate after 30 seconds if not completed
- **AND** the hook SHALL be blocked with an appropriate timeout error message

#### Scenario: Command without timeout configured
- **WHEN** a stop command does not include a timeout field
- **THEN** the command execution SHALL continue without timeout restrictions
- **AND** existing behavior SHALL be preserved for backward compatibility

### Requirement: Timeout Enforcement
The system SHALL enforce command timeouts during execution and provide clear error messaging.

#### Scenario: Command execution exceeds timeout
- **WHEN** a command execution time exceeds the configured timeout duration
- **THEN** the process SHALL be terminated
- **AND** the hook SHALL return a blocked result with a timeout error message
- **AND** the error message SHALL include the timeout duration and command that timed out

#### Scenario: Command completes within timeout
- **WHEN** a command completes execution within the configured timeout duration
- **THEN** the command SHALL proceed normally
- **AND** success or failure SHALL be determined by the command's exit code
- **AND** timeout logic SHALL not interfere with normal command completion

### Requirement: Timeout Configuration Validation
The system SHALL validate timeout values in the configuration to ensure they are properly formatted and reasonable.

#### Scenario: Valid timeout value
- **WHEN** a timeout field contains a positive integer value in seconds
- **THEN** the configuration SHALL be accepted
- **AND** the timeout SHALL be applied during command execution

#### Scenario: Invalid timeout value
- **WHEN** a timeout field contains a non-numeric value or negative number
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the timeout value format issue

### Requirement: Command Execution Display Configuration

The system SHALL provide an optional `showCommand` field for individual stop commands in the configuration to control whether the command being executed is displayed in stdout.

#### Scenario: Command with showCommand explicitly set to true

- **WHEN** a stop command includes `showCommand` field set to `true`
- **THEN** the system SHALL print "Executing command X/Y: <command>" to stdout before execution
- **AND** this matches the current default behavior

#### Scenario: Command with showCommand set to false

- **WHEN** a stop command includes `showCommand` field set to `false`
- **THEN** the system SHALL NOT print "Executing command X/Y: <command>" to stdout
- **AND** the command SHALL still execute normally
- **AND** command output (stdout/stderr) SHALL still be controlled by `showStdout`/`showStderr` flags independently

#### Scenario: Command without showCommand configured

- **WHEN** a stop command does not include a `showCommand` field
- **THEN** the system SHALL default to `true`
- **AND** the system SHALL print "Executing command X/Y: <command>" to stdout
- **AND** existing behavior SHALL be preserved for backward compatibility

#### Scenario: showCommand with subagent stop commands

- **WHEN** a subagent stop command includes `showCommand` field
- **THEN** the same display control behavior SHALL apply as with regular stop commands
- **AND** the subagent stop command execution line SHALL be suppressed when `showCommand` is `false`

### Requirement: showCommand Configuration Validation

The system SHALL validate `showCommand` values in the configuration to ensure they are properly formatted.

#### Scenario: Valid showCommand value

- **WHEN** `showCommand` field contains a boolean value (`true` or `false`)
- **THEN** the configuration SHALL be accepted
- **AND** the display setting SHALL be applied during command execution

#### Scenario: Invalid showCommand value

- **WHEN** `showCommand` field contains a non-boolean value
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the `showCommand` value format issue

### Requirement: Per-Command Notification Integration

The system SHALL integrate per-command notifications into the command execution pipeline for both stop and subagent stop commands.

#### Scenario: Stop command execution with per-command notifications

- **WHEN** a stop command has `notifyPerCommand: true`
- **THEN** the system SHALL call `send_notification()` before command execution
- **AND** the notification SHALL use the hook name "Stop" with context indicating command start
- **AND** the system SHALL call `send_notification()` after command execution completes
- **AND** the notification status SHALL be "success" or "failure" based on exit code

#### Scenario: Subagent stop command execution with per-command notifications

- **WHEN** a subagent stop command has `notifyPerCommand: true`
- **THEN** the system SHALL call `send_notification()` before command execution
- **AND** the notification SHALL use the hook name "SubagentStop" with context indicating command start
- **AND** the system SHALL call `send_notification()` after command execution completes
- **AND** the notification status SHALL be "success" or "failure" based on exit code

#### Scenario: Command execution without per-command notifications

- **WHEN** a command has `notifyPerCommand: false` or omits the field
- **THEN** the system SHALL NOT call `send_notification()` for command start
- **AND** the system SHALL NOT call `send_notification()` for command completion
- **AND** only hook-level notifications SHALL be sent

### Requirement: Per-Command Notification Message Formatting

The system SHALL format per-command notification messages based on command configuration.

#### Scenario: Notification message with showCommand enabled

- **WHEN** generating a per-command notification for a command with `showCommand: true`
- **THEN** the start notification context SHALL include the command string (e.g., "Running: npm test")
- **AND** the completion notification context SHALL include the command string and status (e.g., "Completed: npm test")

#### Scenario: Notification message with showCommand disabled

- **WHEN** generating a per-command notification for a command with `showCommand: false`
- **THEN** the start notification context SHALL use a generic format (e.g., "Running command 2/5")
- **AND** the completion notification context SHALL use a generic format (e.g., "Command 2/5 completed")
- **AND** the command string SHALL NOT appear in the notification

#### Scenario: Notification message includes command index

- **WHEN** generating a per-command notification for any command
- **THEN** the notification context MAY include the command index and total count
- **AND** the format SHALL be "X/Y" where X is the current command number and Y is total commands

### Requirement: Per-Command Notification Configuration Propagation

The system SHALL propagate the `notifyPerCommand` configuration flag through the command execution pipeline.

#### Scenario: notifyPerCommand flag in StopCommandConfig

- **WHEN** collecting stop commands from configuration
- **THEN** the `StopCommandConfig` struct SHALL include a `notify_per_command` boolean field
- **AND** the field SHALL be populated from the command configuration's `notifyPerCommand` value
- **AND** the field SHALL default to `false` when not specified

#### Scenario: notifyPerCommand flag in SubagentStopCommandConfig

- **WHEN** collecting subagent stop commands from configuration
- **THEN** the `SubagentStopCommandConfig` struct SHALL include a `notify_per_command` boolean field
- **AND** the field SHALL be populated from the command configuration's `notifyPerCommand` value
- **AND** the field SHALL default to `false` when not specified

#### Scenario: notifyPerCommand flag during execution

- **WHEN** executing commands in `execute_stop_commands()` or `execute_subagent_stop_commands()`
- **THEN** the execution loop SHALL check the `notify_per_command` flag for each command
- **AND** notifications SHALL be sent conditionally based on this flag
