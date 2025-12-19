## 1. Implementation

- [ ] 1.1 Add `agent` field to `ToolUsageRule` struct in `src/config.rs`
  - Add `#[serde(default)]` for optional field
  - Add doc comment describing agent pattern matching
- [ ] 1.2 Update `check_tool_usage_rules()` in `src/hooks.rs` to:
  - Read current agent context via `read_agent_from_session_file()`
  - Check agent pattern match before applying rule using `matches_agent_pattern()`
  - Skip rules that don't match current agent
  - Include agent context in error messages when agent-specific rule triggers
- [ ] 1.3 Regenerate JSON schema by running `cargo run -- generate-schema`

## 2. Testing

- [ ] 2.1 Add unit tests in `src/config_test.rs` for `ToolUsageRule` deserialization with agent field
- [ ] 2.2 Add integration tests for agent-scoped tool usage validation rules

## 3. Documentation

- [ ] 3.1 Update `.conclaude.yaml` example configuration with agent-scoped toolUsageValidation examples
- [ ] 3.2 Update `src/default-config.yaml` with commented example of agent field usage
