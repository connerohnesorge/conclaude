# Proposal: Add Per-Command Notifications

## Problem Statement

Currently, conclaude sends one notification per hook execution regardless of how many commands are configured. For example, if a `Stop` hook has 3 commands (`npm test`, `npm run build`, `git status`), users receive only a single notification when all commands complete or when one fails.

This creates two issues:
1. **Lack of real-time feedback**: Users don't know which specific command is running or has completed
2. **Poor failure context**: When a command fails, the notification doesn't indicate which command in the sequence caused the failure

Users who configure multiple commands in their hooks would benefit from per-command notifications to monitor progress and quickly identify failures.

## Proposed Solution

Add an optional `notifyPerCommand` boolean field to command configurations that enables sending individual notifications for each command execution. This provides granular feedback without changing the default behavior.

### User-facing behavior

When `notifyPerCommand: true` is configured on a command:
- A notification is sent when the command **starts** execution (optional, controlled by notification config)
- A notification is sent when the command **completes** (success or failure)
- The notification includes the command name (if `showCommand: true`) or a generic message
- All existing notification filters (`hooks`, `showErrors`, `showSuccess`) continue to apply

### Configuration example

```yaml
stop:
  commands:
    - run: npm test
      notifyPerCommand: true  # New field
      showCommand: true
    - run: npm run build
      notifyPerCommand: true
    - run: git status
      # notifyPerCommand defaults to false (existing behavior)

notifications:
  enabled: true
  hooks: ["Stop"]
  showErrors: true
  showSuccess: true
```

With this configuration:
- Hook-level notification: "Stop hook started" (existing)
- Command 1 notification: "Running: npm test" (new)
- Command 1 notification: "Command completed: npm test" (new)
- Command 2 notification: "Running: npm run build" (new)
- Command 2 notification: "Command completed: npm run build" (new)
- Command 3: No per-command notifications (notifyPerCommand: false)
- Hook-level notification: "Stop hook completed" (existing)

## Impact Analysis

### Breaking Changes
None. This is a purely additive feature with default behavior unchanged.

### Configuration Changes
- Add optional `notifyPerCommand: bool` field to `StopCommand` struct
- Add optional `notifyPerCommand: bool` field to `SubagentStopCommand` struct
- Update JSON schema to include new field

### Performance Impact
Minimal. Notifications are non-blocking and failures are gracefully handled.

### User Experience
- **Improved**: Users who want granular feedback can opt-in
- **Unchanged**: Users who don't configure the field see existing behavior

## Alternatives Considered

### Alternative 1: Always send per-command notifications
**Rejected**: Would spam users who have many commands configured and don't need that level of detail.

### Alternative 2: Add a global `notifyPerCommand` setting
**Rejected**: Less flexible than per-command control. Some commands may warrant notifications while others don't.

### Alternative 3: Only send start OR complete notifications
**Rejected**: Both are valuable - start notifications show progress, complete notifications confirm success/failure.

## Success Criteria

1. Users can enable per-command notifications on individual commands
2. Notifications include command context (name if shown, generic message otherwise)
3. Per-command notifications respect all existing notification filters
4. Default behavior (no per-command notifications) is preserved
5. Validation passes for both valid and invalid configurations

## Out of Scope

- Command progress bars or percentage indicators
- Notification grouping or batching
- Custom notification templates per command
- Start notifications controlled separately from completion notifications
