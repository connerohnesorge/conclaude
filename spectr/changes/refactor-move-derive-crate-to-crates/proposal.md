# Change: Move conclaude-field-derive to crates/ directory

## Why
The `conclaude-field-derive` proc-macro crate currently lives at the project root, cluttering it alongside the main crate. Moving it to a `crates/` directory follows the idiomatic Rust convention for multi-crate workspaces, improving project organization and scalability for future internal crates.

## What Changes
- Create `crates/` directory at project root
- Move `conclaude-field-derive/` to `crates/conclaude-field-derive/`
- Update workspace members in root `Cargo.toml` from `[".", "conclaude-field-derive"]` to `[".", "crates/conclaude-field-derive"]`
- Update path dependency in root `Cargo.toml` from `path = "conclaude-field-derive"` to `path = "crates/conclaude-field-derive"`

## Impact
- Affected specs: None (no behavioral changes)
- Affected code: `Cargo.toml` (workspace and dependency path updates)
- Build/CI: No changes required (paths are relative and will resolve correctly)
- Breaking changes: None (internal restructuring only)
