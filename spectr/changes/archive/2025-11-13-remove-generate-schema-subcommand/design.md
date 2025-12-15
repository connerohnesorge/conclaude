# Design: Remove generate-schema Subcommand

## Context

The current implementation includes `generate-schema` as a subcommand in the main `conclaude` CLI. This command generates a JSON Schema file for the `.conclaude.yaml` configuration format. While useful during development and releases, it doesn't belong in the user-facing CLI for several reasons:

1. **User Confusion**: End users don't need to generate schemas; they consume them via IDE tooling
2. **Build Complexity**: Schema generation requires the `schemars` dependency which increases binary size
3. **Release Coupling**: Schema updates should be automatic during releases, not manual user actions
4. **Maintenance Burden**: The command requires ongoing documentation and support despite limited utility

## Solution Architecture

### External Script Approach

Move schema generation to a standalone Rust binary (`scripts/generate-schema.rs`) that:
- Lives outside the main CLI codebase
- Can be compiled and run as part of the build/release process
- Reuses the existing `schema` module functions
- Has no impact on the main binary size or dependencies

### Build-Time Integration

The schema generation script will be integrated into the release process:
1. The script will generate `conclaude-schema.json` in the workspace root
2. `cargo-dist` will automatically include the schema file in release artifacts
3. The schema URL (`https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-schema.json`) will remain stable
4. **Note**: The GitHub Actions workflow is managed by `cargo-dist` and should not be manually edited

### Dependency Management

The `schemars` dependency is already required for the `ConclaudeConfig` derive macros, so:
- The schema module (`src/schema.rs`) remains in the library
- The external script depends on `conclaude` as a library dependency
- No changes to dependencies in `Cargo.toml` are needed
- Binary size is unaffected since schema generation was already compiled in

## Technical Decisions

### Why a Rust Script vs Shell Script?

**Decision**: Use a Rust binary in `scripts/` directory

**Rationale**:
- Type safety: Reuse existing schema generation functions directly
- Cross-platform: Works on Windows, macOS, and Linux without bash
- Maintainability: Same language as the main codebase
- CI simplicity: No additional tooling needed beyond cargo

**Alternatives Considered**:
- Shell script calling `conclaude generate-schema`: Requires keeping the subcommand
- Python/Node script: Adds new language dependencies to the build process
- Build.rs script: Schema generation is release-time, not compile-time

### Why Not Keep the Subcommand?

**Decision**: Completely remove `generate-schema` from the CLI

**Rationale**:
- **Principle of Least Surprise**: Users don't expect schema generation in a hook handler CLI
- **Maintenance**: Every subcommand requires documentation, testing, and support
- **Clarity**: Release automation should not rely on user-facing commands

**Alternatives Considered**:
- Mark as hidden command: Still increases binary complexity
- Document as "internal only": Confusing for users who discover it
- Keep for backward compatibility: Unnecessary since usage is minimal

### Script Location

**Decision**: Place in `scripts/generate-schema.rs` (not `bin/`)

**Rationale**:
- `bin/` typically contains user-facing binaries that get installed
- `scripts/` clearly indicates build/development tooling
- Follows common Rust project conventions (see cargo itself)

**Implementation**:
```rust
// scripts/generate-schema.rs
fn main() -> anyhow::Result<()> {
    let schema = conclaude::schema::generate_config_schema()?;
    let output_path = PathBuf::from("conclaude-schema.json");
    conclaude::schema::write_schema_to_file(&schema, &output_path)?;
    println!("✅ Schema generated: {}", output_path.display());
    Ok(())
}
```

### GitHub Actions Integration

**Decision**: Create a separate workflow for schema upload

**Implementation**:
Create `.github/workflows/upload-schema.yml` that:
- Triggers on `release: types: [published]` (runs AFTER cargo-dist creates the release)
- Checks out the repository
- Installs Rust toolchain using `dtolnay/rust-toolchain@stable`
- Builds the schema generator: `cargo build --release --bin generate-schema`
- Runs the generator: `./target/release/generate-schema`
- Derives the release tag from the GitHub event context: `TAG="${{ github.event.release.tag_name }}"` (or `TAG="$GITHUB_REF_NAME"`)
- Uploads to release: `gh release upload $TAG conclaude-schema.json --clobber`

**Rationale**:
- Completely separate from cargo-dist's managed workflow (no conflicts)
- Runs as a post-release step, adding schema as an additional asset
- Clean separation of concerns: cargo-dist handles binaries, separate workflow handles schema
- Can be tested independently and won't break if cargo-dist updates its workflow
- Follows GitHub Actions best practices for adding assets to existing releases

## Migration Strategy

### For End Users

No action required:
- The schema URL remains the same
- IDE autocomplete continues to work
- No workflows depend on `conclaude generate-schema`

### For Contributors

Update local workflows:
- Instead of `conclaude generate-schema`, use `cargo run --bin generate-schema`
- Schema validation during development: `cargo test` (existing tests validate schema)
- Pre-commit: No changes needed (schema is not committed)

### For CI/CD

- The `generate-schema` script is available for local use: `cargo run --bin generate-schema`
- New workflow `.github/workflows/upload-schema.yml` handles schema upload to releases
- Workflow runs automatically after cargo-dist creates a release
- No changes needed to cargo-dist's managed `release.yml` workflow

## Testing Strategy

1. **Unit Tests**: Existing tests in `src/schema.rs` remain unchanged
2. **Integration Test**: Add test to verify the external script works:
   ```rust
   #[test]
   fn test_generate_schema_script() {
       let output = std::process::Command::new("cargo")
           .args(&["run", "--bin", "generate-schema"])
           .output()
           .expect("Failed to run script");
       assert!(output.status.success());
       assert!(PathBuf::from("conclaude-schema.json").exists());
   }
   ```
3. **Release Workflow Test**: Verify schema is included in release assets
4. **Documentation Audit**: Ensure all references to `generate-schema` are updated

## Rollback Plan

If issues arise after deployment:

1. **Immediate**: The schema file is already in the release, users are unaffected
2. **Short-term**: Revert the PR that removed the subcommand
3. **Long-term**: If the external script approach doesn't work, we can:
   - Restore the subcommand as hidden
   - Keep both approaches temporarily during transition

## Open Questions

None - this is a straightforward refactoring with minimal risk.
