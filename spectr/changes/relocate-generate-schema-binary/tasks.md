## 1. Implementation
- [ ] 1.1 Move `scripts/generate-schema.rs` to `src/bin/generate-schema.rs`
- [ ] 1.2 Update `Cargo.toml:27` [[bin]] path from `scripts/generate-schema.rs` to `src/bin/generate-schema.rs`
- [ ] 1.3 Update doc comment in `src/lib.rs:12` to reference `src/bin/generate-schema.rs`

## 2. Verification
- [ ] 2.1 Verify `cargo build --bin generate-schema` succeeds with no errors
- [ ] 2.2 Verify `cargo run --bin generate-schema` outputs "Schema generated successfully: conclaude-schema.json"
- [ ] 2.3 Verify `conclaude-schema.json` is created in workspace root
