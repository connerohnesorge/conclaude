# Change: Add Agent-Aware File Protection

## Why

Orchestrator workflows delegate tasks to specialized subagents (coder, tester, stuck) that operate in separate contexts. Currently, `uneditableFiles` rules apply uniformly to all agents, but some files should only be protected from specific agents. For example:

- Prevent "coder" subagent from updating task completion status (tasks.jsonc) while allowing the orchestrator to do so
- Allow "tester" subagent to read but not modify test fixtures that only "coder" should touch
- Protect conclaude config from all agents universally

This enables fine-grained access control in multi-agent orchestration patterns.

## What Changes

- **MODIFIED**: Add optional `agent` field to `uneditableFiles` detailed format
- **ADDED**: Glob pattern matching for agent names (supports `*`, exact match, and patterns like `code*`)
- **ADDED**: Transcript parsing during preToolUse to detect current agent context
- **Backward compatible**: Omitting `agent` defaults to `*` (all agents), preserving existing behavior

### Configuration Format

```yaml
preToolUse:
  uneditableFiles:
    # Simple format (unchanged) - applies to all agents
    - ".conclaude.yml"

    # Detailed format with agent field
    - pattern: ".conclaude.yml"
      message: "Do not modify conclaude config"
      agent: "*"  # All agents (default)

    - pattern: "spectr/changes/**/tasks.jsonc"
      message: "Task completion should be reviewed by orchestrator"
      agent: "coder"  # Only blocks coder subagent

    - pattern: "src/**/*.test.ts"
      message: "Test files managed by tester agent"
      agent: "code*"  # Matches coder, coder-v2, etc.
```

## Impact

- **Affected specs**: preToolUse
- **Affected code**:
  - `src/config.rs` - UnEditableFileRule struct extension
  - `src/hooks.rs` - Agent detection and matching logic
  - `schemas/conclaude.schema.json` - Schema update
  - `src/default-config.yaml` - Documentation examples
- **Breaking changes**: No (fully backward compatible)
- **Dependencies**: None new (reuses existing glob crate)
- **Performance**: Minimal - adds one glob match per rule when agent field present
