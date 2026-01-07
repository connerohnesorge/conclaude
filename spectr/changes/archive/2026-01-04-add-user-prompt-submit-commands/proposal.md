# Proposal: UserPromptSubmit Hook Commands

**Change ID:** `add-user-prompt-submit-commands`
**Status:** Proposal
**Author:** Claude Code
**Date:** 2026-01-03

## Executive Summary

Add configurable command execution to the `userPromptSubmit` hook, enabling users to run custom commands when user input is submitted to Claude.

**Key benefits:**
- Run linters or validators on user prompts before processing
- Log user prompts for auditing or analytics
- Trigger external integrations (e.g., Slack notifications, webhooks)
- Inject additional context based on command output
- Support workflow automation (e.g., updating issue trackers)

## Why

Currently the `userPromptSubmit` hook supports only `contextRules` for pattern-based context injection. However, there's no way to execute arbitrary shell commands when a user submits input. This prevents use cases like:

- Running custom validation scripts on user prompts
- Logging prompts to external systems for auditing
- Triggering notifications or integrations when prompts match patterns
- Running commands that produce dynamic context to inject

Adding a `commands` array to `userPromptSubmit` would enable users to execute shell commands with access to the user's prompt via environment variables.

## What Changes

- Add `commands` array field to `UserPromptSubmitConfig` struct
- Define `UserPromptSubmitCommand` struct with pattern filtering and execution options
- Specify environment variable interface for passing prompt data to commands:
  - `CONCLAUDE_USER_PROMPT` - the user's input text
  - `CONCLAUDE_SESSION_ID` - current session ID
  - `CONCLAUDE_CWD` - current working directory
  - `CONCLAUDE_CONFIG_DIR` - directory containing .conclaude.yaml
- Implement command execution in `handle_user_prompt_submit()` with pattern filtering
- Document read-only semantics (commands cannot block user input submission)
- Preserve existing `contextRules` functionality (fully backward compatible)

## Impact

- Affected specs: Modify `prompt-context-injection` capability spec
- Affected code:
  - `src/config.rs` - Add `UserPromptSubmitCommand` struct, update `UserPromptSubmitConfig`
  - `src/hooks.rs` - Implement command execution in `handle_user_prompt_submit()`
  - `src/default-config.yaml` - Add commented example configuration
  - `docs/src/content/docs/reference/config/user-prompt-submit.md` - Document new commands feature

## Scope

### What's Included

**Configuration Structure:**
```yaml
userPromptSubmit:
  # Existing contextRules continue to work unchanged
  contextRules:
    - pattern: "sidebar"
      prompt: "Read @.claude/contexts/sidebar.md"

  # NEW: Command execution support
  commands:
    # Run for all prompts
    - run: ".claude/scripts/log-prompt.sh"

    # Run only for prompts matching pattern (regex)
    - pattern: "deploy|release"
      run: ".claude/scripts/notify-deploy.sh"

    # Run with output display options
    - pattern: "test"
      run: ".claude/scripts/pre-test-check.sh"
      showStdout: true
```

**Command Options (consistent with stop/subagentStop commands):**
- `run`: (required) Command to execute
- `pattern`: (optional) Regex pattern to filter which prompts trigger this command. Default: runs for all prompts
- `caseInsensitive`: (optional) Use case-insensitive pattern matching. Default: false
- `showCommand`: (optional) Show command being executed. Default: true
- `showStdout`: (optional) Show stdout. Default: false
- `showStderr`: (optional) Show stderr. Default: false
- `maxOutputLines`: (optional) Limit output lines. Range: 1-10000
- `timeout`: (optional) Command timeout in seconds. Range: 1-3600

**Environment Variables:**
- `CONCLAUDE_USER_PROMPT` - The user's input text
- `CONCLAUDE_SESSION_ID` - Current session ID
- `CONCLAUDE_CWD` - Current working directory
- `CONCLAUDE_CONFIG_DIR` - Directory containing .conclaude.yaml
- `CONCLAUDE_HOOK_EVENT` - Always "UserPromptSubmit"

**Read-Only Semantics:**
- UserPromptSubmit hook commands CANNOT block prompt submission
- Commands run synchronously but failures do not affect the session
- Command output is logged but does not modify the prompt
- This matches the pattern of PostToolUse hooks (observational, non-blocking)

### What's NOT Included

- Blocking prompt submission based on command exit status
- Modifying the user's prompt text
- Retry logic for failed commands
- Command output injection into context (use contextRules for that)

## Example Use Cases

### Prompt Logging for Auditing

