# Change: Add Agent Field to ToolUsageValidation

## Why

The `uneditableFiles` configuration already supports agent-scoped rules via an optional `agent` field, allowing different file protection policies for different subagents (e.g., "coder" vs "tester"). However, `toolUsageValidation` rules do not have this capability, meaning tool usage restrictions apply uniformly to all agents.

Adding an `agent` field to `ToolUsageRule` enables fine-grained control over which agents can use specific tools with specific patterns—for example, allowing the orchestrator to run all Bash commands while restricting subagents to a narrower set.

## What Changes

- Add optional `agent` field to `ToolUsageRule` struct in `src/config.rs`
- Update `check_tool_usage_rules()` in `src/hooks.rs` to detect current agent context and evaluate agent patterns
- Update JSON schema (auto-generated from Rust structs)
- Add documentation examples showing agent-scoped tool usage rules

## Impact

- Affected specs: `preToolUse`
- Affected code:
  - `src/config.rs:148-169` (`ToolUsageRule` struct)
  - `src/hooks.rs:1891-1961` (`check_tool_usage_rules` function)
  - `conclaude-schema.json` (auto-generated)
- No breaking changes: the field is optional and defaults to `"*"` (match all agents)
