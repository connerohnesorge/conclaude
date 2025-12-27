# Design Document: Fix Broken preventAdditions Setting

## Problem Analysis

### Current State

The `preventAdditions` configuration field in `preToolUse` exists in the schema but is **completely non-functional**:

```yaml
# User configuration (in .conclaude.yaml)
preToolUse:
  preventAdditions:
    - "dist/**"
    - "build/**"
    - "*.log"
```

**What happens today:**
1. Configuration loads successfully [OK]
2. User expects file creation to be blocked in `dist/`, `build/`, and for `*.log` files [NO]
3. Claude executes `Write` tool to create `dist/output.js` [OK] (should be blocked!)
4. Hook runs `check_file_validation_rules()` in `src/hooks.rs` [OK]
5. Function checks `preventRootAdditions` [OK]
6. Function checks `uneditableFiles` [OK]
7. **Function never checks `preventAdditions`** [NO]
8. File creation succeeds (WRONG) [NO]

**The bug:** Lines 298-338 in `src/hooks.rs:check_file_validation_rules()` implement checks for `preventRootAdditions` and `uneditableFiles`, but there is zero code that checks `config.pre_tool_use.prevent_additions`.

### Root Cause

The `preventAdditions` field was added to the config schema (`src/config.rs`, line 86) but the corresponding enforcement logic was never implemented in the hook handler. This is a **"dead configuration"** bug pattern where:

1. Schema defines field [YES]
2. Config parsing accepts field [YES]
3. Runtime logic ignores field [NO]

### Impact

**User Impact:**
- **Silent security failure** - Protection patterns don't work
- **False sense of security** - Config looks correct but provides no protection
- **Difficult debugging** - No error, warning, or indication that preventAdditions is ignored

**System Impact:**
- **Configuration inconsistency** - Only 2 of 3 file protection fields work
- **Documentation confusion** - README describes a feature that doesn't exist

## Solution Design

### High-Level Approach

**Fix strategy:** Add the missing enforcement logic to `check_file_validation_rules()` using TDD.

**Key principle:** `preventAdditions` should **only block Write operations**, not edits to existing files. This differentiates it from `uneditableFiles` (which blocks all modifications).

### Implementation Location

**File:** `src/hooks.rs`
**Function:** `check_file_validation_rules()` (lines 281-341)
**Insertion point:** After `uneditableFiles` check, before returning `Ok(None)` (around line 338)

```rust
// Existing code (line 317-338)
for pattern in &config.rules.uneditable_files {
    if matches_uneditable_pattern(...) {
        return Ok(Some(HookResult::blocked(error_message)));
    }
}

// NEW CODE TO INSERT HERE:
// Check preventAdditions rule - only applies to Write tool
if payload.tool_name == "Write" {
    for pattern in &config.pre_tool_use.prevent_additions {
        if matches_uneditable_pattern(...) {
            return Ok(Some(HookResult::blocked(error_message)));
        }
    }
}

Ok(None) // Line 341
```

### Detailed Logic Flow

```
┌─────────────────────────────────────────┐
│ PreToolUse Hook Triggered               │
│ Tool: Write, Edit, NotebookEdit         │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│ check_file_validation_rules()           │
│ Extract file_path from tool_input       │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│ Check: preventRootAdditions             │
│ (Write tool only, root-level files)     │
└─────────────┬───────────────────────────┘
              │ Blocked? ──Yes──> Return HookResult::blocked()
              │
              No
              ▼
┌─────────────────────────────────────────┐
│ Check: uneditableFiles                  │
│ (All file-modifying tools, all files)   │
└─────────────┬───────────────────────────┘
              │ Blocked? ──Yes──> Return HookResult::blocked()
              │
              No
              ▼
┌─────────────────────────────────────────┐
│ NEW: Check preventAdditions             │◄─── THIS IS THE FIX
│ (Write tool only, pattern-matched files)│
└─────────────┬───────────────────────────┘
              │ Blocked? ──Yes──> Return HookResult::blocked()
              │
              No
              ▼
┌─────────────────────────────────────────┐
│ All checks passed                       │
│ Return Ok(None) - Allow operation       │
└─────────────────────────────────────────┘
```

### Behavioral Specification

#### What gets blocked?

| Tool   | preventAdditions Check? | Why                                           |
|--------|-------------------------|-----------------------------------------------|
| Write  | YES                     | Creating new files - main use case           |
| Edit   | NO                      | Editing existing files - use uneditableFiles |
| NotebookEdit | NO                | Editing existing notebooks                    |

#### Pattern Matching Examples

