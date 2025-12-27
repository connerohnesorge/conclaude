# Fix Broken preventAdditions Setting in preToolUse

## Why

The `preventAdditions` configuration field exists in `preToolUse` but is completely non-functional. The field is properly defined in the config schema and can be set by users, but the hook implementation (`src/hooks.rs`) never checks or enforces this setting. This means users who configure `preventAdditions` with glob patterns expect those patterns to prevent file creation, but the system silently ignores them.

**Current behavior:**
- User sets `preToolUse.preventAdditions: ["dist/**", "build/**"]`
- Configuration loads successfully (field is defined)
- Claude attempts to create `dist/output.js` via `Write` tool
- **BUG**: File creation succeeds (should be blocked)
- `check_file_validation_rules()` only checks `preventRootAdditions` and `uneditableFiles`, never `preventAdditions`

**Expected behavior:**
- User sets `preToolUse.preventAdditions: ["dist/**", "build/**"]`
- Claude attempts to create `dist/output.js` via `Write` tool
- Hook blocks the operation with error: "Blocked Write operation: file matches preventAdditions pattern 'dist/**'. File: dist/output.js"

This is a critical bug because:
1. **Silent failure** - Users think their config is protecting directories when it isn't
2. **Security risk** - Protection patterns are not enforced, potentially allowing unwanted file creation
3. **Configuration confusion** - The field exists and validates, creating false expectations

## What

Implement the missing `preventAdditions` enforcement logic in the `PreToolUse` hook using **Test-Driven Development**. The implementation will:

1. **Add TDD test suite first** - Write comprehensive failing tests that define exact behavior
2. **Check `preventAdditions` patterns** in `check_file_validation_rules()` for `Write` tool operations
3. **Match file paths against glob patterns** using the existing `Pattern::matches()` logic (same as `uneditableFiles`)
4. **Block matching operations** with clear error messages indicating which pattern matched
5. **Only apply to Write tool** - `preventAdditions` should only prevent file creation, not editing existing files

## How

### Phase 1: Test-Driven Development (TDD)
Create comprehensive test suite **before** implementation:

**Test file: `tests/prevent_additions_tests.rs`**

1. **Tests for basic glob matching**
   - Exact directory match: `preventAdditions: ["dist"]` blocks `dist/output.js`
   - Wildcard patterns: `preventAdditions: ["build/**"]` blocks `build/nested/file.js`
   - File extension patterns: `preventAdditions: ["*.log"]` blocks `debug.log`
   - Multiple patterns: Array of patterns enforces all
   - Non-matching paths allowed: `src/main.rs` proceeds when only `dist/**` blocked

2. **Tests for Write-tool-only enforcement**
   - Write tool blocked: `preventAdditions: ["dist/**"]` blocks Write to `dist/file.js`
   - Edit tool allowed: `preventAdditions: ["dist/**"]` allows Edit on existing `dist/file.js`
   - NotebookEdit allowed: Editing existing notebooks not blocked by preventAdditions

3. **Tests for path normalization**
   - Relative path: `preventAdditions: ["./dist/**"]` normalizes and blocks correctly
   - Absolute path: Resolved paths match patterns correctly
   - Parent directory refs: `src/../dist/file.js` normalized to `dist/file.js` and blocked

4. **Tests for error messages**
   - Clear error format: "Blocked Write operation: file matches preventAdditions pattern '{pattern}'. File: {path}"
   - Pattern included: Error message shows which pattern matched
   - Tool name included: Error identifies Write tool

5. **Tests for interaction with other rules**
   - preventAdditions + preventRootAdditions: Both enforced independently
   - preventAdditions + uneditableFiles: Blocks for Write if either matches
   - preventAdditions only affects Write: Edit operations check uneditableFiles but not preventAdditions

6. **Edge case tests**
   - Empty preventAdditions array: No blocking
   - Existing file with Write tool: Should allow (Write can overwrite existing files)
   - Invalid glob pattern: Graceful error handling
   - Pattern with trailing slash: Directory patterns work correctly

**Run tests (all should fail initially):**
```bash
cargo test prevent_additions --lib -- --nocapture
```

