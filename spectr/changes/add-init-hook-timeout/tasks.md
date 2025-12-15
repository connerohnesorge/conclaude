## 1. Implementation

- [ ] 1.1 Add optional `timeout` field to `ClaudeHookConfig` struct in `src/main.rs`
- [ ] 1.2 Update hook generation in `handle_init` to include `timeout: 600` for all hooks
- [ ] 1.3 Test `conclaude init` generates settings.json with timeout field

## 2. Validation

- [ ] 2.1 Run `cargo build` to verify compilation
- [ ] 2.2 Run `cargo test` to verify existing tests pass
- [ ] 2.3 Manually test `conclaude init` in a temp directory and verify generated settings.json includes timeout
