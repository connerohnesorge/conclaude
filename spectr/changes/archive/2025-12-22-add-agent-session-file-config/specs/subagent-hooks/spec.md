## ADDED Requirements

### Requirement: Agent Session File Preservation Configuration

The system SHALL provide a `preserveAgentSessionFiles` boolean configuration option in the `subagentStop` section to control whether agent session files are deleted after subagent completion.

#### Scenario: Preserve agent session files enabled

- **GIVEN** configuration contains `subagentStop.preserveAgentSessionFiles: true`
- **WHEN** a SubagentStop hook fires
- **THEN** the agent session file (`/tmp/conclaude-agent-{session_id}.json`) SHALL NOT be deleted
- **AND** the file SHALL remain available for debugging purposes
- **AND** the SubagentStop hook SHALL complete successfully

#### Scenario: Preserve agent session files disabled (default)

- **GIVEN** configuration contains `subagentStop.preserveAgentSessionFiles: false`
- **OR** the `preserveAgentSessionFiles` field is not specified
- **WHEN** a SubagentStop hook fires
- **THEN** the agent session file SHALL be deleted
- **AND** cleanup errors SHALL be logged (not silently ignored)
- **AND** the SubagentStop hook SHALL complete successfully regardless of cleanup outcome

#### Scenario: Configuration default value

- **WHEN** `subagentStop.preserveAgentSessionFiles` is not specified in configuration
- **THEN** the system SHALL default to `false`
- **AND** agent session files SHALL be deleted after SubagentStop
- **AND** existing behavior is maintained (backward compatible)

#### Scenario: Configuration validation

- **WHEN** `subagentStop.preserveAgentSessionFiles` contains a boolean value (`true` or `false`)
- **THEN** the configuration SHALL be accepted as valid
- **AND** the setting SHALL be applied during SubagentStop hook execution

#### Scenario: Invalid configuration value

- **WHEN** `subagentStop.preserveAgentSessionFiles` contains a non-boolean value (e.g., `"yes"`, `1`, `null`)
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the type mismatch and expected boolean value

### Requirement: Agent Session File Cleanup Error Handling

The system SHALL properly handle and log errors that occur during agent session file cleanup, rather than silently ignoring them.

#### Scenario: Cleanup error is logged

- **GIVEN** configuration has `preserveAgentSessionFiles: false` (or unset)
- **WHEN** SubagentStop attempts to delete the agent session file
- **AND** the deletion fails (permission denied, file locked, etc.)
- **THEN** the error SHALL be logged to stderr with a warning message
- **AND** the warning SHALL include the file path and error reason
- **AND** the SubagentStop hook SHALL still complete successfully

#### Scenario: File not found during cleanup

- **GIVEN** configuration has `preserveAgentSessionFiles: false`
- **WHEN** SubagentStop attempts to delete the agent session file
- **AND** the file does not exist
- **THEN** no error or warning SHALL be logged
- **AND** this is treated as a normal condition (file may have been cleaned up already)
- **AND** the SubagentStop hook SHALL complete successfully

#### Scenario: Successful cleanup logs at debug level

- **GIVEN** configuration has `preserveAgentSessionFiles: false`
- **WHEN** SubagentStop successfully deletes the agent session file
- **THEN** a debug-level log message MAY be emitted indicating successful cleanup
- **AND** the SubagentStop hook SHALL complete successfully

#### Scenario: Cleanup never blocks hook completion

- **GIVEN** any configuration value for `preserveAgentSessionFiles`
- **WHEN** SubagentStop hook fires
- **THEN** cleanup operations SHALL NOT cause the hook to fail
- **AND** cleanup errors SHALL NOT prevent notification sending
- **AND** cleanup errors SHALL NOT affect the hook's return status
