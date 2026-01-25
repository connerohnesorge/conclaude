# Proposal: Slash Command and Skill Hooks Configuration

**Change ID:** `add-slash-command-skill-hooks`
**Status:** Proposal
**Author:** Claude Code
**Date:** 2026-01-25

## Executive Summary

Add conclaude-specific configuration for detecting and handling Claude Code slash commands (`/command`) and skills within `.conclaude.yaml`. This enables users to:
- Run custom commands when specific slash commands are invoked
- Inject context when skills are activated
- Block or allow slash commands/skills based on configurable rules
- Integrate slash command usage with existing workflow policies

## Why

Claude Code provides slash commands (user-invoked via `/command` syntax) and skills (auto-activated based on task context). Currently, conclaude has no mechanism to:

1. **Detect slash command invocation** - When a user types `/commit` or `/review-pr`, conclaude cannot intercept or augment this
2. **React to skill activation** - When Claude loads a skill like `coder` or `tester`, conclaude cannot run validation or inject context
3. **Enforce policies on commands/skills** - Teams cannot block certain commands or require approval for specific skills

The Claude Agent SDK's `UserPromptSubmit` hook receives the user's prompt text, which includes the `/command` syntax when a slash command is invoked. By parsing this in conclaude, we can provide slash command and skill-specific hooks.

**Reference:** Based on Claude Agent SDK types from `@anthropic-ai/claude-agent-sdk`:
- `UserPromptSubmitHookInput.prompt` contains the user's input text (including `/command`)
- `SDKSystemMessage` contains `slash_commands: string[]` and `skills: string[]` arrays
- `SubagentStartHookInput.agent_type` indicates which agent/skill is starting

## What Changes

- Add `slashCommands` section to `UserPromptSubmitConfig` for slash command detection
- Add `skillStart` section to config for skill activation hooks (detected via `SubagentStart`)
- Support pattern matching on command/skill names with glob patterns
- Provide context injection, command execution, and blocking capabilities
- Add environment variables exposing command/skill metadata

**Configuration additions:**
```yaml
userPromptSubmit:
  # NEW: Slash command detection within UserPromptSubmit
  slashCommands:
    commands:
      # Run for specific slash commands
      "/commit":
        - run: ".claude/scripts/pre-commit-check.sh"
          showStdout: true
      "/deploy":
        - run: ".claude/scripts/require-approval.sh"
          message: "Deployment requires team lead approval"
      # Glob pattern matching
      "/test*":
        - run: ".claude/scripts/test-setup.sh"

# NEW: Skill activation hooks (via SubagentStart detection)
skillStart:
  commands:
    # Run when specific skills start
    "coder":
      - run: ".claude/scripts/coder-init.sh"
    "tester":
      - run: ".claude/scripts/test-env-check.sh"
    # Wildcard for all skills
    "*":
      - run: ".claude/scripts/log-skill.sh"
        showCommand: false
```

## Impact

- Affected specs: Modify `prompt-context-injection` capability spec, add to `subagent-hooks` spec
- Affected code:
  - `src/config.rs` - Add `SlashCommandConfig`, `SkillStartConfig` structs
  - `src/hooks.rs` - Parse `/command` patterns from prompts, handle SubagentStart skill detection
  - `src/types.rs` - Add payload types for slash command/skill metadata
  - `conclaude-schema.json` - Extend schema with new configuration sections
  - Documentation updates for new configuration options

## Scope

### What's Included

**Slash Command Detection (via UserPromptSubmit):**
- Parse prompts for `/command` patterns at prompt submission time
- Match command names against configured patterns (exact, glob)
- Run configured commands with access to command name and arguments
- Support blocking, context injection, and command execution

**Environment Variables for Slash Commands:**
- `CONCLAUDE_SLASH_COMMAND` - The detected command name (e.g., "commit", "deploy")
- `CONCLAUDE_SLASH_COMMAND_ARGS` - Arguments passed to the command
- `CONCLAUDE_USER_PROMPT` - Full prompt text (existing)

**Skill Start Detection (via SubagentStart):**
- React to SubagentStart events where `agent_type` matches configured skills
- Run configured commands when skills activate
- Support pattern matching on skill names

**Environment Variables for Skills:**
- `CONCLAUDE_SKILL_NAME` - The skill/agent name being started
- `CONCLAUDE_AGENT_ID` - The subagent's unique ID
- Existing subagent environment variables

**Command Options (consistent with existing hooks):**
- `run`: (required) Command to execute
- `message`: (optional) Custom error message when command fails
- `showCommand`: (optional) Show command being executed. Default: true
- `showStdout`: (optional) Show stdout. Default: false
- `showStderr`: (optional) Show stderr. Default: false
- `maxOutputLines`: (optional) Limit output lines. Range: 1-10000
- `timeout`: (optional) Command timeout in seconds. Range: 1-3600

### What's NOT Included

