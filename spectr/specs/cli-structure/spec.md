# Cli Structure Specification

## Purpose

Define the CLI command structure with a Hooks parent subcommand that organizes all 11 hook handlers under a single nested command hierarchy, replacing the previous top-level hook commands.

## Requirements

### Requirement: Hooks Parent Subcommand
The CLI SHALL provide a `Hooks` parent subcommand (PascalCase) that contains all hook-handling commands as nested subcommands.

#### Scenario: Invoking hook via nested subcommand
- **WHEN** a user invokes `conclaude Hooks PreToolUse`
- **THEN** the PreToolUse hook handler SHALL execute
- **AND** the hook SHALL read JSON payload from stdin
- **AND** the hook SHALL output JSON result to stdout

#### Scenario: Listing Hooks subcommands
- **WHEN** a user invokes `conclaude Hooks --help`
- **THEN** all 11 hook subcommands SHALL be listed (PreToolUse, PostToolUse, PermissionRequest, Notification, UserPromptSubmit, SessionStart, SessionEnd, Stop, SubagentStart, SubagentStop, PreCompact)
- **AND** each subcommand SHALL have a brief description

#### Scenario: Top-level help excludes individual hooks
- **WHEN** a user invokes `conclaude --help`
- **THEN** individual hook commands SHALL NOT appear at top level
- **AND** only `Init`, `Validate`, `Visualize`, and `Hooks` SHALL appear as top-level subcommands

### Requirement: Hook Command Path in Init
The `Init` command SHALL generate Claude Code hook configurations using the nested `Hooks` subcommand path and SHALL overwrite any existing hook configurations.

#### Scenario: Init generates nested hook commands
- **WHEN** a user runs `conclaude Init`
- **THEN** the generated `.claude/settings.json` SHALL contain hook commands in the format `conclaude Hooks <HookName>`
- **AND** all 11 hook types SHALL use the nested command format

#### Scenario: Init overwrites existing hooks
- **WHEN** a user runs `conclaude Init` on a project with existing old-format hooks
- **THEN** all existing hook entries SHALL be replaced with new-format commands
- **AND** no old-format commands SHALL remain in the configuration

#### Scenario: Init output shows correct command paths
- **WHEN** a user runs `conclaude Init`
- **THEN** the console output listing configured hooks SHALL show the nested command format (e.g., `conclaude Hooks PreToolUse`)

### Requirement: All Hooks Under Parent Command
All 11 hook commands SHALL be accessible only via the `Hooks` parent subcommand. Old top-level hook commands SHALL NOT be available (clean break).

#### Scenario: PreToolUse accessible via Hooks
- **WHEN** `conclaude Hooks PreToolUse` is invoked with valid JSON payload
- **THEN** the PreToolUse handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: PostToolUse accessible via Hooks
- **WHEN** `conclaude Hooks PostToolUse` is invoked with valid JSON payload
- **THEN** the PostToolUse handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: PermissionRequest accessible via Hooks
- **WHEN** `conclaude Hooks PermissionRequest` is invoked with valid JSON payload
- **THEN** the PermissionRequest handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: Notification accessible via Hooks
- **WHEN** `conclaude Hooks Notification` is invoked with valid JSON payload
- **THEN** the Notification handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: UserPromptSubmit accessible via Hooks
- **WHEN** `conclaude Hooks UserPromptSubmit` is invoked with valid JSON payload
- **THEN** the UserPromptSubmit handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SessionStart accessible via Hooks
- **WHEN** `conclaude Hooks SessionStart` is invoked with valid JSON payload
- **THEN** the SessionStart handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SessionEnd accessible via Hooks
- **WHEN** `conclaude Hooks SessionEnd` is invoked with valid JSON payload
- **THEN** the SessionEnd handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: Stop accessible via Hooks
- **WHEN** `conclaude Hooks Stop` is invoked with valid JSON payload
- **THEN** the Stop handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SubagentStart accessible via Hooks
- **WHEN** `conclaude Hooks SubagentStart` is invoked with valid JSON payload
- **THEN** the SubagentStart handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: SubagentStop accessible via Hooks
- **WHEN** `conclaude Hooks SubagentStop` is invoked with valid JSON payload
- **THEN** the SubagentStop handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: PreCompact accessible via Hooks
- **WHEN** `conclaude Hooks PreCompact` is invoked with valid JSON payload
- **THEN** the PreCompact handler SHALL process the payload
- **AND** return appropriate JSON response

#### Scenario: Old top-level hook command fails
- **WHEN** a user invokes `conclaude SubagentStop` (old format)
- **THEN** the CLI SHALL return an error indicating unknown command
- **AND** no hook processing SHALL occur

