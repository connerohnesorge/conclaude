## ADDED Requirements

### Requirement: Ripgrep Library Integration

The system SHALL use ripgrep's library crates (grep-searcher, grep-regex, ignore) for pattern searching instead of spawning an external binary.

#### Scenario: No external binary dependency

- **WHEN** an rg command is executed
- **THEN** the system SHALL use compiled-in ripgrep libraries
- **AND** no external `rg` binary SHALL be required
- **AND** all search functionality SHALL work without PATH dependencies

#### Scenario: Library configuration mapping

- **WHEN** an rg command is executed with configuration options
- **THEN** `RegexMatcherBuilder` SHALL be configured with regex options
- **AND** `WalkBuilder` SHALL be configured with file walking options
- **AND** `SearcherBuilder` SHALL be configured with search options

### Requirement: Pattern Matching Execution

The system SHALL execute ripgrep-based pattern searches using the grep-regex crate.

#### Scenario: Basic pattern execution

- **WHEN** an rg command specifies `pattern: "TODO"` and `files: "**/*.rs"`
- **THEN** the system SHALL search all matching Rust files for "TODO"
- **AND** matching lines SHALL be collected for counting and output

#### Scenario: Regex option application

- **WHEN** `ignoreCase: true` is configured
- **THEN** `RegexMatcherBuilder::case_insensitive(true)` SHALL be called
- **AND** pattern "todo" SHALL match "TODO", "Todo", etc.

#### Scenario: Word boundary matching

- **WHEN** `word: true` is configured
- **THEN** `RegexMatcherBuilder::word(true)` SHALL be called
- **AND** pattern "map" SHALL NOT match "hashmap" or "mapping"

#### Scenario: Fixed strings mode

- **WHEN** `fixedStrings: true` is configured
- **THEN** `RegexMatcherBuilder::fixed_strings(true)` SHALL be called
- **AND** pattern "a.*b" SHALL match literal "a.*b", not regex

### Requirement: File Walking Execution

The system SHALL traverse files using the ignore crate's WalkBuilder.

#### Scenario: Glob pattern matching

- **WHEN** `files: "**/*.rs"` is configured
- **THEN** only files matching the glob SHALL be searched
- **AND** directories SHALL be traversed recursively

#### Scenario: Gitignore respect (default)

- **WHEN** `gitIgnore` is not specified or `gitIgnore: true`
- **THEN** files matching .gitignore patterns SHALL be skipped
- **AND** .git directories SHALL be skipped

#### Scenario: Hidden files exclusion (default)

- **WHEN** `hidden` is not specified or `hidden: false`
- **THEN** hidden files (starting with .) SHALL be skipped
- **AND** hidden directories SHALL not be traversed

#### Scenario: File type filtering

- **WHEN** `types: ["rust", "js"]` is configured
- **THEN** only files with extensions matching those types SHALL be searched
- **AND** type definitions SHALL use ripgrep's built-in type database

#### Scenario: Empty glob error

- **WHEN** the `files` glob matches zero files
- **THEN** command execution SHALL fail with an error
- **AND** the error message SHALL indicate no files matched the glob pattern
- **AND** this SHALL be treated as a configuration error, not a constraint pass

### Requirement: Match Counting

The system SHALL count matches according to the configured count mode.

#### Scenario: Lines count mode (default)

- **WHEN** `countMode: lines` is configured or not specified
- **THEN** each line containing matches SHALL count as 1
- **AND** multiple matches on the same line SHALL count as 1 total

#### Scenario: Occurrences count mode

- **WHEN** `countMode: occurrences` is configured
- **THEN** every match instance SHALL be counted
- **AND** a line with "TODO fix TODO" matching "TODO" SHALL count as 2

#### Scenario: Cross-file counting

- **WHEN** pattern matches are found in multiple files
- **THEN** the total count SHALL be the sum across all files
- **AND** per-file counts SHALL be available for output

### Requirement: Match Count Constraints

The system SHALL evaluate match counts against configured constraints.

#### Scenario: max constraint validation

- **WHEN** an rg command has `max: 5` configured
- **AND** the search finds 3 matches
- **THEN** the command SHALL pass (3 <= 5)

