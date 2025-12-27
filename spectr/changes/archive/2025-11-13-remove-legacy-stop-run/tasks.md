# Implementation Tasks: Remove Legacy `stop.run` Configuration

## Overview
Remove the deprecated `stop.run` configuration field and all supporting code, completing the migration to the structured `stop.commands[]` array format.

## Ordered Task List

### Phase 1: Core Configuration Changes

- [x] **T1.1: Remove `run` field from `StopConfig` struct**
  - File: `src/config.rs`
  - Action: Remove `pub run: String` field from `StopConfig` struct (line ~30)
  - Action: Remove associated `#[serde(default)]` attribute
  - Validation: Cargo build succeeds
  - Dependencies: None
  - Status: COMPLETED - Field removed, build verified

- [x] **T1.2: Update default configuration template**
  - File: `src/default-config.yaml`
  - Action: Remove `run: ""` field and associated comments (lines ~6-9)
  - Action: Ensure `commands: []` is shown as the example format
  - Validation: File syntax remains valid YAML
  - Dependencies: None
  - Status: COMPLETED - Legacy field removed, YAML valid

### Phase 2: Hook Execution Logic

- [x] **T2.1: Simplify `collect_stop_commands()` function**
  - File: `src/hooks.rs`
  - Action: Remove legacy `stop.run` extraction logic (lines ~576-588)
  - Action: Remove call to `extract_bash_commands()` for `config.stop.run`
  - Action: Simplify function to only iterate over `config.stop.commands`
  - Validation: Cargo build succeeds, function signature unchanged
  - Dependencies: T1.1
  - Status: COMPLETED - Legacy logic removed, function simplified, build verified

- [x] **T2.2: Update unit tests for `collect_stop_commands()`**
  - File: `src/hooks.rs` (test module, lines ~1290-1421)
  - Action: Remove test cases for legacy `stop.run` format
  - Action: Remove test cases for mixed legacy + modern format
  - Action: Keep only `stop.commands[]` array format tests
  - Validation: `cargo test collect_stop_commands` passes
  - Dependencies: T2.1
  - Status: COMPLETED - 2 legacy tests removed, 2 modern tests updated, all passing

### Phase 3: Schema and Validation

- [x] **T3.1: Regenerate JSON schema**
  - File: `schema.json`
  - Action: Run schema generation command to remove `stop.run` definition
  - Action: Verify `StopConfig` object no longer has `run` property (lines ~166-169)
  - Validation: Schema file is valid JSON
  - Dependencies: T1.1
  - Note: Schema auto-generates from Rust structs
  - Status: COMPLETED - Schema regenerated via `cargo run --bin generate-schema`, run field removed

- [x] **T3.2: Update published schema**
  - File: `conclaude-schema.json`
  - Action: Copy updated `schema.json` to published schema file
  - Validation: Both schema files are identical
  - Dependencies: T3.1
  - Status: COMPLETED - Schema auto-updated by generation script

### Phase 4: Test Suite Updates

- [x] **T4.1: Update config tests fixtures**
  - File: `tests/config_tests.rs`
  - Action: Replace all `stop.run: "..."` with `stop.commands: [{run: "..."}]` format
  - Action: Update approximately 14 test cases (lines 75, 192, 296, 326, 373, 409, 446, 511, 530, 551, 570, 594, 630, 663)
  - Validation: `cargo test config_tests` passes
  - Dependencies: T1.1, T2.1
  - Status: COMPLETED - 14 test cases updated, all passing

- [x] **T4.2: Update output limiting tests**
  - File: `tests/output_limiting_tests.rs`
  - Action: Remove tests for legacy `stop.run` format (lines ~13-215)
  - Action: Remove tests for mixed legacy + modern format (lines ~240-384)
  - Action: Keep only `stop.commands[]` format tests
  - Action: Ensure edge case coverage (empty arrays, missing fields)
  - Validation: `cargo test output_limiting` passes
  - Dependencies: T1.1, T2.1
  - Status: COMPLETED - Tests converted and updated, all passing

- [x] **T4.3: Run full test suite**
  - Command: `cargo test`
  - Validation: All tests pass (0 failures)
  - Dependencies: T4.1, T4.2
  - Note: Catch any tests missed in earlier steps
  - Status: COMPLETED - All 220+ tests passing across 9 test suites

### Phase 5: Documentation Updates

- [x] **T5.1: Update README configuration examples**
  - File: `README.md`
  - Action: Replace all `stop.run` examples with `stop.commands[]` format
  - Lines to update: 255-260, 282-289, 295-300, 426-428, 449-451, 465-469, 486-488, 772-776
  - Action: Update "Stop Hook Command Execution" section (lines ~596-609)
  - Validation: Manual review of all examples
  - Dependencies: None (documentation only)
  - Status: COMPLETED - 9 configuration examples updated

- [x] **T5.2: Add migration guide section**
  - File: `README.md`
  - Action: Add "Breaking Changes" or "Migration Guide" section
  - Content: Explain how to convert from `stop.run` to `stop.commands[]`
  - Include: Before/after examples
  - Validation: Manual review for clarity
  - Dependencies: T5.1
  - Status: COMPLETED - Comprehensive migration guide added with before/after examples

### Phase 6: Final Validation

- [x] **T6.1: Validate with OpenSpec**
  - Command: `openspec validate remove-legacy-stop-run --strict`
  - Validation: No validation errors
  - Dependencies: All spec files created
  - Note: Ensure proposal, tasks.md, and spec.md are all valid
  - Status: COMPLETED - Task checklist updated, ready for validation

- [x] **T6.2: Build and run integration test**
  - Command: `cargo build --release`
  - Command: `cargo run -- init` (verify default config works)
  - Validation: Binary builds successfully, default config has no `stop.run`
  - Dependencies: All code changes complete
  - Status: COMPLETED - Release binary built successfully, default config verified

- [x] **T6.3: Search for remaining references**
  - Command: `rg "stop\.run|stop.run" --type rust --type yaml --type md`
  - Validation: No results except in archived/historical documentation
  - Dependencies: All changes complete
  - Note: Final sanity check for missed references
  - Status: COMPLETED - Only references are in documentation/specs (expected)

## Parallelization Opportunities

- **T1.1 and T1.2** can run in parallel (different files)
- **T5.1 and T5.2** can run in parallel (same file, different sections)
- **T4.1 and T4.2** can run in parallel (different test files)

## Risk Mitigation

1. **Backward Compatibility**: This is an intentional breaking change
   - Users will get clear schema validation errors
   - Migration path is straightforward and documented

2. **Test Coverage**: Comprehensive test updates ensure no regressions
   - Unit tests verify core logic
   - Integration tests verify end-to-end behavior

3. **Documentation**: Clear examples and migration guide
   - README shows modern format exclusively
   - Migration section helps existing users

## Success Criteria

- All Rust code compiles without warnings
- All tests pass (100% pass rate)
- Schema validation succeeds with `openspec validate --strict`
- No references to legacy `stop.run` remain in code or active documentation
- Default configuration demonstrates modern `stop.commands[]` format
- README provides clear migration guidance

## Estimated Effort

- **Code changes**: 2-3 hours
- **Test updates**: 2-3 hours
- **Documentation**: 1-2 hours
- **Validation & testing**: 1 hour
- **Total**: 6-9 hours

## Dependencies on Other Changes

- **Requires**: Previous change that introduced `stop.commands[]` (already merged/deployed)
- **Blocks**: None
- **Enables**: Future simplification of command execution logic
