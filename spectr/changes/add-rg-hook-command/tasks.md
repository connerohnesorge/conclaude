# Tasks

## 1. Dependencies

- [ ] 1.1 Add `grep-searcher` crate to Cargo.toml
- [ ] 1.2 Add `grep-regex` crate to Cargo.toml
- [ ] 1.3 Add `ignore` crate to Cargo.toml (for WalkBuilder)
- [ ] 1.4 Verify crate versions are compatible and build succeeds

## 2. Configuration Schema

- [ ] 2.1 Create `RgConfig` struct with all fields:
  - Required: `pattern`, `files`
  - Constraints: `max`, `min`, `equal`
  - Regex: `ignore_case`, `smart_case`, `word`, `fixed_strings`, `multi_line`, `whole_line`, `dot_matches_new_line`, `unicode`
  - Walk: `max_depth`, `hidden`, `follow_links`, `max_filesize`, `git_ignore`, `ignore`, `parents`, `same_file_system`, `threads`, `types`
  - Search: `context`, `count_mode`, `invert_match`
- [ ] 2.2 Create `CountMode` enum: `Lines`, `Occurrences`
- [ ] 2.3 Create `CommandAction` enum: `Block`, `Warn`
- [ ] 2.4 Modify `StopCommand` to have optional `run` and optional `rg` fields
- [ ] 2.5 Modify `SubagentStopCommand` similarly
- [ ] 2.6 Add `action` field to both command types with default `Block`
- [ ] 2.7 Add serde rename attributes for camelCase (e.g., `ignoreCase`, `maxDepth`)
- [ ] 2.8 Implement Default for RgConfig with sensible defaults
- [ ] 2.9 Update JSON schema generation

## 3. Configuration Validation

- [ ] 3.1 Validate mutual exclusivity: exactly one of `run` or `rg` required
- [ ] 3.2 Validate constraint mutual exclusivity: at most one of `max`/`min`/`equal`
- [ ] 3.3 Validate `pattern` is valid regex (or literal if `fixedStrings: true`)
- [ ] 3.4 Validate `files` glob pattern syntax
- [ ] 3.5 Validate `types` array contains known type names
- [ ] 3.6 Validate `countMode` is `lines` or `occurrences`
- [ ] 3.7 Validate `action` is `block` or `warn`
- [ ] 3.8 Validate numeric ranges: `maxDepth >= 0`, `context >= 0`, `threads >= 1`

## 4. Search Implementation

- [ ] 4.1 Create `RgSearcher` struct to encapsulate search logic
- [ ] 4.2 Implement `RegexMatcherBuilder` configuration from `RgConfig`
- [ ] 4.3 Implement `WalkBuilder` configuration from `RgConfig`
- [ ] 4.4 Implement custom `Sink` for match counting and output collection
- [ ] 4.5 Implement lines vs occurrences counting modes
- [ ] 4.6 Implement context line collection
- [ ] 4.7 Integrate with `SearcherBuilder` for binary detection (quit on NUL)
- [ ] 4.8 Handle file type filtering via `Types` builder

## 5. Constraint Evaluation

- [ ] 5.1 Implement `max` constraint: count <= N
- [ ] 5.2 Implement `min` constraint: count >= N
- [ ] 5.3 Implement `equal` constraint: count == N
- [ ] 5.4 Implement default constraint: `max: 0` when none specified
- [ ] 5.5 Format constraint violation messages:
  - "Found N matches, maximum allowed is M"
  - "Found N matches, minimum required is M"
  - "Found N matches, expected exactly M"

## 6. Command Execution Integration

- [ ] 6.1 Create `RgCommandConfig` struct (parallel to `StopCommandConfig`)
- [ ] 6.2 Modify `collect_stop_commands` to handle `rg` commands
- [ ] 6.3 Modify `collect_subagent_stop_commands` similarly
- [ ] 6.4 Create `execute_rg_command` function
- [ ] 6.5 Integrate output limiting (`maxOutputLines`)
- [ ] 6.6 Integrate output display (`showStdout`, `showStderr`)
- [ ] 6.7 Implement `action: warn` behavior (log but don't block)
- [ ] 6.8 Implement timeout for rg searches

## 7. Error Handling

- [ ] 7.1 Handle empty glob (zero files matched) as validation error
- [ ] 7.2 Handle invalid regex pattern with helpful error
- [ ] 7.3 Handle invalid glob pattern with helpful error
- [ ] 7.4 Handle unknown file type names
- [ ] 7.5 Handle permission errors during file walk
- [ ] 7.6 Handle timeout gracefully

## 8. Testing

- [ ] 8.1 Unit tests for `RgConfig` parsing and defaults
- [ ] 8.2 Unit tests for mutual exclusivity validation (run vs rg)
- [ ] 8.3 Unit tests for constraint validation
- [ ] 8.4 Unit tests for regex option configuration
- [ ] 8.5 Unit tests for walk option configuration
- [ ] 8.6 Integration tests: basic pattern matching
- [ ] 8.7 Integration tests: constraint checking (max/min/equal)
- [ ] 8.8 Integration tests: count modes (lines vs occurrences)
- [ ] 8.9 Integration tests: action warn vs block
- [ ] 8.10 Integration tests: file type filtering
- [ ] 8.11 Integration tests: gitignore respect
- [ ] 8.12 Integration tests: empty glob error
- [ ] 8.13 Integration tests: context lines
- [ ] 8.14 Integration tests: output limiting

## 9. Documentation

- [ ] 9.1 Update default-config.yaml with rg command examples
- [ ] 9.2 Add comprehensive docstrings to RgConfig and all fields
- [ ] 9.3 Document available file types (from ignore crate)
- [ ] 9.4 Add examples for common use cases in doc comments
