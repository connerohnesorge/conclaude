## ADDED Requirements

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