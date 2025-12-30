## ADDED Requirements

### Requirement: Tree-sitter Library Integration

The system SHALL use the tree-sitter crate with bundled language grammars for structural code analysis.

#### Scenario: No external grammar dependency

- **WHEN** a ts command is executed
- **THEN** the system SHALL use compiled-in tree-sitter grammars
- **AND** no external grammar files SHALL be required
- **AND** all supported languages SHALL work without additional installation

#### Scenario: Parser and query setup

- **WHEN** a ts command is executed
- **THEN** a tree-sitter Parser SHALL be created
- **AND** the appropriate Language SHALL be loaded based on file extension
- **AND** the Query SHALL be compiled from the S-expression string

### Requirement: Language Detection

The system SHALL auto-detect file languages from extensions.

#### Scenario: Rust file detection

- **WHEN** a file with extension `.rs` is encountered
- **THEN** the Rust grammar SHALL be used for parsing

#### Scenario: JavaScript/TypeScript detection

- **WHEN** a file with extension `.js`, `.mjs`, `.cjs` is encountered
- **THEN** the JavaScript grammar SHALL be used
- **AND** files with `.ts`, `.tsx` SHALL use the TypeScript grammar

#### Scenario: Language override

- **WHEN** `language: "python"` is configured
- **AND** a file with extension `.txt` is encountered
- **THEN** the Python grammar SHALL be used regardless of extension

#### Scenario: Unknown extension

- **WHEN** a file with an unknown extension is encountered
- **AND** no `language` override is specified
- **THEN** the file SHALL be skipped with a warning
- **AND** no error SHALL be raised

### Requirement: Query Execution

The system SHALL execute tree-sitter queries against parsed files.

#### Scenario: Basic query execution

- **WHEN** a ts command specifies `query: "(function_item) @fn"` and `files: "**/*.rs"`
- **THEN** the system SHALL parse all matching Rust files
- **AND** execute the query against each file's syntax tree
- **AND** collect all captures matching `@fn`

#### Scenario: Predicate filtering

- **WHEN** a query includes `(#eq? @name "main")`
- **THEN** only captures where the node text equals "main" SHALL be counted
- **AND** non-matching captures SHALL be excluded

#### Scenario: Regex predicate

- **WHEN** a query includes `(#match? @comment "TODO|FIXME")`
- **THEN** only captures where the node text matches the regex SHALL be counted

#### Scenario: Multiple captures

- **WHEN** a query defines multiple captures `@fn`, `@name`, `@body`
- **AND** `capture: "@fn"` is specified
- **THEN** only nodes captured as `@fn` SHALL be counted

### Requirement: File Walking Integration

The system SHALL traverse files using the ignore crate, sharing options with the `rg` command.

#### Scenario: Glob pattern matching

- **WHEN** `files: "src/**/*.rs"` is configured
- **THEN** only files matching the glob SHALL be parsed
- **AND** directories SHALL be traversed recursively

#### Scenario: Shared walk options

- **WHEN** `hidden: true`, `gitIgnore: false` are configured
- **THEN** file walking SHALL behave identically to the `rg` command with same options

#### Scenario: Empty glob error

- **WHEN** the `files` glob matches zero files
- **THEN** command execution SHALL fail with an error
- **AND** the error message SHALL indicate no files matched the glob pattern

### Requirement: Capture Counting

The system SHALL count captures for constraint evaluation.

#### Scenario: Count captures across files

- **WHEN** matches are found in multiple files
- **THEN** the total count SHALL be the sum across all files
- **AND** per-file counts SHALL be available for output

#### Scenario: Count specified capture only

- **WHEN** `capture: "@unsafe"` is specified
- **AND** the query has multiple captures
- **THEN** only nodes captured as `@unsafe` SHALL contribute to the count

#### Scenario: Default capture counting

- **WHEN** no `capture` field is specified
- **THEN** the first capture defined in the query SHALL be counted

### Requirement: Match Count Constraints

The system SHALL evaluate capture counts against configured constraints.

#### Scenario: max constraint validation

- **WHEN** a ts command has `max: 0` configured
- **AND** the query finds 2 matches
- **THEN** the command SHALL fail
- **AND** the error message SHALL be "Found 2 captures of @name, maximum allowed is 0"

#### Scenario: min constraint validation

- **WHEN** a ts command has `min: 1` configured
- **AND** the query finds 0 matches
- **THEN** the command SHALL fail
- **AND** the error message SHALL be "Found 0 captures of @name, minimum required is 1"

#### Scenario: equal constraint validation

- **WHEN** a ts command has `equal: 1` configured
- **AND** the query finds 3 matches
- **THEN** the command SHALL fail
- **AND** the error message SHALL be "Found 3 captures of @name, expected exactly 1"

#### Scenario: Default constraint (max: 0)

- **WHEN** no constraint is specified
- **THEN** `max: 0` SHALL be applied implicitly
- **AND** any capture SHALL cause the command to fail

### Requirement: Output Formatting

The system SHALL format matched captures for display.

#### Scenario: Detailed output format

- **WHEN** `showStdout: true` is configured
- **THEN** each match SHALL be displayed as: `file:line:col [node_type]: captured_text`
- **AND** line and column SHALL be 1-based

#### Scenario: Long text truncation

- **WHEN** a captured node's text exceeds 100 characters
- **THEN** the text SHALL be truncated
- **AND** an ellipsis SHALL indicate truncation

#### Scenario: Multiline text

- **WHEN** a captured node spans multiple lines
- **THEN** only the first line SHALL be shown
- **AND** `...` SHALL indicate additional lines

#### Scenario: maxOutputLines limit

- **WHEN** `maxOutputLines: 5` is configured
- **AND** 20 matches are found
- **THEN** only the first 5 matches SHALL be displayed
- **AND** a truncation message SHALL show "(15 matches omitted)"

### Requirement: Parse Error Handling

The system SHALL handle parse errors gracefully.

#### Scenario: Syntax error in file

- **WHEN** a file has syntax errors that prevent full parsing
- **THEN** tree-sitter SHALL produce a partial tree (error-tolerant)
- **AND** the query SHALL still execute against valid portions
- **AND** no error SHALL be raised for partial parses

#### Scenario: Completely unparseable file

- **WHEN** a file cannot be parsed at all (e.g., binary disguised as source)
- **THEN** the file SHALL be skipped
- **AND** a warning SHALL be logged if `showStderr: true`

### Requirement: Command Action Behavior

The system SHALL handle command failures according to the configured action.

#### Scenario: Block action (default)

- **WHEN** a constraint is violated with `action: block` or no action specified
- **THEN** the hook SHALL return a blocked result
- **AND** Claude SHALL receive the error message

#### Scenario: Warn action

- **WHEN** a constraint is violated with `action: warn`
- **THEN** a warning SHALL be logged to stderr
- **AND** the hook SHALL continue without blocking

### Requirement: Timeout Handling

The system SHALL enforce timeouts on ts command execution.

#### Scenario: Timeout exceeded

- **WHEN** `timeout: 30` is configured
- **AND** parsing and querying takes longer than 30 seconds
- **THEN** execution SHALL be cancelled
- **AND** a timeout error SHALL be returned

#### Scenario: Normal completion

- **WHEN** execution completes before the timeout
- **THEN** results SHALL be processed normally
