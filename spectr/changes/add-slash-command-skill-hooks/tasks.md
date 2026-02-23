# Tasks: Slash Command and Skill Hooks Configuration

## 1. Configuration Structs

- [x] 1.1 Add `SlashCommandConfig` struct to `src/config.rs` with pattern-to-commands mapping
- [x] 1.2 Add `SkillStartConfig` struct to `src/config.rs` with skill pattern-to-commands mapping
- [x] 1.3 Add `SlashCommandEntry` struct with run, message, showCommand, showStdout, showStderr, maxOutputLines, timeout fields
- [x] 1.4 Add `SkillStartCommand` struct (same fields as SlashCommandEntry)
- [x] 1.5 Extend `UserPromptSubmitConfig` to include optional `slashCommands` field
- [x] 1.6 Add `skillStart` field to `ConclaudeConfig` at root level
- [x] 1.7 Add serde derive macros and JsonSchema for new structs

## 2. Slash Command Detection Logic

- [x] 2.1 Create `detect_slash_command()` function to parse `/command` from prompt text
- [x] 2.2 Implement regex pattern: `/^\/(\w+)(?:\s+(.*))?$/m` to extract command and args
- [x] 2.3 Add unit tests for slash command detection edge cases
- [x] 2.4 Integrate slash command detection into `handle_user_prompt_submit()` flow
- [x] 2.5 Match detected command against configured patterns using glob matching

## 3. Skill Start Detection Logic

- [x] 3.1 Extend `SubagentStartPayload` handling to check for skill patterns
- [x] 3.2 Add pattern matching against `agent_type` field in SubagentStart events
- [x] 3.3 Create `handle_skill_start()` function for skill-specific hooks
- [x] 3.4 Integrate with existing subagent hook processing flow

## 4. Command Execution

- [x] 4.1 Add `CONCLAUDE_SLASH_COMMAND` environment variable support
- [x] 4.2 Add `CONCLAUDE_SLASH_COMMAND_ARGS` environment variable support
- [x] 4.3 Add `CONCLAUDE_SKILL_NAME` environment variable support
- [x] 4.4 Reuse existing command execution infrastructure from `execute_stop_commands()`
- [x] 4.5 Support exit code 2 for blocking behavior

## 5. Schema and Validation

- [x] 5.1 Update JSON schema generator to include new config sections
- [x] 5.2 Run `/generate-schema` to regenerate `conclaude-schema.json`
- [x] 5.3 Add validation for pattern syntax (valid glob patterns)
- [x] 5.4 Add validation for command field requirements

## 6. Configuration Defaults and Documentation

- [x] 6.1 Add commented examples to `src/default-config.yaml`
- [x] 6.2 Run `/generate-docs` to regenerate configuration reference docs
- [x] 6.3 Update docs with slash command and skill hook examples

## 7. Testing

- [x] 7.1 Add unit tests for `SlashCommandConfig` parsing
- [x] 7.2 Add unit tests for `SkillStartConfig` parsing
- [x] 7.3 Add unit tests for slash command detection from prompts
- [x] 7.4 Add unit tests for pattern matching (exact, glob, wildcard)
- [x] 7.5 Add integration tests for end-to-end slash command hooks
- [x] 7.6 Add integration tests for skill start hooks

## 8. Finalization

- [x] 8.1 Run `cargo clippy` and fix any warnings
- [x] 8.2 Run `cargo fmt` to format code
- [x] 8.3 Run `cargo test` to verify all tests pass
- [x] 8.4 Verify schema validation with example configs
