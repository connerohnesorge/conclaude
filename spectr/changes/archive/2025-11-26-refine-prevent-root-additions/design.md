# Design Document: Refine preventRootAdditions to Allow Root File Edits

## Problem Analysis

### Current State

The `preventRootAdditions` rule currently treats file **creation** and file **modification** identically, blocking all Write tool operations on root-level files:

```yaml
# User configuration
rules:
  preventRootAdditions: true
```

**What happens today:**

| Operation | Path | Tool | Result | Issue |
|-----------|------|------|--------|-------|
| Create | `.env` | Write | BLOCKED | Correct behavior |
| Create | `package.json` | Write | BLOCKED | Correct behavior |
| **Edit** | **existing `.env`** | **Write** | **BLOCKED** | **WRONG - file exists, just editing** |
| **Edit** | **existing `tsconfig.json`** | **Edit** | **BLOCKED** | **WRONG - Edit tool on existing file** |

The `is_root_addition()` function (lines 372-402) doesn't check if the file already exists—it only checks if the parent directory is the root. This makes root-level files completely read-only.

### Root Cause

The `is_root_addition()` function lacks a file existence check:

```rust
pub fn is_root_addition(_file_path: &str, relative_path: &str, config_path: &Path) -> bool {
    // ... path resolution logic ...
    // Block if the file is being created in the same directory as the config
    config_dir_canonical == file_dir_canonical
    // Missing: check if file already exists
}
```

### User Impact

**Pain Point:** Root-level configuration files become impossible to update:
- `.env` - Cannot add or update environment variables
- `package.json` - Cannot update dependencies, scripts
- `tsconfig.json` - Cannot update TypeScript configuration
- `.eslintrc` - Cannot update linting rules

Users either:
1. Disable `preventRootAdditions` entirely (loses protection)
2. Manually edit files outside Claude (poor experience)
3. Work around by creating subdirectories (messy)

## Solution Design

### High-Level Approach

**Semantic Refinement:** `preventRootAdditions` means "prevent **additions** of files to root", not "make root read-only".

**Implementation:**
1. Keep the existing `is_root_addition()` check for blocking creation
2. Add a file existence check to allow modifications
3. Only block Write when: `root_file && !exists(file)`
4. Preserve Edit/NotebookEdit tool behavior (no change)

### Architecture

```
┌─────────────────────────────────────────┐
│ PreToolUse Hook Triggered               │
│ Tool: Write, Edit, NotebookEdit         │
│ File: /path/to/file                     │
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
│ (Write tool only, root-level, new files)│
└─────────────┬───────────────────────────┘
              │
              ├─ Tool != Write? ──> Continue (Edit/NotebookEdit unaffected)
              │
              ├─ Not at root? ──> Continue
              │
              ├─ File exists? ──> Continue (modification OK)
              │
              └─ New file at root? ──> BLOCK!
              │
              ▼
┌─────────────────────────────────────────┐
│ Check: uneditableFiles                  │
│ (All file-modifying tools, all files)   │
└─────────────┬───────────────────────────┘
              ...
```

### Implementation Location

**File:** `src/hooks.rs`
**Function:** `check_file_validation_rules()` (lines 281-341)
**Current code:** Lines 298-314 (preventRootAdditions check)

#### Option A: Modify check_file_validation_rules()
Add existence check before blocking:

```rust
// Check preventRootAdditions rule - only applies to Write tool for NEW files
if config.rules.prevent_root_additions
    && payload.tool_name == "Write"
    && is_root_addition(&file_path, &relative_path, config_path)
    && !resolved_path.exists()  // NEW: Allow modifications to existing files
{
    let error_message = format!(
        "Blocked {} operation: preventRootAdditions rule prevents creating files at repository root. File: {}",
        payload.tool_name, file_path
    );

    eprintln!(
        "PreToolUse blocked by preventRootAdditions rule: tool_name={}, file_path={}",
        payload.tool_name, file_path
    );

    return Ok(Some(HookResult::blocked(error_message)));
}
```

**Rationale:** Minimal change, preserves existing structure

#### Option B: Extract to separate function
Create `is_new_root_addition()` for clarity:

```rust
pub fn is_new_root_addition(
    file_path: &str,
    relative_path: &str,
    resolved_path: &Path,
    config_path: &Path,
) -> bool {
    is_root_addition(file_path, relative_path, config_path) && !resolved_path.exists()
}
```

**Rationale:** More readable, facilitates testing

### Behavioral Specification

#### Write Tool Behavior

| Scenario | File Exists | Root Level | Result | Error |
|----------|-------------|-----------|--------|-------|
| New file | No | No | ALLOWED | None |
| New file | No | Yes | BLOCKED | Root addition blocked |
| Update file | Yes | No | ALLOWED | None |
| **Update file** | **Yes** | **Yes** | **ALLOWED** | **None** |
| Overwrite | Yes | No | ALLOWED | None |
| **Overwrite** | **Yes** | **Yes** | **ALLOWED** | **None** |

#### Edit/NotebookEdit Tool Behavior