```yaml
userPromptSubmit:
  commands:
    - run: ".claude/scripts/log-prompt.sh"
      showCommand: false
```

`.claude/scripts/log-prompt.sh`:
```bash
#!/bin/bash
# Append prompt to audit log
echo "[$(date -Iseconds)] $CONCLAUDE_USER_PROMPT" >> .claude/logs/prompts.log
```

### Deployment Notifications

```yaml
userPromptSubmit:
  commands:
    - pattern: "deploy|release|ship"
      run: ".claude/scripts/notify-slack.sh"
      showCommand: false
```

### Pre-Processing Validation

```yaml
userPromptSubmit:
  commands:
    - pattern: "(?i)sql|database"
      run: ".claude/scripts/check-db-access.sh"
      showStdout: true
```

### Combined with Context Injection

```yaml
userPromptSubmit:
  # Context injection rules (existing feature)
  contextRules:
    - pattern: "auth"
      prompt: "Review @.claude/contexts/auth.md"

  # Command execution (new feature)
  commands:
    - pattern: "auth"
      run: ".claude/scripts/log-security-prompt.sh"
```

## Questions & Decisions

### Q: Should commands be able to block prompt submission?
**Decision:** No. UserPromptSubmit hooks are read-only, matching the pattern of PostToolUse. Users can use PreToolUse for blocking tool execution. Blocking prompts would create a poor UX and potential for stuck sessions.

### Q: How should pattern matching work for commands?
**Decision:** Use regex patterns (same as contextRules) rather than glob patterns. This keeps the interface consistent within the same configuration section.

### Q: Should commands run before or after contextRules processing?
**Decision:** Commands run after contextRules are evaluated. This ensures context injection happens first, and commands can observe the final state.

### Q: Should command output be injectable as context?
**Decision:** No. Command output goes to stdout/stderr logging only. For dynamic context injection, users should update files that are referenced by `@file` syntax in contextRules. This keeps the two features cleanly separated.

## Success Criteria

1. **Config validates** - Schema accepts `userPromptSubmit.commands` section
2. **Pattern filtering works** - Commands only run for matching prompts
3. **Environment variables set** - All CONCLAUDE_* variables available
4. **Read-only behavior** - Commands cannot block prompt processing
5. **Error handling** - Failed commands logged but don't break session
6. **Backward compatible** - Existing contextRules unaffected
7. **Documentation** - Default config has commented examples
8. **Tests passing** - Unit tests for pattern matching and command execution

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Long prompts in env vars | Low | Low | Shell handles large env vars; users can truncate in scripts |
| Too many commands per prompt | Medium | Low | Recommend filtering by pattern to reduce command volume |
| Script errors cause noise | Low | Medium | Errors logged to stderr only; don't affect session |
| Confusion with contextRules | Low | Low | Clear documentation; different purposes (commands vs context) |

## Migration Path

**For existing users:**
- No changes required
- Existing `contextRules` configuration works unchanged
- New `commands` field defaults to empty array

**For users wanting command execution:**
- Add `commands` array to `userPromptSubmit` section
- Create scripts to process `CONCLAUDE_USER_PROMPT` environment variable
- Test with specific patterns first, then expand if needed

**Breaking changes:**
- None - this is a purely additive feature

## Alternatives Considered

### Alternative 1: Add command output as context source
**Rejected:** Mixing command output with context injection creates complexity. Better to keep contextRules for static/file-based context and commands for side effects.

### Alternative 2: Allow blocking on command failure
**Rejected:** Would create a poor user experience and potential for stuck sessions. Users should validate at the tool level (PreToolUse) rather than prompt level.

### Alternative 3: Use glob patterns instead of regex
**Rejected:** contextRules already use regex patterns. Keeping the same pattern syntax within the same config section is more consistent.

## Related Work

- **contextRules** - Existing pattern-based context injection in userPromptSubmit
- **PostToolUse hook** - Similar read-only command execution pattern
- **SubagentStop hook** - Pattern-based command filtering reference
- **Stop hook** - Command execution pattern reference

## Implementation Notes

**Key files to modify:**
- `src/config.rs` - Add `UserPromptSubmitCommand` struct, update `UserPromptSubmitConfig`
- `src/hooks.rs` - Implement command execution in `handle_user_prompt_submit()`
- `src/default-config.yaml` - Add example configuration
- Tests in `tests/` for new functionality

**Reuse patterns from:**
- `StopCommandConfig` for command structure
- `compile_rule_pattern()` for regex pattern compilation
- `execute_stop_commands()` for command execution logic (adapted for non-blocking)
- `build_subagent_env_vars()` for environment variable construction pattern
