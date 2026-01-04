# notifications Specification

## Purpose
TBD - created by archiving change add-system-notifications. Update Purpose after archive.
## Requirements

### Requirement: System Notification Configuration

The system SHALL provide configuration options for enabling and customizing system notifications.

#### Scenario: Default configuration disables notifications

- **WHEN** no notification configuration is specified
- **THEN** the system SHALL NOT send any system notifications

#### Scenario: Enable notifications globally

- **WHEN** `notifications.enabled` is set to `true` in configuration
- **THEN** the system SHALL send notifications for all configured hooks

#### Scenario: Configure per-hook notifications

- **WHEN** `notifications.hooks` array specifies hook names (e.g., `["Stop", "PreToolUse"]`)
- **THEN** the system SHALL only send notifications for the specified hooks
- **AND** SHALL NOT send notifications for hooks not in the array

#### Scenario: All hooks notification

- **WHEN** `notifications.hooks` contains the wildcard `"*"`
- **THEN** the system SHALL send notifications for all hook types

### Requirement: Notification Content

The system SHALL send informative notification messages that include relevant context about hook execution.

#### Scenario: Successful hook execution notification

- **WHEN** a configured hook executes successfully
- **THEN** the notification title SHALL be "Conclaude - [HookName]"
- **AND** the notification body SHALL indicate success with brief context (e.g., "All checks passed")

#### Scenario: Failed hook execution notification

- **WHEN** a configured hook fails
- **THEN** the notification title SHALL be "Conclaude - [HookName] Failed"
- **AND** the notification body SHALL indicate the failure reason or context

#### Scenario: Stop hook with validation details

- **WHEN** the Stop hook runs validation commands
- **THEN** the notification SHALL indicate the validation status (e.g., "Tests passed", "Build failed")

### Requirement: Graceful Notification Failure Handling

The system SHALL handle notification failures gracefully without impacting hook execution.

#### Scenario: Notification library unavailable

- **WHEN** system notifications are not supported on the platform
- **THEN** the system SHALL log a warning message
- **AND** SHALL continue hook execution normally
- **AND** SHALL NOT return an error status

#### Scenario: Notification send failure

- **WHEN** sending a notification fails for any reason
- **THEN** the system SHALL log the error
- **AND** SHALL continue hook execution normally
- **AND** SHALL NOT block the hook from completing

### Requirement: Configuration Schema

The system SHALL include notification configuration in the YAML schema and default configuration.

#### Scenario: Default configuration file includes notifications section

- **WHEN** generating a default configuration with `conclaude init`
- **THEN** the configuration SHALL include a `notifications` section
- **AND** SHALL set `enabled` to `false` by default
- **AND** SHALL include commented examples showing available options

#### Scenario: Configuration validation

- **WHEN** loading configuration with notifications section
- **THEN** the system SHALL validate that `enabled` is a boolean
- **AND** SHALL validate that `hooks` is an array of strings (if provided)
- **AND** SHALL reject invalid hook names with a helpful error message

### Requirement: Notification Timing

The system SHALL send notifications at appropriate times during hook and command execution.

#### Scenario: Hook-level and per-command notifications coexist

- **WHEN** a hook executes with some commands having `notifyPerCommand: true`
- **THEN** the system SHALL send hook start notification (if configured)
- **AND** the system SHALL send per-command notifications for enabled commands
- **AND** the system SHALL send hook completion notification (if configured)
- **AND** notifications SHALL be sent in execution order without batching

#### Scenario: Multiple commands with notifyPerCommand enabled

- **WHEN** multiple commands in a hook have `notifyPerCommand: true`
- **THEN** each command SHALL generate its own start and completion notifications
- **AND** notifications SHALL be sent in the order commands execute
- **AND** the system SHALL NOT batch or suppress per-command notifications

### Requirement: Per-Command Notification Configuration

The system SHALL provide an optional `notifyPerCommand` field for individual stop and subagent stop commands to enable notifications for each command execution.

#### Scenario: Command with notifyPerCommand enabled

- **WHEN** a stop command includes `notifyPerCommand: true`
- **AND** notifications are enabled for the Stop hook
- **THEN** the system SHALL send a notification when the command starts execution
- **AND** the system SHALL send a notification when the command completes (success or failure)

#### Scenario: Command with notifyPerCommand disabled

- **WHEN** a stop command includes `notifyPerCommand: false` or omits the field
- **THEN** the system SHALL NOT send per-command notifications
- **AND** only hook-level notifications SHALL be sent

#### Scenario: Per-command notifications respect hook filters

- **WHEN** a command has `notifyPerCommand: true`
- **AND** the Stop hook is not in the `notifications.hooks` array
- **THEN** no per-command notifications SHALL be sent
- **AND** the hook filter takes precedence over per-command settings

### Requirement: Per-Command Notification Content

The system SHALL include command-specific context in per-command notifications when enabled.

#### Scenario: Per-command notification with showCommand enabled

- **WHEN** a command has `notifyPerCommand: true` and `showCommand: true`
- **THEN** the start notification SHALL include the command being executed (e.g., "Running: npm test")
- **AND** the completion notification SHALL include the command name (e.g., "Completed: npm test" or "Failed: npm test")

#### Scenario: Per-command notification with showCommand disabled

- **WHEN** a command has `notifyPerCommand: true` and `showCommand: false`
- **THEN** the start notification SHALL use a generic message (e.g., "Running command 1/3")
- **AND** the completion notification SHALL use a generic message (e.g., "Command 1/3 completed")

#### Scenario: Per-command failure notification includes context

- **WHEN** a command with `notifyPerCommand: true` fails
- **AND** `notifications.showErrors` is true
- **THEN** the notification SHALL indicate failure status
- **AND** the notification MAY include the exit code or error summary

### Requirement: Per-Command Notification Filtering

The system SHALL apply all existing notification filters to per-command notifications.

#### Scenario: Per-command notifications respect showSuccess filter

- **WHEN** a command with `notifyPerCommand: true` completes successfully
- **AND** `notifications.showSuccess` is false
- **THEN** no success notification SHALL be sent for that command
- **AND** only failure notifications (if any) SHALL be sent

#### Scenario: Per-command notifications respect showErrors filter

- **WHEN** a command with `notifyPerCommand: true` fails
- **AND** `notifications.showErrors` is false
- **THEN** no failure notification SHALL be sent for that command
- **AND** only success notifications (if any) SHALL be sent

#### Scenario: Per-command notifications respect showSystemEvents filter

- **WHEN** a command with `notifyPerCommand: true` starts or completes
- **THEN** `notifications.showSystemEvents` SHALL NOT affect per-command notifications
- **AND** per-command notifications SHALL be controlled by `showSuccess` and `showErrors` flags only

### Requirement: Per-Command Notification Validation

The system SHALL validate `notifyPerCommand` values in configuration to ensure proper formatting.

#### Scenario: Valid notifyPerCommand value

- **WHEN** `notifyPerCommand` field contains a boolean value (`true` or `false`)
- **THEN** the configuration SHALL be accepted
- **AND** the notification behavior SHALL be applied during command execution

#### Scenario: Invalid notifyPerCommand value

- **WHEN** `notifyPerCommand` field contains a non-boolean value
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the `notifyPerCommand` value format issue

#### Scenario: Missing notifyPerCommand field

- **WHEN** a command configuration omits the `notifyPerCommand` field
- **THEN** the system SHALL default to `false`
- **AND** no per-command notifications SHALL be sent
