# Refine preventRootAdditions to Allow Root File Edits

## Why

The `preventRootAdditions` rule currently blocks ALL modifications to files at the repository root, treating both file **creation** and file **modification** equally. However, users need the ability to:

1. **Block creation** of new files at the root (prevents accidental file generation)
2. **Allow editing** of existing files at the root (enables necessary updates to root-level configuration files like `.env`, `package.json`, `tsconfig.json`, etc.)

**Current behavior:**
- User sets `rules.preventRootAdditions: true`
- Claude attempts to edit existing `package.json` at root level via `Edit` tool
- **BUG**: Operation is blocked because current check treats edits same as additions
- Root-level files become completely read-only

**Expected behavior:**
- User sets `rules.preventRootAdditions: true`
- Claude attempts to **create** new root-level file `README.md` via `Write` tool
- Operation is **blocked** [YES]
- Claude attempts to **edit** existing `package.json` via `Edit` tool
- Operation is **allowed** [YES] (file already exists, just being modified)

## What

Refine the `preventRootAdditions` enforcement logic to:

1. **Only block Write operations** when targeting root-level files (file creation)
2. **Allow Edit operations** to root-level files that already exist (file modification)
3. **Allow Write operations** to files that already exist at root (overwriting is modification, not addition)
4. **Maintain backwards compatibility** - Same configuration, refined semantics

## How

### Implementation Changes

**File:** `src/hooks.rs`
**Function:** `check_file_validation_rules()` (lines 281-341)
**Current location:** `preventRootAdditions` check at lines 298-314

#### Current Logic (Blocks All Modifications)
```rust
// Current: blocks both creation AND edit of root files
if config.rules.prevent_root_additions
    && payload.tool_name == "Write"
    && is_root_addition(&file_path, &relative_path, config_path)
{
    // Blocked!
}
```

#### Refined Logic (Only Blocks Creation)
```rust
// Refined: only block Write when file doesn't exist at root
if config.rules.prevent_root_additions
    && payload.tool_name == "Write"
    && is_root_addition(&file_path, &relative_path, config_path)
    && !file_exists(&resolved_path)  // NEW: Allow if file exists (modification, not addition)
{
    // Only blocked for NEW files at root
}
```

### Key Changes
1. **Add file existence check** - Use `Path::exists()` before blocking
2. **Preserve prevention of additions** - Only new files at root are blocked
3. **Enable root-level edits** - Existing root files can be modified
4. **Maintain semantics** - "preventRootAdditions" still means "prevent adding files to root"

## Success Criteria

- New files created at root are blocked
- Existing root files can be edited
- Existing root files can be overwritten (Write tool)
- Error messages remain clear
- All existing tests pass
- Backwards compatible - no config changes needed

## Non-Goals

- Changing the `preventRootAdditions` configuration field
- Affecting Edit or NotebookEdit tools (they remain unaffected)
- Modifying `preventAdditions` or other file protection rules
- Adding new configuration options

## Risks & Mitigation

**Risk:** Breaking users who rely on current "block all root modifications" behavior
**Mitigation:** Semantically, "preventRootAdditions" means "prevent additions", not "prevent all root modifications"

**Risk:** Unintended file overwrites at root
**Mitigation:** Users can combine with `uneditableFiles` rule to block specific root files if needed

## Related Changes

- **fix-prevent-additions-hook:** Implements `preventAdditions` field enforcement (separate from root-level prevention)
- **consolidate-rules-config:** Future consolidation of file protection rules

---

**This proposal refines semantics while maintaining intuitive user expectations.**
