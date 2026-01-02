# Proposal: PostToolUse Hook Commands

**Change ID:** `add-post-tool-use-commands`
**Status:** Proposal
**Author:** Claude Code
**Date:** 2026-01-02

## Executive Summary

Add configurable command execution to the `postToolUse` hook, enabling read-only observation of tool results for logging, documentation, and integration purposes.

**Key benefits:**
- Log Q&A interactions from AskUserQuestion for documentation
- Capture search results for later reference
- Audit tool usage patterns across sessions
- Build custom integrations that react to tool outputs
- Support Spectr workflow automation (logging proposals, tracking changes)

## Why

Currently conclaude supports hooks for `stop`, `subagentStop`, `preToolUse`, `permissionRequest`, and `userPromptSubmit`. However, there's no way to hook into tool use results after they complete. The `PostToolUsePayload` type exists but `handle_post_tool_use()` doesn't execute any user-configured commands.

This prevents use cases like:
- Logging Q&A interactions from AskUserQuestion for documentation
- Capturing search results for later reference
- Auditing tool usage patterns
- Building custom integrations that react to tool outputs

A `postToolUse` hook with command support would enable read-only observation of tool results, allowing users to log, document, or integrate with external systems.

## What Changes

- Add `postToolUse` configuration section to `.conclaude.yaml` schema
- Define `PostToolUseConfig` struct with command configuration and tool filtering
- Specify environment variable interface for passing tool data to hooks:
  - `CONCLAUDE_TOOL_NAME` - name of the tool that was executed
  - `CONCLAUDE_TOOL_INPUT` - JSON string of tool input parameters
  - `CONCLAUDE_TOOL_OUTPUT` - JSON string of tool response/result
  - `CONCLAUDE_TOOL_TIMESTAMP` - ISO 8601 timestamp of completion
  - `CONCLAUDE_TOOL_USE_ID` - unique identifier for correlation with preToolUse
- Implement command execution in `handle_post_tool_use()` with tool filtering
- Document read-only semantics (hooks observe but cannot modify or block)

## Impact

- Affected specs: New `post-tool-use` capability spec
- Affected code:
  - `src/config.rs` - Add `PostToolUseConfig` and `PostToolUseCommand` structs
  - `src/hooks.rs` - Implement command execution in `handle_post_tool_use()`
  - `src/default-config.yaml` - Add commented example configuration
  - `src/schema.rs` - Schema auto-generates from config structs

## Scope

### What's Included

**Configuration Structure:**
```yaml
postToolUse:
  commands:
    # Run for all tools
    - run: ".claude/scripts/log-tool.sh"

    # Run only for specific tools (glob patterns supported)
    - tool: "AskUserQuestion"
      run: ".claude/scripts/log-qa.sh"

    # Run for multiple tool patterns
    - tool: "*Search*"
      run: ".claude/scripts/log-search.sh"
```

**Command Options (same as stop hook commands):**
- `run`: (required) Command to execute
- `tool`: (optional) Glob pattern to filter which tools trigger this command. Default: `"*"` (all tools)
- `showStdout`: (optional) Show stdout. Default: false
- `showStderr`: (optional) Show stderr. Default: false
- `showCommand`: (optional) Show command being executed. Default: true
- `maxOutputLines`: (optional) Limit output lines. Range: 1-10000
- `timeout`: (optional) Command timeout in seconds. Range: 1-3600

**Environment Variables:**
- `CONCLAUDE_TOOL_NAME` - The tool that was executed (e.g., "AskUserQuestion", "Bash")
- `CONCLAUDE_TOOL_INPUT` - JSON string of input parameters
- `CONCLAUDE_TOOL_OUTPUT` - JSON string of tool response
- `CONCLAUDE_TOOL_TIMESTAMP` - ISO 8601 timestamp (e.g., "2026-01-02T12:34:56Z")
- `CONCLAUDE_TOOL_USE_ID` - Unique ID for PreToolUse/PostToolUse correlation
- `CONCLAUDE_SESSION_ID` - Current session ID
- `CONCLAUDE_CWD` - Current working directory
- `CONCLAUDE_CONFIG_DIR` - Directory containing .conclaude.yaml

**Read-Only Semantics:**
- PostToolUse hooks CANNOT block or modify tool execution (it already happened)
- Commands run asynchronously after tool completion
- Command failures are logged but do not affect the session
- Hooks observe results but cannot change them

### What's NOT Included

- Blocking tool execution (use preToolUse for that)
- Modifying tool results
- Retry logic for failed hooks
- Rate limiting for high-frequency tools
- Persistent storage (users implement their own logging)

## Example Use Cases

