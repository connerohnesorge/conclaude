# Tasks

## 1. Dependencies

- [x] 1.1 Add `grep-searcher` crate to Cargo.toml
- [x] 1.2 Add `grep-regex` crate to Cargo.toml
- [x] 1.3 Add `ignore` crate to Cargo.toml (for WalkBuilder)
- [x] 1.4 Verify crate versions are compatible and build succeeds

## 2. Configuration Schema

- [x] 2.1 Create `RgConfig` struct with all fields:
  - Required: `pattern`, `files`
  - Constraints: `max`, `min`, `equal`
  - Regex: `ignore_case`, `smart_case`, `word`, `fixed_strings`, `multi_line`, `whole_line`, `dot_matches_new_line`, `unicode`
  - Walk: `max_depth`, `hidden`, `follow_links`, `max_filesize`, `git_ignore`, `ignore`, `parents`, `same_file_system`, `threads`, `types`
  - Search: `context`, `count_mode`, `invert_match`
- [x] 2.2 Create `CountMode` enum: `Lines`, `Occurrences`
- [x] 2.3 Create `CommandAction` enum: `Block`, `Warn`
- [x] 2.4 Modify `StopCommand` to have optional `run` and optional `rg` fields
- [x] 2.5 Modify `SubagentStopCommand` similarly
- [x] 2.6 Add `action` field to both command types with default `Block`
- [x] 2.7 Add serde rename attributes for camelCase (e.g., `ignoreCase`, `maxDepth`)
- [x] 2.8 Implement Default for RgConfig with sensible defaults
- [x] 2.9 Update JSON schema generation

## 3. Configuration Validation

- [x] 3.1 Validate mutual exclusivity: exactly one of `run` or `rg` required
- [x] 3.2 Validate constraint mutual exclusivity: at most one of `max`/`min`/`equal`
- [x] 3.3 Validate `pattern` is valid regex (or literal if `fixedStrings: true`)
- [x] 3.4 Validate `files` glob pattern syntax
- [x] 3.5 Validate `types` array contains known type names
- [x] 3.6 Validate `countMode` is `lines` or `occurrences`
- [x] 3.7 Validate `action` is `block` or `warn`
- [x] 3.8 Validate numeric ranges: `maxDepth >= 0`, `context >= 0`, `threads >= 1`

## 4. Search Implementation

- [x] 4.1 Create `RgSearcher` struct to encapsulate search logic
- [x] 4.2 Implement `RegexMatcherBuilder` configuration from `RgConfig`
- [x] 4.3 Implement `WalkBuilder` configuration from `RgConfig`
- [x] 4.4 Implement custom `Sink` for match counting and output collection
- [x] 4.5 Implement lines vs occurrences counting modes
- [x] 4.6 Implement context line collection
- [x] 4.7 Integrate with `SearcherBuilder` for binary detection (quit on NUL)
- [x] 4.8 Handle file type filtering via `Types` builder

## 5. Constraint Evaluation

- [x] 5.1 Implement `max` constraint: count <= N
- [x] 5.2 Implement `min` constraint: count >= N
- [x] 5.3 Implement `equal` constraint: count == N
- [x] 5.4 Implement default constraint: `max: 0` when none specified
- [x] 5.5 Format constraint violation messages:
  - "Found N matches, maximum allowed is M"
  - "Found N matches, minimum required is M"
  - "Found N matches, expected exactly M"

## 6. Command Execution Integration

- [x] 6.1 Create `RgCommandConfig` struct (parallel to `StopCommandConfig`)
- [x] 6.2 Modify `collect_stop_commands` to handle `rg` commands
- [x] 6.3 Modify `collect_subagent_stop_commands` similarly
- [x] 6.4 Create `execute_rg_command` function
- [x] 6.5 Integrate output limiting (`maxOutputLines`)
- [x] 6.6 Integrate output display (`showStdout`, `showStderr`)
- [x] 6.7 Implement `action: warn` behavior (log but don't block)
- [x] 6.8 Implement timeout for rg searches

## 7. Error Handling

- [x] 7.1 Handle empty glob (zero files matched) as validation error
- [x] 7.2 Handle invalid regex pattern with helpful error
- [x] 7.3 Handle invalid glob pattern with helpful error
- [x] 7.4 Handle unknown file type names
- [x] 7.5 Handle permission errors during file walk
- [x] 7.6 Handle timeout gracefully

## 8. Testing

- [x] 8.1 Unit tests for `RgConfig` parsing and defaults
- [x] 8.2 Unit tests for mutual exclusivity validation (run vs rg)
- [x] 8.3 Unit tests for constraint validation
- [x] 8.4 Unit tests for regex option configuration
- [x] 8.5 Unit tests for walk option configuration
- [x] 8.6 Integration tests: basic pattern matching
- [x] 8.7 Integration tests: constraint checking (max/min/equal)
- [x] 8.8 Integration tests: count modes (lines vs occurrences)
- [x] 8.9 Integration tests: action warn vs block
- [x] 8.10 Integration tests: file type filtering
- [x] 8.11 Integration tests: gitignore respect
- [x] 8.12 Integration tests: empty glob error
- [x] 8.13 Integration tests: context lines
- [x] 8.14 Integration tests: output limiting

## 9. Documentation

- [x] 9.1 Update default-config.yaml with rg command examples
- [x] 9.2 Add comprehensive docstrings to RgConfig and all fields
- [x] 9.3 Document available file types (from ignore crate)
- [x] 9.4 Add examples for common use cases in doc comments
