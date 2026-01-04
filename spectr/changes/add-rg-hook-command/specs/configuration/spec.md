## ADDED Requirements

### Requirement: RgConfig Structure

The system SHALL define an `RgConfig` struct for ripgrep-based pattern matching configuration with comprehensive options.

#### Scenario: RgConfig required fields

- **WHEN** parsing an rg command configuration
- **THEN** `pattern` field SHALL be required (regex pattern string)
- **AND** `files` field SHALL be required (glob pattern for file matching)

#### Scenario: RgConfig regex options

- **WHEN** an rg configuration includes regex options
- **THEN** the following fields SHALL be supported:
  - `ignoreCase` (bool, default: false) - case insensitive matching
  - `smartCase` (bool, default: false) - auto-detect from pattern
  - `word` (bool, default: false) - word boundary matching
  - `fixedStrings` (bool, default: false) - literal pattern
  - `multiLine` (bool, default: false) - ^ and $ match line boundaries
  - `wholeLine` (bool, default: false) - match entire line
  - `dotMatchesNewLine` (bool, default: false) - . matches newlines
  - `unicode` (bool, default: true) - Unicode character classes

#### Scenario: RgConfig file walking options

- **WHEN** an rg configuration includes file walking options
- **THEN** the following fields SHALL be supported:
  - `maxDepth` (Option<usize>, default: none) - max directory depth
  - `hidden` (bool, default: false) - include hidden files
  - `followLinks` (bool, default: false) - follow symlinks
  - `maxFilesize` (Option<u64>, default: none) - skip large files
  - `gitIgnore` (bool, default: true) - respect .gitignore
  - `ignore` (bool, default: true) - respect .ignore files
  - `parents` (bool, default: true) - read parent ignore files
  - `sameFileSystem` (bool, default: false) - don't cross filesystems
  - `threads` (Option<usize>, default: auto) - parallel threads
  - `types` (Vec<String>, default: empty) - file type filters

#### Scenario: RgConfig search options

- **WHEN** an rg configuration includes search options
- **THEN** the following fields SHALL be supported:
  - `context` (usize, default: 0) - lines before and after matches
  - `countMode` (enum, default: "lines") - "lines" or "occurrences"
  - `invertMatch` (bool, default: false) - show non-matching lines

#### Scenario: RgConfig constraint fields

- **WHEN** an rg configuration includes constraint fields
- **THEN** `max`, `min`, and `equal` SHALL be mutually exclusive
- **AND** each SHALL accept a non-negative integer
- **AND** if none specified, `max: 0` SHALL be implicit

### Requirement: CountMode Enumeration

The system SHALL define a `CountMode` enum for configuring how matches are counted.

#### Scenario: Lines count mode

- **WHEN** `countMode: lines` is configured
- **THEN** each line containing one or more matches SHALL count as 1
- **AND** a line with pattern "TODO TODO" SHALL count as 1 match

#### Scenario: Occurrences count mode

- **WHEN** `countMode: occurrences` is configured
- **THEN** every match instance SHALL be counted separately
- **AND** a line with pattern "TODO TODO" matching "TODO" SHALL count as 2 matches

### Requirement: CommandAction Enumeration

The system SHALL define a `CommandAction` enum for configuring command failure behavior.

#### Scenario: Block action (default)

- **WHEN** `action: block` is configured or action is not specified
- **THEN** constraint violations SHALL block the hook
- **AND** the hook result SHALL contain the error message

#### Scenario: Warn action

- **WHEN** `action: warn` is configured
- **THEN** constraint violations SHALL log a warning
- **AND** the hook SHALL continue without blocking
- **AND** subsequent commands SHALL still execute

### Requirement: Mutual Exclusivity of run and rg

The system SHALL enforce that each command entry specifies exactly one of `run` or `rg`, not both.

#### Scenario: Command with only run field

- **WHEN** a stop command has `run: "npm test"` and no `rg` field
- **THEN** the configuration SHALL be accepted
- **AND** the command SHALL execute as a shell command

#### Scenario: Command with only rg field

- **WHEN** a stop command has an `rg` configuration and no `run` field
- **THEN** the configuration SHALL be accepted
- **AND** the command SHALL execute as a ripgrep search

#### Scenario: Command with both run and rg fields

- **WHEN** a stop command has both `run` and `rg` fields
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate that `run` and `rg` are mutually exclusive

#### Scenario: Command with neither run nor rg field

- **WHEN** a stop command has neither `run` nor `rg` field
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate that either `run` or `rg` is required

### Requirement: Mutual Exclusivity of Constraints

The system SHALL enforce that at most one constraint field (max, min, equal) is specified per rg command.

#### Scenario: Single constraint accepted

- **WHEN** an rg configuration includes exactly one of `max`, `min`, or `equal`
- **THEN** the configuration SHALL be accepted

#### Scenario: Multiple constraints rejected

- **WHEN** an rg configuration includes more than one of `max`, `min`, `equal`
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate that only one constraint is allowed

#### Scenario: No constraint defaults to max zero

- **WHEN** an rg configuration has no constraint fields
- **THEN** the configuration SHALL be accepted
- **AND** an implicit `max: 0` constraint SHALL be applied at execution time

### Requirement: File Type Validation

The system SHALL validate that file type names in the `types` array are recognized.

#### Scenario: Valid file type names

- **WHEN** `types: ["rust", "js", "py"]` is configured
- **THEN** the configuration SHALL be accepted
- **AND** only files matching those types SHALL be searched

#### Scenario: Unknown file type name

- **WHEN** `types: ["unknowntype"]` is configured
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the unrecognized type name
- **AND** the error message SHALL suggest similar valid type names if available

### Requirement: Pattern Validation

The system SHALL validate regex patterns at configuration load time.

#### Scenario: Valid regex pattern

- **WHEN** `pattern: "TODO|FIXME"` is configured
- **THEN** the configuration SHALL be accepted

#### Scenario: Invalid regex pattern

- **WHEN** `pattern: "unclosed(group"` is configured
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL include the regex compilation error

#### Scenario: Literal pattern with fixedStrings

- **WHEN** `pattern: "literal.*text"` with `fixedStrings: true` is configured
- **THEN** the configuration SHALL be accepted
- **AND** the pattern SHALL be treated as literal (not regex)

### Requirement: SubagentStopCommand rg Support

The system SHALL support the `rg` field in subagent stop commands with identical structure to regular stop commands.

#### Scenario: SubagentStop with rg command

- **WHEN** a subagentStop command has an `rg` configuration
- **THEN** the configuration SHALL be accepted
- **AND** all rg options SHALL work identically to regular stop commands

#### Scenario: SubagentStop mutual exclusivity

- **WHEN** a subagentStop command has both `run` and `rg` fields
- **THEN** configuration loading SHALL fail with the same validation error as regular stop commands
