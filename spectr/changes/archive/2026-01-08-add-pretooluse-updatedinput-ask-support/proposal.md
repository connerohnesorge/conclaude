# Proposal: Add updatedInput Support for PreToolUse Hooks with Ask Decision

## Why

Claude Code recently added support for `updatedInput` when returning an `ask` permission decision in PreToolUse hooks. This enables hooks to act as middleware that can modify tool inputs while still requesting user consent before execution. Conclaude must support this pattern to maintain compatibility with the Claude Code hook protocol.

## Problem

Currently, conclaude's `HookResult` struct only supports:
- `message`: Optional custom message
- `blocked`: Boolean to allow/block the operation
- `system_prompt`: Optional context injection

Claude Code now supports returning both `updatedInput` (modified tool parameters) and `permissionDecision: "ask"` together in `hookSpecificOutput`. This allows hooks to:
1. Sanitize or transform tool inputs
2. Still require user approval before the modified operation proceeds

Without this support, conclaude cannot act as middleware for PreToolUse hooks while preserving user consent.

## Context

Claude Code's PreToolUse hooks now support three permission decisions:
- **`allow`**: Bypasses permission system, proceeds with (optionally modified) input
- **`deny`**: Blocks the tool call with a reason
- **`ask`**: Prompts user to confirm, showing `permissionDecisionReason` (NEW: now supports `updatedInput`)

The fix in Claude Code enables `updatedInput` to work with `ask`, allowing hooks to function as middleware while maintaining the user consent flow.

## Proposed Solution

Extend conclaude's `HookResult` struct to include:
1. **`updated_input`**: Optional `HashMap<String, serde_json::Value>` for modified tool parameters
2. **`decision`**: Optional string for explicit permission decisions (`"allow"`, `"deny"`, `"ask"`)

This allows hooks to return:
```json
{
  "decision": "ask",
  "message": "Modified command requires approval",
  "updated_input": {
    "command": "sanitized-command"
  }
}
```

## Impact

- **Affected specs**: preToolUse, hooks-system
- **Affected code**:
  - `src/types.rs` - Extend HookResult struct
  - `src/hooks.rs` - Handle updated_input in PreToolUse response
- **Configuration**: No config schema changes needed
- **Breaking changes**: None - new optional fields maintain backward compatibility

## Dependencies

None - this is a self-contained extension to the hook response protocol.

## Alternatives Considered

1. **Separate response type for PreToolUse** - Rejected: adds unnecessary complexity when optional fields suffice
2. **Only support updatedInput with allow** - Rejected: doesn't align with Claude Code's capabilities
3. **Wrap in hookSpecificOutput structure** - Considered: could add for full Claude Code alignment, but flat structure is simpler for now

## Success Criteria

- [ ] `updated_input` field added to HookResult as optional HashMap
- [ ] `decision` field added to HookResult as optional String
- [ ] PreToolUse handler respects `decision: "ask"` with `updated_input`
- [ ] Tests verify serialization/deserialization of new fields
- [ ] `spectr validate` passes