### Spectr Q&A Documentation

```yaml
postToolUse:
  commands:
    - tool: "AskUserQuestion"
      run: ".claude/scripts/log-qa.sh"
```

`.claude/scripts/log-qa.sh`:
```bash
#!/bin/bash
# Log Q&A interactions for proposal documentation
echo "[$CONCLAUDE_TOOL_TIMESTAMP] Q&A logged" >> .claude/logs/qa.jsonl
echo "{\"timestamp\": \"$CONCLAUDE_TOOL_TIMESTAMP\", \"input\": $CONCLAUDE_TOOL_INPUT, \"output\": $CONCLAUDE_TOOL_OUTPUT}" >> .claude/logs/qa.jsonl
```

### Search Result Caching

```yaml
postToolUse:
  commands:
    - tool: "*Search*"
      run: ".claude/scripts/cache-search.sh"
      showStdout: false
```

### Tool Usage Audit Trail

```yaml
postToolUse:
  commands:
    - run: ".claude/scripts/audit-tool.sh"
      showCommand: false
```

## Questions & Decisions

### Q: Should postToolUse commands be able to block?
**Decision:** No. PostToolUse is read-only observation. The tool has already executed, so blocking serves no purpose. This keeps the mental model simple and prevents confusion with preToolUse.

### Q: Should commands run synchronously or asynchronously?
**Decision:** Synchronously by default (like stop commands), but command failures do not block the session. This ensures logging completes before the next tool executes, maintaining order.

### Q: How should tool filtering work?
**Decision:** Use glob patterns for tool name matching, consistent with other pattern-based config (uneditableFiles, toolUsageValidation). The `tool` field defaults to `"*"` (all tools) if not specified.

### Q: What happens if CONCLAUDE_TOOL_OUTPUT is very large?
**Decision:** Pass the full JSON. Users can use `maxOutputLines` on their scripts or handle truncation themselves. Large outputs are uncommon and users need full data for accurate logging.

## Success Criteria

1. **Config validates** - Schema accepts `postToolUse` section with commands
2. **Tool filtering works** - Commands only run for matching tool patterns
3. **Environment variables set** - All CONCLAUDE_TOOL_* variables available
4. **Read-only behavior** - Hooks cannot block or modify results
5. **Error handling** - Failed commands logged but don't break session
6. **Backward compatible** - Existing hooks unaffected; feature is additive
7. **Documentation** - Default config has commented examples
8. **Tests passing** - Unit tests for tool pattern matching and command execution

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Large JSON payloads slow execution | Low | Low | Env var passing is efficient; users can truncate in scripts |
| Too many commands per tool call | Medium | Low | Recommend filtering by tool pattern to reduce command volume |
| Script errors cause noise | Low | Medium | Errors logged to stderr only; don't affect session |
| Confusion with preToolUse blocking | Low | Low | Clear documentation; different config sections |

## Migration Path

**For existing users:**
- No changes required
- PostToolUse continues working (no-op if no commands configured)
- Existing preToolUse, stop, subagentStop hooks unaffected

**For users wanting post-tool logging:**
- Add `postToolUse.commands` section to `.conclaude.yaml`
- Create logging scripts to process environment variables
- Test with specific tools first, then expand to `"*"` if needed

**Breaking changes:**
- None - this is a purely additive feature

## Alternatives Considered

### Alternative 1: Add logging to preToolUse
**Rejected:** preToolUse fires before execution, so tool_response is not available. PostToolUse is the correct lifecycle point for observing results.

### Alternative 2: Unified hook command system
**Rejected:** Each hook type has different semantics (blocking vs read-only, different payloads). Keeping them separate maintains clarity and matches Claude Code's hook event model.

### Alternative 3: Built-in logging without commands
**Rejected:** Commands provide flexibility for users to implement their own storage, formatting, and integration patterns. Built-in logging would be too opinionated.

## Related Work

- **PreToolUse hook** - Fires before tool execution, can block
- **PermissionRequest hook** - Tool permission decisions
- **SubagentStop hook** - Similar command execution pattern with filtering
- **Stop hook** - Command execution pattern reference

## Implementation Notes

**Key files to modify:**
- `src/config.rs` - Add `PostToolUseConfig`, `PostToolUseCommand` structs
- `src/hooks.rs` - Implement command execution in `handle_post_tool_use()`
- `src/default-config.yaml` - Add example configuration
- Tests in `tests/` for new functionality

**Reuse patterns from:**
- `StopCommandConfig` for command structure
- `match_subagent_patterns()` for tool pattern matching
- `execute_stop_commands()` for command execution logic
- `build_subagent_env_vars()` for environment variable construction
