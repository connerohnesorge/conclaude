# Implementation Tasks: Add Per-Command Notifications

## Configuration Schema (5 tasks)

- [ ] Add `notifyPerCommand` optional boolean field to `StopCommand` struct in `src/config.rs`
- [ ] Add `notifyPerCommand` optional boolean field to `SubagentStopCommand` struct in `src/config.rs`
- [ ] Update JSON schema (`conclaude-schema.json`) to include `notifyPerCommand` field for stop commands
- [ ] Update JSON schema to include `notifyPerCommand` field for subagent stop commands
- [ ] Add configuration validation test for valid and invalid `notifyPerCommand` values

## Notification Logic (4 tasks)

- [ ] Add per-command notification support to `execute_stop_commands()` in `src/hooks.rs`
- [ ] Add per-command notification support to `execute_subagent_stop_commands()` in `src/hooks.rs`
- [ ] Ensure per-command notifications respect `showCommand` flag (show command name vs generic message)
- [ ] Ensure per-command notifications respect existing notification filters (`hooks`, `showErrors`, `showSuccess`)

## Command Execution (3 tasks)

- [ ] Pass `notifyPerCommand` flag from `StopCommandConfig` through execution pipeline
- [ ] Pass `notifyPerCommand` flag from `SubagentStopCommandConfig` through execution pipeline
- [ ] Send start notification before command execution (if `notifyPerCommand: true` and notifications enabled)
- [ ] Send completion notification after command execution (if `notifyPerCommand: true` and notifications enabled)

## Testing (5 tasks)

- [ ] Add unit test for per-command notification with `showCommand: true` (shows command name)
- [ ] Add unit test for per-command notification with `showCommand: false` (generic message)
- [ ] Add unit test verifying notifications respect `showErrors` and `showSuccess` filters
- [ ] Add integration test for stop hook with mixed `notifyPerCommand` settings
- [ ] Add integration test for subagent stop hook with `notifyPerCommand` enabled

## Documentation (3 tasks)

- [ ] Update configuration documentation for `Stop` commands to include `notifyPerCommand`
- [ ] Update configuration documentation for `SubagentStop` commands to include `notifyPerCommand`
- [ ] Add example configuration showing per-command notifications in use

## Total: 20 tasks

### Parallelizable Work
- Configuration schema tasks (1-5) can be done in parallel with notification logic tasks (6-9)
- Testing tasks (14-18) can be done in parallel after implementation is complete
- Documentation tasks (19-21) can be done in parallel with testing

### Dependencies
- Command execution tasks (10-13) depend on configuration schema (1-5) and notification logic (6-9)
- Testing tasks (14-18) depend on all implementation tasks (1-13)
- Documentation tasks (19-21) have no strict dependencies
