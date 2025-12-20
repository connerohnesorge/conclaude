# Change: Remove Dead Code Technical Debt

## Why

The codebase contains unused dependencies, dead code behind `#[allow(dead_code)]` annotations, and code that was marked dead but is actually used. This cleanup:
1. Removes 3 unused external dependencies (faster builds, smaller audit surface)
2. Removes truly dead code that was never called
3. Removes unnecessary `#[allow(dead_code)]` annotations from code that IS used
4. Isolates test-only code with `#[cfg(test)]`

## What Changes

### Unused Dependencies (Cargo.toml)
- **REMOVE**: `config = "0.14"` - External crate never imported (shadowed by local `config` module)
- **REMOVE**: `thiserror = "1.0"` - Error derive macro never used (project uses `anyhow`)
- **REMOVE**: `chrono = { version = "0.4", features = ["serde"] }` - Date/time crate never imported

### Dead Types (src/types.rs)
- **REMOVE**: `HookPayload` enum (lines 200-227) - Union type defined but never instantiated
- **REMOVE**: `impl HookPayload` block (lines 229-283) - Methods `session_id()`, `transcript_path()`, `hook_event_name()` never called

### Unnecessary Annotations (src/types.rs)
- **REMOVE**: `#[allow(dead_code)]` from `validate_permission_request_payload()` - function IS used in hooks.rs:392
- **REMOVE**: `#[allow(dead_code)]` from `validate_subagent_start_payload()` - function IS used in hooks.rs:1236
- **REMOVE**: `#[allow(dead_code)]` from `validate_subagent_stop_payload()` - function IS used in hooks.rs:1780

### Unnecessary Annotations (src/schema.rs)
- **REMOVE**: `#[allow(dead_code)]` from `generate_config_schema()` - function IS used by generate-schema binary
- **REMOVE**: `#[allow(dead_code)]` from `write_schema_to_file()` - function IS used by generate-schema binary
- **MODIFY**: `validate_config_against_schema()` - change from `#[allow(dead_code)]` to `#[cfg(test)]` (only used in tests)

### Struct Field Cleanup (src/bin/generate-docs.rs)
- **CLEAN UP**: `Schema.title`, `Schema.description`, `PropertyDefinition.name` fields - either remove if not needed for JSON parsing, or accept dead code annotations as necessary for serde deserialization

## Impact

- **Affected specs**: Adds new code hygiene requirements to `spectr/specs/tooling/spec.md`
- **Affected code**:
  - `Cargo.toml` - 3 dependency removals
  - `src/types.rs` - ~90 lines removed, 3 annotations removed
  - `src/schema.rs` - 2 annotations removed, 1 changed to `#[cfg(test)]`
  - `src/bin/generate-docs.rs` - 3 annotations reviewed
- **Build impact**: Reduced compile time and binary size
- **Risk**: Low - all removed code is verifiably unused (no callers found via grep/clippy)