(Unchanged - these tools don't trigger preventRootAdditions check)

| Scenario | File Exists | Root Level | Result |
|----------|-------------|-----------|--------|
| Edit file | Yes | Yes | ALLOWED |
| Edit file | Yes | No | ALLOWED |

### Examples

#### Example 1: Block creation, allow edit
```yaml
rules:
  preventRootAdditions: true
```

```bash
# Attempt 1: Create new root file
tool_input: { file_path: "README.md" }
tool_name: "Write"
# Result: BLOCKED - new file at root

# Attempt 2: Edit existing root file
tool_input: { file_path: "package.json" }
tool_name: "Edit"
# Result: ALLOWED - Edit tool not checked by preventRootAdditions

# Attempt 3: Update root file via Write (overwrite)
tool_input: { file_path: "package.json" }
tool_name: "Write"
# Result: ALLOWED - file exists (modification, not addition)
```

#### Example 2: Prevent creation, allow configuration updates
```yaml
rules:
  preventRootAdditions: true
  uneditableFiles:
    - "src/types.ts"  # Can still edit, not at root
```

```bash
# Create new root file
file_path: ".env"
tool: Write
# Result: BLOCKED - preventRootAdditions

# Update root .env
file_path: ".env"
tool: Write (file exists)
# Result: ALLOWED - file exists

# Update .env via Edit
file_path: ".env"
tool: Edit
# Result: ALLOWED - Edit not blocked by preventRootAdditions
```

### Error Messages

**Format unchanged:** Same error message as before
```
Blocked Write operation: preventRootAdditions rule prevents creating files at repository root. File: README.md
```

The error message is still accurate—it specifically says "prevents creating files" (additions), which is exactly what the refined logic does.

## Test Strategy

### Unit Tests

**Test file:** `tests/hooks_tests.rs` (extend existing tests)

#### Test Cases for Refined preventRootAdditions

1. **Block new root files**
   - Create `README.md` (no prior file) → BLOCKED
   - Create `.env` (no prior file) → BLOCKED

2. **Allow updates to existing root files**
   - Update `package.json` (exists) with Write tool → ALLOWED
   - Update `tsconfig.json` (exists) with Write tool → ALLOWED
   - Update `package.json` (exists) with Edit tool → ALLOWED

3. **Allow Write overwrites**
   - Overwrite `config.json` (exists) with Write tool → ALLOWED

4. **Maintain behavior for non-root files**
   - Create `src/main.rs` (new, non-root) → ALLOWED
   - Create `src/lib/utils.js` (new, non-root) → ALLOWED

5. **Interaction with other rules**
   - preventRootAdditions + uneditableFiles → both enforced
   - preventRootAdditions + preventAdditions → both enforced

### Regression Tests

Ensure existing tests still pass:
- Root file creation blocked
- Non-root files unaffected
- Error messages correct

## Implementation Plan

### Phase 1: Add Tests First (TDD)
1. Write tests for refined behavior (they will fail initially)
2. Run: `cargo test hooks_tests`
3. Confirm all new tests fail

### Phase 2: Implement Fix
1. Add existence check to `check_file_validation_rules()`
2. Run: `cargo test hooks_tests`
3. Confirm all tests pass

### Phase 3: Validation
1. Manual testing with real config
2. Run full test suite
3. Verify error messages unchanged

## Edge Cases

### Case 1: Parent directories created
```bash
# File: "new_dir/new_file.txt"
# Parent dir doesn't exist, file doesn't exist
# Parent is not root-level
# Result: ALLOWED (not at root)
```

### Case 2: Symlinks to root files
```bash
# File: "link_to_root" -> points to root file
# Depends on how resolved_path handles symlinks
# Should: Use canonical path (symlink resolution)
# Result: Check against actual location
```

### Case 3: Case-sensitive file system
```bash
# File: "Package.json" vs "package.json"
# Linux: Different files (case-sensitive)
# macOS/Windows: Same file (case-insensitive)
# Should: Use `Path::exists()` (OS-aware)
```

### Case 4: File permissions
```bash
# File: "package.json" exists but no write permission
# Check: File exists? Yes
# Prevent: No (preventRootAdditions allows it)
# Later check: OS will fail on actual Write
```

## Backwards Compatibility

**No breaking changes:**
- Configuration stays the same
- `rules.preventRootAdditions: true` works as expected
- More permissive (allows edits), not more restrictive
- Users who didn't rely on "block edits" see immediate benefit
- Users who somehow needed "block edits" can use `uneditableFiles` rule

**Migration:** None needed

## Performance Impact

**Negligible:**
- One additional `Path::exists()` check per preventRootAdditions evaluation
- Only evaluated for Write tool operations on root-level files
- File system operations are cheap (cached by OS)
- Most codebases don't have preventRootAdditions enabled

## Related Specs

- **fix-prevent-additions-hook:** Implements separate `preventAdditions` field (pattern-based)
- **consolidate-rules-config:** Future consolidation of file protection rules

---

**This refinement improves usability while maintaining strong protection against accidental root-level file creation.**