Given config:
```yaml
preToolUse:
  preventAdditions:
    - "dist/**"
    - "build/**"
    - "*.log"
```

| Operation | Path | Tool | Result | Reason |
|-----------|------|------|--------|--------|
| Create | `dist/output.js` | Write | BLOCKED | Matches `dist/**` |
| Create | `dist/nested/deep/file.js` | Write | BLOCKED | Matches `dist/**` (recursive) |
| Create | `build/app.js` | Write | BLOCKED | Matches `build/**` |
| Create | `debug.log` | Write | BLOCKED | Matches `*.log` |
| Create | `src/main.rs` | Write | ALLOWED | No pattern match |
| Edit | `dist/existing.js` | Edit | ALLOWED | Not Write tool |
| Edit | `dist/existing.js` | Write | ALLOWED | File already exists (overwrite) |

### Error Message Design

**Format:**
```
Blocked {tool_name} operation: file matches preventAdditions pattern '{pattern}'. File: {file_path}
```

**Example:**
```
Blocked Write operation: file matches preventAdditions pattern 'dist/**'. File: dist/output.js
```

**Log output (stderr):**
```
PreToolUse blocked by preventAdditions rule: tool_name=Write, file_path=dist/output.js, pattern=dist/**
```

### Code Structure

#### Pseudocode

```rust
// In check_file_validation_rules(), after uneditableFiles check:

// Only check preventAdditions for Write tool
if payload.tool_name == "Write" {
    // Iterate through configured patterns
    for pattern in &config.pre_tool_use.prevent_additions {
        // Use existing pattern matching function
        if matches_uneditable_pattern(&file_path, &relative_path, &resolved_path, pattern)? {
            // Build error message
            let error_message = format!(
                "Blocked {} operation: file matches preventAdditions pattern '{}'. File: {}",
                payload.tool_name, pattern, file_path
            );

            // Log the block
            eprintln!(
                "PreToolUse blocked by preventAdditions rule: tool_name={}, file_path={}, pattern={}",
                payload.tool_name, file_path, pattern
            );

            // Block the operation
            return Ok(Some(HookResult::blocked(error_message)));
        }
    }
}

// If no patterns matched, continue with remaining checks
Ok(None)
```

### Edge Cases & Handling

1. **Empty preventAdditions array**
   - Behavior: No blocking (shortcut: skip loop)
   - Rationale: Common case, optimize for it

2. **Pattern matches but file exists**
   - Behavior: ALLOW (Write can overwrite existing files)
   - Rationale: preventAdditions is for creation, not modification
   - Implementation: Check `Path::exists()` before blocking

3. **Invalid glob pattern in config**
   - Behavior: `matches_uneditable_pattern()` returns `Err()`
   - Rationale: Propagate error to user (fail fast)
   - Implementation: Use `?` operator, error bubbles up

4. **Multiple patterns match**
   - Behavior: Block on first match, report that pattern
   - Rationale: Performance (early exit), simplicity
   - Implementation: Return immediately on first match

5. **Absolute vs relative paths**
   - Behavior: Try matching against all path forms
   - Rationale: User might use absolute or relative patterns
   - Implementation: `matches_uneditable_pattern()` already handles this

### Test-Driven Development Strategy

**TDD Cycle:**

1. **RED Phase** - Write failing tests
   ```bash
   cargo test prevent_additions --lib
   # Expected: All tests fail
   ```

2. **GREEN Phase** - Implement minimal code to pass tests
   ```rust
   // Add preventAdditions check to check_file_validation_rules()
   ```

3. **REFACTOR Phase** - Clean up code
   ```bash
   cargo clippy
   cargo fmt
   ```

**Test Categories:**

1. **Unit Tests** (24 tests)
   - Basic glob matching (5 tests)
   - Tool-specific enforcement (3 tests)
   - Path normalization (3 tests)
   - Error messages (4 tests)
   - Rule interactions (4 tests)
   - Edge cases (5 tests)

2. **Integration Tests** (reuse existing framework)
   - End-to-end hook invocation
   - Config loading + hook execution
   - Multiple rules combined

3. **Manual Tests** (validation)
   - Real config file
   - CLI invocation
   - Verify blocking behavior

### Interaction with Existing Features

#### preventRootAdditions
- **Purpose:** Block file creation at repository root
- **Scope:** Write tool only
- **Interaction:** Independent, both can block same operation
- **Priority:** Check before preventAdditions (existing order)

#### uneditableFiles
- **Purpose:** Block file modification (edit + write)
- **Scope:** All file-modifying tools
- **Interaction:** Independent, both can block same operation
- **Priority:** Check before preventAdditions (existing order)