- New Claude Code hook events (we work with existing UserPromptSubmit and SubagentStart)
- Modifying Claude Code's slash command behavior directly
- SkillStop hooks (use existing SubagentStop for this)
- Automatic skill/command discovery (static configuration only)

## Example Use Cases

### Pre-Commit Validation

```yaml
userPromptSubmit:
  slashCommands:
    commands:
      "/commit":
        - run: ".claude/scripts/validate-staged.sh"
          showStdout: true
          message: "Commit blocked: validation failed"
```

### Deployment Approval Gate

```yaml
userPromptSubmit:
  slashCommands:
    commands:
      "/deploy":
        - run: ".claude/scripts/check-deploy-approval.sh"
          message: "Deployment requires approval from #releases channel"
```

### Skill-Specific Context Loading

```yaml
skillStart:
  commands:
    "coder":
      - run: "echo 'Remember to follow coding standards in CLAUDE.md'"
        showStdout: true
    "tester":
      - run: ".claude/scripts/setup-test-env.sh"
```

### Audit Logging

```yaml
userPromptSubmit:
  slashCommands:
    commands:
      "*":
        - run: ".claude/scripts/log-command.sh"
          showCommand: false

skillStart:
  commands:
    "*":
      - run: ".claude/scripts/log-skill-activation.sh"
        showCommand: false
```

## Questions & Decisions

### Q: How should slash commands be detected from prompt text?

**Decision:** Parse the prompt for patterns matching `^/\w+` at the start or after newlines. Extract the command name and any trailing arguments. This matches Claude Code's slash command syntax.

### Q: Should slash command hooks be able to block the command?

**Decision:** Yes, using exit code 2 (same as other hooks). This allows teams to enforce policies like requiring approval for certain commands.

### Q: Where does skillStart fit in the hook lifecycle?

**Decision:** `skillStart` hooks process `SubagentStart` events when `agent_type` matches a configured skill pattern. This is distinct from `subagentStop` which handles agent completion.

### Q: Should we support both exact match and glob patterns?

**Decision:** Yes. Exact match for specific commands (`"/commit"`), glob patterns for families (`"/test*"`), and wildcard (`"*"`) for catch-all rules. Same pattern matching as `subagentStop`.

## Success Criteria

1. **Slash command detection works** - `/commit` triggers configured hooks
2. **Skill start detection works** - SubagentStart events trigger skill hooks
3. **Pattern matching works** - Glob patterns match correctly
4. **Environment variables set** - All CONCLAUDE_* variables available
5. **Blocking behavior works** - Exit code 2 blocks command/skill execution
6. **Backward compatible** - Existing configs work unchanged
7. **Documentation complete** - Examples in default config and docs
8. **Tests passing** - Unit tests for detection logic

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Prompt parsing false positives | Medium | Low | Strict regex: only `/command` at line start |
| SubagentStart doesn't include skill name | High | Low | Verify SDK types; agent_type field confirmed |
| Performance impact on every prompt | Low | Low | Regex is fast; skip if no slashCommands config |
| Confusion with existing hooks | Low | Medium | Clear documentation; distinct config sections |

## Migration Path

**For existing users:**
- No changes required
- Existing configuration works unchanged
- New sections are optional

**For users wanting slash command hooks:**
1. Add `slashCommands` section under `userPromptSubmit`
2. Configure commands for specific slash commands
3. Create scripts that use `CONCLAUDE_SLASH_COMMAND` env var

**For users wanting skill hooks:**
1. Add `skillStart` section at root level
2. Configure commands for specific skills
3. Create scripts that use `CONCLAUDE_SKILL_NAME` env var

**Breaking changes:**
- None - this is a purely additive feature

## Alternatives Considered

### Alternative 1: Wait for Claude Code to add dedicated hooks
**Rejected:** Claude Code's hook system is well-defined and stable. Adding detection within existing hooks is cleaner than waiting for new events.

### Alternative 2: Parse all prompts for any pattern
**Rejected:** Too broad. Slash commands have a specific `/command` syntax that is easy to detect reliably.

### Alternative 3: Use contextRules for slash commands
**Rejected:** contextRules are for context injection only; they can't execute commands or block prompts. Separate configuration is cleaner.

## Related Work

- **UserPromptSubmit** - Existing hook where slash command detection occurs
- **SubagentStart/SubagentStop** - Existing hooks for agent lifecycle
- **contextRules** - Pattern-based context injection (different purpose)
- **Claude Agent SDK** - TypeScript types define the payload structures

## Implementation Notes

**Key files to modify:**
- `src/config.rs` - Add `SlashCommandConfig`, `SkillStartConfig` structs
- `src/hooks.rs` - Add slash command detection in `handle_user_prompt_submit()`, skill detection in subagent handling
- `src/types.rs` - Add metadata types
- `src/default-config.yaml` - Add example configuration
- Tests for new functionality

**Reuse patterns from:**
- `SubagentStopConfig` for pattern-based command maps
- `UserPromptSubmitCommand` for command structure
- Existing glob pattern matching utilities
