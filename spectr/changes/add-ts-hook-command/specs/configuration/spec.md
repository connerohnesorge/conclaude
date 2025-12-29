## ADDED Requirements

### Requirement: TsConfig Structure

The system SHALL define a `TsConfig` struct for tree-sitter-based structural code analysis.

#### Scenario: TsConfig required fields

- **WHEN** parsing a ts command configuration
- **THEN** `query` field SHALL be required (tree-sitter S-expression query)
- **AND** `files` field SHALL be required (glob pattern for file matching)

#### Scenario: TsConfig query options

- **WHEN** a ts configuration includes query options
- **THEN** the following fields SHALL be supported:
  - `capture` (Option<String>, default: first capture in query) - which capture to count
  - `language` (Option<String>, default: auto-detect) - override language detection

#### Scenario: TsConfig constraint fields

- **WHEN** a ts configuration includes constraint fields
- **THEN** `max`, `min`, and `equal` SHALL be mutually exclusive
- **AND** each SHALL accept a non-negative integer
- **AND** if none specified, `max: 0` SHALL be implicit

#### Scenario: TsConfig file walking options

- **WHEN** a ts configuration includes file walking options
- **THEN** the same options as `RgConfig` SHALL be supported:
  - `maxDepth`, `hidden`, `followLinks`, `maxFilesize`
  - `gitIgnore`, `ignore`, `parents`, `sameFileSystem`
  - `threads`

### Requirement: Mutual Exclusivity of run, rg, and ts

The system SHALL enforce that each command entry specifies exactly one of `run`, `rg`, or `ts`.

#### Scenario: Command with only ts field

- **WHEN** a stop command has a `ts` configuration and no `run` or `rg` field
- **THEN** the configuration SHALL be accepted
- **AND** the command SHALL execute as a tree-sitter query

#### Scenario: Command with ts and rg fields

- **WHEN** a stop command has both `ts` and `rg` fields
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate that `run`, `rg`, and `ts` are mutually exclusive

#### Scenario: Command with ts and run fields

- **WHEN** a stop command has both `ts` and `run` fields
- **THEN** configuration loading SHALL fail with a validation error

#### Scenario: Command with all three fields

- **WHEN** a stop command has `run`, `rg`, and `ts` fields
- **THEN** configuration loading SHALL fail with a validation error

### Requirement: Query Validation

The system SHALL validate tree-sitter queries at configuration load time.

#### Scenario: Valid S-expression query

- **WHEN** `query: "(function_item) @fn"` is configured
- **THEN** the configuration SHALL be accepted
- **AND** the query SHALL compile successfully

#### Scenario: Invalid S-expression syntax

- **WHEN** `query: "(unclosed_paren"` is configured
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL include the tree-sitter query parse error

#### Scenario: Query with predicates

- **WHEN** `query: "(identifier) @id (#eq? @id \"main\")"` is configured
- **THEN** the configuration SHALL be accepted
- **AND** predicates SHALL be included in the compiled query

#### Scenario: Query with undefined capture in predicate

- **WHEN** `query: "(identifier) @id (#eq? @undefined \"test\")"` is configured
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the undefined capture

### Requirement: Capture Validation

The system SHALL validate that specified captures exist in the query.

#### Scenario: Valid capture reference

- **WHEN** `capture: "@fn"` is configured
- **AND** the query contains `@fn` capture
- **THEN** the configuration SHALL be accepted

#### Scenario: Invalid capture reference

- **WHEN** `capture: "@nonexistent"` is configured
- **AND** the query does not contain `@nonexistent` capture
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL list available captures

#### Scenario: Default capture selection

- **WHEN** no `capture` field is specified
- **AND** the query contains captures `@first`, `@second`
- **THEN** `@first` SHALL be used as the default capture for counting

### Requirement: Language Validation

The system SHALL validate language names when explicitly specified.

#### Scenario: Valid language name

- **WHEN** `language: "rust"` is configured
- **THEN** the configuration SHALL be accepted

#### Scenario: Unknown language name

- **WHEN** `language: "unknownlang"` is configured
- **THEN** configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the unknown language
- **AND** the error message SHALL list similar supported languages if available

### Requirement: Supported Languages

The system SHALL support parsing files in the following languages (bundled grammars):

#### Scenario: Primary languages

- **WHEN** files with extensions `.rs`, `.js`, `.ts`, `.py`, `.go`, `.java`, `.c`, `.cpp` are processed
- **THEN** the corresponding bundled grammar SHALL be used
- **AND** queries SHALL execute successfully

#### Scenario: Web languages

- **WHEN** files with extensions `.html`, `.css`, `.json`, `.yaml`, `.toml`, `.md` are processed
- **THEN** the corresponding bundled grammar SHALL be used

#### Scenario: Additional languages

- **WHEN** files with extensions for Ruby, PHP, C#, Swift, Kotlin, Bash, SQL, Lua, Zig, Haskell, OCaml are processed
- **THEN** the corresponding bundled grammar SHALL be used

### Requirement: SubagentStopCommand ts Support

The system SHALL support the `ts` field in subagent stop commands with identical structure to regular stop commands.

#### Scenario: SubagentStop with ts command

- **WHEN** a subagentStop command has a `ts` configuration
- **THEN** the configuration SHALL be accepted
- **AND** all ts options SHALL work identically to regular stop commands