#### preventGeneratedFileEdits
- **Purpose:** Block editing auto-generated files
- **Scope:** All file-modifying tools
- **Interaction:** Independent, different check function
- **Priority:** Checked separately in `check_auto_generated_file()`

**Combined example:**
```yaml
rules:
  preventRootAdditions: true  # Block: /foo.txt
  uneditableFiles:            # Block: /package.json (edit or write)
    - "package.json"

preToolUse:
  preventAdditions:           # Block: dist/* (write only)
    - "dist/**"
  preventGeneratedFileEdits: true  # Block: auto-generated files
```

Each rule is evaluated independently. If ANY rule blocks, the operation is denied.

### Performance Considerations

**Overhead:**
- **Zero overhead when preventAdditions is empty** - Loop is skipped
- **O(n) pattern checks** - n = number of patterns in preventAdditions
- **Pattern matching cost** - Same as uneditableFiles (glob library)

**Optimization:**
- Early exit on first match (no need to check all patterns)
- Only runs for Write tool (most hooks are not Write)
- Reuses existing `matches_uneditable_pattern()` function (no new code)

**Benchmark expectations:**
- Negligible impact: <1ms per hook invocation
- Only affects Write tool operations
- Most projects have <10 patterns

### Migration & Backward Compatibility

**No breaking changes:**
- Field already exists in schema
- Default value is empty array (no blocking)
- Users who don't use preventAdditions see no behavior change
- Users who configured preventAdditions will now see it work (improvement, not break)

**Migration story:**
- "If you previously configured preventAdditions and it didn't work, it will now work after this fix"
- No config changes needed
- No action required from users

### Testing Strategy Summary

**Phase 1: TDD Tests**
1. Write all tests first [YES]
2. Run tests - all fail [YES]
3. Document test failures [YES]

**Phase 2: Implementation**
1. Add preventAdditions check [YES]
2. Run tests - all pass [YES]

**Phase 3: Validation**
1. Manual testing with real config [YES]
2. Integration tests [YES]
3. Regression tests (existing tests still pass) [YES]

**Success Criteria:**
- All new tests pass
- All existing tests pass
- Manual validation confirms blocking behavior
- Error messages are clear and helpful

## Alternative Approaches Considered

### Alternative 1: Make preventAdditions work for all tools
**Rejected because:**
- Semantic confusion with uneditableFiles
- Would duplicate functionality
- Users expect "Additions" to mean "creation", not "modification"

### Alternative 2: Remove preventAdditions field entirely
**Rejected because:**
- Breaking change for users who configured it
- Field serves a useful purpose (distinct from uneditableFiles)
- Better to fix than remove

### Alternative 3: Merge preventAdditions into uneditableFiles
**Rejected because:**
- Different semantics (creation vs modification)
- Breaking change
- Consolidation proposal (separate change) already addresses this

## Implementation Checklist

- [ ] Phase 1: Create TDD test suite (24 tests)
- [ ] Verify all tests fail before implementation
- [ ] Phase 2: Implement preventAdditions check in hooks.rs
- [ ] Verify all tests pass after implementation
- [ ] Phase 3: Manual validation with real config
- [ ] Run full regression test suite
- [ ] Phase 4: Update README.md documentation
- [ ] Update CHANGELOG.md with bug fix entry
- [ ] Review default-config.yaml examples
- [ ] Phase 5: Create spec deltas in OpenSpec format
- [ ] Run `openspec validate fix-prevent-additions-hook --strict`

## References

- **Code locations:**
  - Config definition: `src/config.rs:84-91` (PreToolUseConfig struct)
  - Hook handler: `src/hooks.rs:211-274` (handle_pre_tool_use)
  - Validation logic: `src/hooks.rs:281-341` (check_file_validation_rules)
  - Pattern matching: `src/hooks.rs:409-421` (matches_uneditable_pattern)

- **Related features:**
  - preventRootAdditions: `src/hooks.rs:298-314`
  - uneditableFiles: `src/hooks.rs:317-338`
  - preventGeneratedFileEdits: `src/hooks.rs:1002-1048`

- **Test files:**
  - Config tests: `tests/config_tests.rs`
  - Hook tests: `tests/hooks_tests.rs`
  - Integration tests: `tests/integration_tests.rs`

- **Documentation:**
  - README: `README.md` (preventAdditions examples)
  - Default config: `src/default-config.yaml:72-81`
  - Schema: `conclaude-schema.json:52-55`
