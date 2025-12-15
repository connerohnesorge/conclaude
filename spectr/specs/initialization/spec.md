# Initialization Specification

## Requirements

### Requirement: Generated Hooks Include Timeout Configuration

The `conclaude init` command SHALL generate Claude Code hook configurations that include a timeout field to prevent indefinite hook execution.

#### Scenario: Default timeout for generated hooks

- **WHEN** a user runs `conclaude init` to initialize a project
- **THEN** the generated `.claude/settings.json` SHALL include hook configurations with a `timeout` field
- **AND** the timeout value SHALL be 600 seconds (10 minutes)
- **AND** all hook types (PreToolUse, PostToolUse, Stop, etc.) SHALL have the same timeout value

#### Scenario: Settings.json hook structure with timeout

- **WHEN** the `.claude/settings.json` file is generated or updated
- **THEN** each hook configuration SHALL follow the Claude Code hooks schema:
  ```json
  {
    "type": "command",
    "command": "conclaude Hooks <HookType>",
    "timeout": 600
  }
  ```
- **AND** the timeout field SHALL be a positive integer representing seconds

#### Scenario: Existing settings preserved with timeout added

- **WHEN** a user runs `conclaude init` on a project with existing `.claude/settings.json`
- **AND** the `--force` flag is not specified
- **THEN** existing hooks SHALL be updated to include the timeout field
- **AND** other existing settings (permissions, includeCoAuthoredBy, etc.) SHALL be preserved

