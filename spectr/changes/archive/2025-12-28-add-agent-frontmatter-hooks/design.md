## Context

Claude Code introduced agent frontmatter hooks (per Boris Cherny's tweet). This allows hooks to be defined directly in `.claude/agents/*.md` files:

```yaml
---
name: my-custom-agent
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "echo 'About to run Bash'"
  Stop:
    - hooks:
        - type: command
          command: "echo 'Agent finished'"
---
```

This is superior to the current approach because:
1. No transcript parsing needed - agent name is known at hook definition time
2. No session files needed - hooks are scoped to the agent natively
3. Simpler configuration - hooks live with the agent definition

## Goals / Non-Goals

**Goals:**
- Inject hooks into agent frontmatter via `conclaude init`
- Add `--agent` flag to pass agent name to hook handlers
- Remove brittle transcript parsing and session file code

**Non-Goals:**
- Backward compatibility with old SubagentStart/SubagentStop system
- Support for agents without frontmatter hooks

## Decisions

### Decision 1: CLI Flag Format
Use `--agent <name>` flag on existing hook commands rather than new subcommands.

**Rationale:** Minimal CLI changes, consistent with existing pattern.

**Example:** `conclaude Hooks PreToolUse --agent coder`

### Decision 2: Agent Frontmatter Structure
Follow Claude Code's native hook format exactly.

**Structure:**
```yaml
hooks:
  <HookType>:
    - matcher: ""  # or specific tool matcher
      hooks:
        - type: command
          command: "conclaude Hooks <HookType> --agent <name>"
```

### Decision 3: All Hook Types
Inject all hook types, not just Stop. This provides full observability.

**Hook Types:** PreToolUse, PostToolUse, Stop, SessionStart, SessionEnd, Notification, PreCompact, PermissionRequest, UserPromptSubmit

### Decision 4: Remove Old System
Fully remove SubagentStart/SubagentStop from settings.json generation.

**Rationale:** Clean break, no confusion about which system is active.

## Risks / Trade-offs

- **Risk**: Users with existing SubagentStart/SubagentStop configs
  - **Mitigation**: Document migration, those hooks will simply be unused

- **Risk**: Agent files without `name` field
  - **Mitigation**: Derive name from filename, log warning

## Migration Plan

1. Run `conclaude init` to update agent files
2. Old SubagentStart/SubagentStop hooks in settings.json are ignored
3. Agent frontmatter hooks take over

## Open Questions

None - user clarified all decisions.
