# Design: Agent-Aware File Protection

## Context

conclaude enforces file protection rules via the `preToolUse` hook, blocking file operations that match `uneditableFiles` patterns. In orchestration workflows, a main Claude session delegates tasks to specialized subagents (coder, tester, stuck). Currently, protection rules apply uniformly regardless of which agent is operating.

**Problem**: Operators need differentiated access control. For example:
- Orchestrator should update task status, but coder should not
- Coder should create implementation files, but not modify test fixtures
- All agents should be blocked from modifying conclaude config

## Goals / Non-Goals

**Goals:**
- Enable per-agent file protection rules in `uneditableFiles` config
- Support glob patterns for flexible agent matching
- Maintain full backward compatibility with existing configurations
- Minimal performance overhead

**Non-Goals:**
- Complex permission systems (RBAC, hierarchical permissions)
- Real-time agent permission changes during session
- Agent-aware rules for other protection mechanisms (preventRootAdditions, preventAdditions)

## Decisions

### Decision 1: Agent Identification Strategy

**Decision**: Use transcript parsing to extract agent type from Task tool invocation.

**Context**: During a preToolUse hook invocation within a subagent, we need to know which agent is executing. The transcript contains the original Task tool call with `subagent_type` parameter.

**Alternatives Considered**:
1. **Environment variable injection** - Claude Code could pass agent context via env vars
   - Pro: Simple, reliable
   - Con: Requires Claude Code changes, not currently available
2. **Transcript parsing** (chosen)
   - Pro: Works with existing Claude Code, data already available
   - Con: Slightly more complex, depends on transcript format
3. **Session ID pattern matching** - Derive agent from session ID format
   - Pro: Very simple
   - Con: Session IDs don't encode agent type

**Rationale**: Transcript parsing is already implemented for SubagentStop hook (`extract_agent_name_from_transcript()`). Reusing this approach maintains consistency and avoids external dependencies.

### Decision 2: Main Session Agent Name

**Decision**: Use literal string `"main"` for the orchestrator/main session.

**Rationale**:
- Intuitive naming that matches the orchestration mental model
- Allows explicit targeting of main session in rules
- Distinguishes from subagents clearly

### Decision 3: Agent Pattern Matching

**Decision**: Use glob patterns for agent matching (same library as file patterns).

**Matching Rules**:
- `"*"` - Matches all agents (wildcard)
- `"coder"` - Exact match for coder subagent
- `"code*"` - Glob pattern matching coder, coder-v2, etc.
- `"main"` - Matches main/orchestrator session only

**Rationale**:
- Consistent with existing pattern matching (uneditableFiles uses glob)
- Familiar syntax for users already using glob patterns
- Covers common use cases (exact, wildcard, prefix matching)
- No new dependencies (uses existing `glob` crate)

### Decision 4: Default Agent Value

**Decision**: When `agent` field is omitted, default to `"*"` (all agents).

**Rationale**:
- Backward compatible - existing configs work unchanged
- Secure default - rules apply universally unless explicitly scoped
- Follows principle of least surprise

### Decision 5: Error Behavior

**Decision**: Include agent context in error messages when agent-specific rules trigger blocks.

**Format**:
```
Blocked Edit operation: file matches preToolUse.uneditableFiles pattern 'tasks.jsonc' (agent: coder). File: spectr/changes/foo/tasks.jsonc
```

**Rationale**: Clear debugging - operator knows why rule matched and can adjust agent scoping.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Transcript format changes | Agent detection breaks | Version-check transcript format, log warnings on parse failures |
| Glob pattern ambiguity | Unexpected matches | Document pattern semantics clearly, prefer exact matches in examples |
| Performance overhead | Slower hook execution | Parse transcript once per invocation, cache result |
| Main session detection | Incorrect agent identification | Clear heuristic: if not in subagent transcript, it's "main" |

## Migration Plan

**No migration required** - change is fully backward compatible:
1. Existing `uneditableFiles` entries without `agent` field default to `"*"`
2. New agent field is optional
3. No config format changes, only additions

## Open Questions

1. **Should we support negative patterns?** (e.g., `agent: "!coder"` meaning "all except coder")
   - Initial answer: No, keep it simple. Use explicit positive patterns.

2. **Should agent matching be case-sensitive?**
   - Decision: Yes, case-sensitive (consistent with existing pattern matching)

## Testing Strategy

**Unit Tests**:
- `matches_agent_pattern()` with edge cases (empty string, special chars, unicode)
- `detect_current_agent()` with mocked transcripts
- Config parsing with various agent field values

**Integration Tests**:
- Full preToolUse flow with agent-specific rules
- Cross-agent scenarios (main blocks coder, coder doesn't block main)
- Glob pattern scenarios

**Manual Testing**:
- Real Claude Code session with orchestrator + subagents
- Verify blocking behavior matches expectations
