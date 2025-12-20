## 1. Configuration

- [ ] 1.1 Add `prevent_root_additions_message: Option<String>` field to `PreToolUseConfig` in `src/config.rs`
- [ ] 1.2 Add serde rename to `preventRootAdditionsMessage`
- [ ] 1.3 Add documentation with `{file_path}` and `{tool}` template variable examples
- [ ] 1.4 Update Default impl to set `prevent_root_additions_message: None`
- [ ] 1.5 Add to help text field list in `src/config.rs`

## 2. Hook Implementation

- [ ] 2.1 Update `check_file_validation_rules` in `src/hooks.rs` to use custom message when configured
- [ ] 2.2 Support `{file_path}` and `{tool}` template variable substitution

## 3. Tests and Validation

- [ ] 3.1 Add `preventRootAdditionsMessage` to expected field list in `src/config_test.rs`
- [ ] 3.2 Run `cargo test` to verify all tests pass
- [ ] 3.3 Run `cargo clippy` to verify no warnings

## 4. Documentation

- [ ] 4.1 Add `preventRootAdditionsMessage` to `src/default-config.yaml`
- [ ] 4.2 Add documentation section to `docs/src/content/docs/reference/config/pre-tool-use.md`
- [ ] 4.3 Regenerate JSON schema via `cargo run -- schema`