#### Scenario: max constraint violation

- **WHEN** an rg command has `max: 5` configured
- **AND** the search finds 7 matches
- **THEN** the command SHALL fail
- **AND** the error message SHALL be "Found 7 matches, maximum allowed is 5"

#### Scenario: min constraint validation

- **WHEN** an rg command has `min: 1` configured
- **AND** the search finds 3 matches
- **THEN** the command SHALL pass (3 >= 1)

#### Scenario: min constraint violation

- **WHEN** an rg command has `min: 1` configured
- **AND** the search finds 0 matches
- **THEN** the command SHALL fail
- **AND** the error message SHALL be "Found 0 matches, minimum required is 1"

#### Scenario: equal constraint validation

- **WHEN** an rg command has `equal: 1` configured
- **AND** the search finds exactly 1 match
- **THEN** the command SHALL pass

#### Scenario: equal constraint violation

- **WHEN** an rg command has `equal: 1` configured
- **AND** the search finds 3 matches
- **THEN** the command SHALL fail
- **AND** the error message SHALL be "Found 3 matches, expected exactly 1"

#### Scenario: Default constraint (max: 0)

- **WHEN** no constraint is specified
- **THEN** `max: 0` SHALL be applied implicitly
- **AND** any match SHALL cause the command to fail

### Requirement: Command Action Behavior

The system SHALL handle command failures according to the configured action.

#### Scenario: Block action (default)

- **WHEN** a constraint is violated with `action: block` or no action specified
- **THEN** the hook SHALL return a blocked result
- **AND** Claude SHALL receive the error message
- **AND** subsequent commands SHALL NOT execute

#### Scenario: Warn action

- **WHEN** a constraint is violated with `action: warn`
- **THEN** a warning SHALL be logged to stderr
- **AND** the hook SHALL continue without blocking
- **AND** subsequent commands SHALL execute normally

### Requirement: Binary File Handling

The system SHALL skip binary files by default.

#### Scenario: Binary file detection

- **WHEN** a file contains NUL bytes (binary indicator)
- **THEN** the file SHALL be skipped
- **AND** no matches from that file SHALL be counted
- **AND** no error SHALL be reported

### Requirement: Context Line Collection

The system SHALL collect context lines when configured.

#### Scenario: Context lines enabled

- **WHEN** `context: 2` is configured
- **THEN** 2 lines before each match SHALL be collected
- **AND** 2 lines after each match SHALL be collected
- **AND** context lines SHALL be included in output when `showStdout: true`

#### Scenario: Context with no surrounding lines

- **WHEN** a match is on line 1 with `context: 2`
- **THEN** no before-context lines SHALL be shown (none exist)
- **AND** 2 after-context lines SHALL be shown if they exist

### Requirement: Output Integration

The system SHALL integrate rg command output with existing output limiting and display configuration.

#### Scenario: showStdout with rg command

- **WHEN** an rg command has `showStdout: true`
- **THEN** matching lines (and context if configured) SHALL be included in the hook result
- **AND** output SHALL include file paths and line numbers
- **AND** `maxOutputLines` SHALL apply to limit displayed matches

#### Scenario: showStderr with rg command

- **WHEN** an rg command has `showStderr: true`
- **AND** errors occur during search (e.g., permission denied)
- **THEN** stderr SHALL be included in the hook result

#### Scenario: maxOutputLines applies to rg output

- **WHEN** an rg command has `maxOutputLines: 10`
- **AND** the search finds 50 matching lines
- **THEN** only the first 10 lines SHALL be displayed
- **AND** a truncation indicator SHALL show "(40 lines omitted)"

### Requirement: Timeout Handling

The system SHALL enforce timeouts on rg command execution.

#### Scenario: Timeout specified

- **WHEN** `timeout: 30` is configured for an rg command
- **THEN** the search SHALL be cancelled after 30 seconds
- **AND** the command SHALL fail with a timeout error

#### Scenario: Search completes within timeout

- **WHEN** the search completes before the timeout
- **THEN** results SHALL be processed normally
- **AND** no timeout error SHALL occur
