# Change: Slash Commands and Skills Hooks Support

## Why

Claude Code supports organizing reusable capabilities as Slash Commands (in `.claude/commands/*.md`) and Skills (in `.claude/skills/*.md`). These files have YAML frontmatter similar to agents. Currently, conclaude can inject hooks into agent files, but it lacks support for commands and skills files.

This change extends conclaude's hook injection capability to support:
1. **Slash Commands** - User-invoked commands (e.g., `/commit`, `/test`)
2. **Skills** - Auto-activated capabilities (e.g., testing skills, documentation skills)

By injecting conclaude hooks into these files, users can enforce guardrails and validation specific to particular commands or skills, enabling more granular workflow control.

## What Changes

### CLI Updates
- Add `--skill` flag to all hook commands (PreToolUse, PostToolUse, Stop, SessionStart, etc.)
- The `--skill` flag passes skill context to hook handlers via `CONCLAUDE_SKILL` environment variable

### Hook Injection During `conclaude init`
- Discover `.claude/commands/*.md` files and inject conclaude hooks
- Discover `.claude/skills/*.md` files and inject conclaude hooks
- Support `--force` flag to re-inject even if hooks exist
- Use `--skill {name}` pattern similar to `--agent {name}`

### Hook Processing
- Read skill context from `CONCLAUDE_SKILL` environment variable
- Apply skill-specific rules from `.conclaude.yaml` configuration
- Support skill pattern matching in rules (e.g., `skill: "test*"`)

### Configuration Schema
- Add `skill` field to applicable rule types (similar to existing `agent` field)
- Allow rules to target specific skills or skill patterns

## Configuration Example

```yaml
# .conclaude.yaml - Example skill-specific rules
preToolUse:
  uneditableFiles:
    - pattern: "tests/**"
      skill: "test*"  # Only applies to test-related skills
      message: "Test files managed by testing skills"
    
  toolUsageValidation:
    - tool: "Write"
      pattern: "src/**/*.rs"
      skill: "doc*"   # Only applies to documentation skills
      action: "block"
      message: "Documentation skills should not modify source code"

stop:
  commands:
    # Global commands run for all skills
    - run: "cargo check"
    
    # Skill-specific commands  
    - run: "cargo test --lib"
      skill: "tester"
      message: "Library tests must pass"
```

## Impact

- **Affected specs:** 
  - `initialization` - Extend init to handle commands/skills
  - `configuration` - Add skill field to rule types
  - `hooks-system` - Add skill context to hook processing
  - `cli-structure` - Add --skill flag

- **Affected code:**
  - `src/main.rs` - Add --skill flag, implement command/skill file discovery and injection
  - `src/hooks.rs` - Read CONCLAUDE_SKILL, apply skill-specific rules
  - `src/config.rs` - Add skill field to rule types
  - `src/types.rs` - Skill context types

- **Breaking changes:** None - additive only

## Migration Path

**For existing users:**
- No changes required
- Existing configuration works unchanged
- Run `conclaude init` again to inject hooks into command/skill files

**For users wanting skill-specific rules:**
1. Add `skill` field to existing rules where needed
2. Re-run `conclaude init` to update command/skill files with hooks
3. Use `CONCLAUDE_SKILL` environment variable in custom scripts

## Success Criteria

1. **Command file discovery** - `conclaude init` finds `.claude/commands/*.md` files
2. **Skill file discovery** - `conclaude init` finds `.claude/skills/*.md` files  
3. **Hook injection** - Hooks injected with `--skill {name}` pattern
4. **Flag support** - All hook commands accept `--skill` flag
5. **Environment variable** - `CONCLAUDE_SKILL` set in hook handlers
6. **Configuration** - Skill field works in rules (pattern matching)
7. **Backward compatible** - Existing agent hooks unaffected
8. **Tests passing** - Unit and integration tests for new functionality

## Alternatives Considered

### Alternative 1: Use --agent flag for skills too
**Rejected:** While technically possible, having separate `--skill` and `--agent` flags provides clearer semantics. Skills and agents serve different purposes in Claude Code, and users may want different rules for each.

### Alternative 2: Detect skill from subagent_type in payload
**Rejected:** This only works for subagents, not for slash commands which don't have a subagent payload. The environment variable approach is more flexible and works across all hook types.

### Alternative 3: Store skill context in session files
**Rejected:** Similar to how agents work, but adds unnecessary complexity. The environment variable set by the CLI flag is simpler and sufficient.

## Related Work

- **Agent hooks** - Existing pattern that this extends
- **SubagentStopConfig** - Pattern for pattern-matched commands (reused for skills)
- **CONCLAUDE_AGENT** environment variable - Similar pattern for `--skill`
