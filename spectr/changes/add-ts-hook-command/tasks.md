# Tasks

## 1. Dependencies

- [ ] 1.1 Add `tree-sitter` crate to Cargo.toml
- [ ] 1.2 Evaluate grammar bundling approach: `rs-tree-sitter-languages` vs individual crates
- [ ] 1.3 Add grammar crates for all 25 languages
- [ ] 1.4 Verify crate versions are compatible and build succeeds
- [ ] 1.5 Measure binary size impact

## 2. Language Registry

- [ ] 2.1 Create `LanguageRegistry` struct to manage loaded grammars
- [ ] 2.2 Implement file extension to language mapping
- [ ] 2.3 Support all 25 bundled languages with correct extensions
- [ ] 2.4 Implement `get_language(extension: &str)` method
- [ ] 2.5 Handle unknown extensions gracefully (skip file with warning)

## 3. Configuration Schema

- [ ] 3.1 Create `TsConfig` struct with all fields:
  - Required: `query`, `files`
  - Constraints: `max`, `min`, `equal`
  - Options: `capture`, `language`
  - Walk options (shared with RgConfig)
- [ ] 3.2 Modify `StopCommand` to add optional `ts` field
- [ ] 3.3 Modify `SubagentStopCommand` similarly
- [ ] 3.4 Add serde rename attributes for camelCase
- [ ] 3.5 Implement Default for TsConfig
- [ ] 3.6 Ensure mutual exclusivity: exactly one of `run`, `rg`, or `ts`
- [ ] 3.7 Update JSON schema generation

## 4. Configuration Validation

- [ ] 4.1 Validate mutual exclusivity: exactly one of `run`/`rg`/`ts` required
- [ ] 4.2 Validate constraint mutual exclusivity: at most one of `max`/`min`/`equal`
- [ ] 4.3 Validate `query` is valid tree-sitter S-expression syntax
- [ ] 4.4 Validate `capture` references a capture defined in query
- [ ] 4.5 Validate `language` is a supported language name
- [ ] 4.6 Validate `files` glob pattern syntax

## 5. Query Compilation

- [ ] 5.1 Create `CompiledQuery` struct wrapping tree-sitter Query
- [ ] 5.2 Implement query compilation with helpful error messages
- [ ] 5.3 Extract capture names from compiled query
- [ ] 5.4 Validate specified capture exists in query
- [ ] 5.5 Determine default capture (first in query) when not specified

## 6. Search Implementation

- [ ] 6.1 Create `TsSearcher` struct to encapsulate search logic
- [ ] 6.2 Integrate with `WalkBuilder` from ignore crate (shared with rg)
- [ ] 6.3 Implement per-file parsing with Parser
- [ ] 6.4 Implement query execution with QueryCursor
- [ ] 6.5 Implement capture counting for specified capture name
- [ ] 6.6 Collect matched node info: file, line, column, node_type, text
- [ ] 6.7 Handle parse errors gracefully (skip file, log warning)
- [ ] 6.8 Handle query execution errors

## 7. Constraint Evaluation

- [ ] 7.1 Implement `max` constraint: count <= N
- [ ] 7.2 Implement `min` constraint: count >= N
- [ ] 7.3 Implement `equal` constraint: count == N
- [ ] 7.4 Implement default constraint: `max: 0` when none specified
- [ ] 7.5 Format constraint violation messages with capture name

## 8. Output Formatting

- [ ] 8.1 Implement detailed output format: `file:line:col [node_type]: text`
- [ ] 8.2 Truncate long captured text (single line, max ~100 chars)
- [ ] 8.3 Apply `maxOutputLines` limit
- [ ] 8.4 Include truncation indicator when output limited

## 9. Command Execution Integration

- [ ] 9.1 Create `TsCommandConfig` struct (parallel to StopCommandConfig, RgCommandConfig)
- [ ] 9.2 Modify `collect_stop_commands` to handle `ts` commands
- [ ] 9.3 Modify `collect_subagent_stop_commands` similarly
- [ ] 9.4 Create `execute_ts_command` function
- [ ] 9.5 Integrate with `action: warn` behavior
- [ ] 9.6 Implement timeout for ts searches

## 10. Error Handling

- [ ] 10.1 Handle empty glob (zero files matched) as error
- [ ] 10.2 Handle invalid query syntax with helpful error
- [ ] 10.3 Handle invalid capture reference
- [ ] 10.4 Handle unsupported language
- [ ] 10.5 Handle file read errors
- [ ] 10.6 Handle parse errors per file (skip, warn)
- [ ] 10.7 Handle timeout gracefully

## 11. Testing

- [ ] 11.1 Unit tests for `TsConfig` parsing and defaults
- [ ] 11.2 Unit tests for mutual exclusivity validation
- [ ] 11.3 Unit tests for query compilation
- [ ] 11.4 Unit tests for language detection from extension
- [ ] 11.5 Integration tests: basic query matching (Rust)
- [ ] 11.6 Integration tests: predicate filtering (#eq?, #match?)
- [ ] 11.7 Integration tests: constraint checking (max/min/equal)
- [ ] 11.8 Integration tests: multiple languages (JS, Python)
- [ ] 11.9 Integration tests: action warn vs block
- [ ] 11.10 Integration tests: output formatting
- [ ] 11.11 Integration tests: parse error handling

## 12. Documentation

- [ ] 12.1 Update default-config.yaml with ts command examples
- [ ] 12.2 Add docstrings to TsConfig and all fields
- [ ] 12.3 Document supported languages and file extensions
- [ ] 12.4 Document query syntax with examples
- [ ] 12.5 Add common query patterns for each language
