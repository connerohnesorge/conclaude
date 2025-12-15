# Change: Relocate generate-schema binary to src/bin/

## Why
The `generate-schema` binary is currently located at `scripts/generate-schema.rs`, while the `generate-docs` binary is at `src/bin/generate-docs.rs`. This inconsistency is confusing - both are Cargo-managed auxiliary binaries serving similar documentation/schema generation purposes, but they live in different locations.

**Note:** The previous design (archived change `2025-11-13-remove-generate-schema-subcommand`) chose `scripts/` to distinguish "build tooling" from "user-facing binaries." However, `generate-docs` was subsequently added to `src/bin/`, establishing a new precedent. This change aligns `generate-schema` with that precedent for consistency.

## What Changes
- Move `scripts/generate-schema.rs` to `src/bin/generate-schema.rs`
- Update `Cargo.toml` to reflect the new binary path
- Update documentation comment in `src/lib.rs:12` that references the old path
- Note: `scripts/` directory remains (contains `download-ts-sdk.sh`)

## Impact
- Affected specs: `workspace-structure` (adds new requirement for binary location)
- Affected code:
  - `scripts/generate-schema.rs` → `src/bin/generate-schema.rs` (move)
  - `Cargo.toml:27` (update [[bin]] path)
  - `src/lib.rs:12` (update doc comment path reference)
- No behavioral changes; this is purely a file organization refactor
- CI/CD workflows unaffected (they use `cargo build --bin generate-schema`, not source paths)
