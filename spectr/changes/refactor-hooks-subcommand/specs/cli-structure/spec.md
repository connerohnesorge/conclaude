## ADDED Requirements

### Requirement: Hooks Parent Subcommand
The CLI SHALL provide a `hooks` parent subcommand that contains all hook-handling commands as nested subcommands.

#### Scenario: Invoking hook via nested subcommand
- **WHEN** a user invokes `conclaude hooks PreToolUse`
- **THEN** the PreToolUse hook handler SHALL execute
- **AND** the hook SHALL read JSON payload from stdin
- **AND** the hook SHALL output JSON result to stdout

#### Scenario: Listing hooks subcommands
- **WHEN** a user invokes `conclaude hooks --help`
- **THEN** all 11 hook subcommands SHALL be listed (PreToolUse, PostToolUse, PermissionRequest, Notification, UserPromptSubmit, SessionStart, SessionEnd, Stop, SubagentStart, SubagentStop, PreCompact)
- **AND** each subcommand SHALL have a brief description

#### Scenario: Top-level help excludes individual hooks
- **WHEN** a user invokes `conclaude --help`
- **THEN** individual hook commands SHALL NOT appear at top level
- **AND** only `Init`, `Validate`, `Visualize`, and `hooks` SHALL appear as top-level subcommands

### Requirement: Hook Command Path in Init
The `Init` command SHALL generate Claude Code hook configurations using the nested `hooks` subcommand path.

#### Scenario: Init generates nested hook commands
- **WHEN** a user runs `conclaude Init`
- **THEN** the generated `.claude/settings.json` SHALL contain hook commands in the format `conclaude hooks <HookName>`
- **AND** all 11 hook types SHALL use the nested command format

#### Scenario: Init output shows correct command paths
- **WHEN** a user runs `conclaude Init`
- **THEN** the console output listing configured hooks SHALL show the nested command format (e.g., `conclaude hooks PreToolUse`)

### Requirement: All Hooks Under Parent Command
All 11 hook commands SHALL be accessible only via the `hooks` parent subcommand.

#### Scenario: PreToolUse accessible via hooks
- **WHEN** `conclaude hooks PreToolUse` is invoked with valid JSON payload
- **THEN** the PreToolUse handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: PostToolUse accessible via hooks
- **WHEN** `conclaude hooks PostToolUse` is invoked with valid JSON payload
- **THEN** the PostToolUse handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: PermissionRequest accessible via hooks
- **WHEN** `conclaude hooks PermissionRequest` is invoked with valid JSON payload
- **THEN** the PermissionRequest handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: Notification accessible via hooks
- **WHEN** `conclaude hooks Notification` is invoked with valid JSON payload
- **THEN** the Notification handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: UserPromptSubmit accessible via hooks
- **WHEN** `conclaude hooks UserPromptSubmit` is invoked with valid JSON payload
- **THEN** the UserPromptSubmit handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SessionStart accessible via hooks
- **WHEN** `conclaude hooks SessionStart` is invoked with valid JSON payload
- **THEN** the SessionStart handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SessionEnd accessible via hooks
- **WHEN** `conclaude hooks SessionEnd` is invoked with valid JSON payload
- **THEN** the SessionEnd handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: Stop accessible via hooks
- **WHEN** `conclaude hooks Stop` is invoked with valid JSON payload
- **THEN** the Stop handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SubagentStart accessible via hooks
- **WHEN** `conclaude hooks SubagentStart` is invoked with valid JSON payload
- **THEN** the SubagentStart handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SubagentStop accessible via hooks
- **WHEN** `conclaude hooks SubagentStop` is invoked with valid JSON payload
- **THEN** the SubagentStop handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: PreCompact accessible via hooks
- **WHEN** `conclaude hooks PreCompact` is invoked with valid JSON payload
- **THEN** the PreCompact handler SHALL process the payload
- **AND** return appropriate JSON response