### Phase 2: Implementation
**Only after tests are written and failing**, implement the fix in `src/hooks.rs`:

1. **Modify `check_file_validation_rules()` function** (around line 281):
   ```rust
   // Add after existing uneditableFiles check (around line 338):

   // Check preventAdditions rule - only applies to Write tool
   if payload.tool_name == "Write" {
       for pattern in &config.pre_tool_use.prevent_additions {
           if matches_uneditable_pattern(
               &file_path,
               &relative_path,
               &resolved_path.to_string_lossy(),
               pattern,
           )? {
               let error_message = format!(
                   "Blocked {} operation: file matches preventAdditions pattern '{}'. File: {}",
                   payload.tool_name, pattern, file_path
               );

               eprintln!(
                   "PreToolUse blocked by preventAdditions rule: tool_name={}, file_path={}, pattern={}",
                   payload.tool_name,
                   file_path,
                   pattern
               );

               return Ok(Some(HookResult::blocked(error_message)));
           }
       }
   }
   ```

2. **Verify implementation**:
   - Re-run test suite: `cargo test prevent_additions`
   - All tests should now pass
   - Run integration tests: `cargo test --test integration_tests`
   - Run hook tests: `cargo test --test hooks_tests`

3. **Manual verification**:
   - Create test config with `preventAdditions: ["dist/**"]`
   - Attempt Write to `dist/test.js` via CLI
   - Verify blocking occurs with correct error message

### Phase 3: Documentation
Update after implementation is confirmed working:

1. **README.md** - Update preventAdditions documentation with working examples
2. **CHANGELOG.md** - Add bug fix entry
3. **default-config.yaml** - Ensure examples are accurate

## Implementation Details

### Key Technical Decisions

1. **Reuse existing pattern matching**: Use `matches_uneditable_pattern()` function (already exists and handles glob matching correctly)

2. **Write-tool-only semantics**: `preventAdditions` should only block file creation/writing, not editing existing files. This is distinct from `uneditableFiles` which blocks all modifications.

3. **Error message format**: Follow existing pattern from `uneditableFiles` for consistency:
   ```
   Blocked Write operation: file matches preventAdditions pattern 'dist/**'. File: dist/output.js
   ```

4. **Ordering**: Check `preventAdditions` after `preventRootAdditions` and `uneditableFiles` to maintain existing validation priority.

5. **Pattern syntax**: Use same glob syntax as `uneditableFiles` (supports `**`, `*`, `?`, character classes)

### Test Strategy

**TDD Approach:**
1. Write all tests first (they will fail)
2. Run tests to confirm failures: `cargo test prevent_additions`
3. Implement minimal code to make tests pass
4. Refactor if needed
5. Ensure all existing tests still pass

**Test Coverage Goals:**
- Line coverage: 100% of new `preventAdditions` code path
- Branch coverage: All pattern matching branches
- Integration: Tests cover interaction with existing rules

## Success Criteria

- **TDD test suite created** - Comprehensive tests written before implementation
- **All tests failing initially** - Confirms tests actually validate the fix
- **Implementation makes tests pass** - Core functionality works
- **No regressions** - Existing tests still pass
- **Manual validation** - Real-world config blocks file creation as expected
- **Error messages clear** - Users understand why operations were blocked
- **Documentation updated** - README, CHANGELOG, and examples reflect fix

## Non-Goals

- Changing the behavior of `uneditableFiles` (remains as-is)
- Modifying `preventRootAdditions` logic (orthogonal feature)
- Adding new configuration fields (only fixing existing field)
- Changing config schema (field already exists)

## Risks & Mitigation

**Risk:** Implementation differs from test expectations
**Mitigation:** TDD ensures implementation matches test spec

**Risk:** Breaking existing behavior
**Mitigation:** Run full test suite before/after changes

**Risk:** Pattern matching edge cases
**Mitigation:** Comprehensive edge case tests in Phase 1

**Risk:** Performance impact from additional pattern checks
**Mitigation:** Only checks patterns when `preventAdditions` is non-empty (minimal impact)
