# Tasks: Slash Commands and Skills Hooks Support

## 1. CLI Flag Implementation

- [ ] 1.1 Add `--skill` flag to `PreToolUse` hook command
- [ ] 1.2 Add `--skill` flag to `PostToolUse` hook command
- [ ] 1.3 Add `--skill` flag to `Stop` hook command
- [ ] 1.4 Add `--skill` flag to `SessionStart` hook command
- [ ] 1.5 Add `--skill` flag to `SessionEnd` hook command
- [ ] 1.6 Add `--skill` flag to `UserPromptSubmit` hook command
- [ ] 1.7 Add `--skill` flag to `Notification` hook command
- [ ] 1.8 Add `--skill` flag to `SubagentStart` hook command
- [ ] 1.9 Add `--skill` flag to `SubagentStop` hook command
- [ ] 1.10 Add `--skill` flag to `PreCompact` hook command
- [ ] 1.11 Add `--skill` flag to `PermissionRequest` hook command
- [ ] 1.12 Add `CONCLAUDE_SKILL` environment variable handling (similar to `CONCLAUDE_AGENT`)

## 2. Hook Injection for Commands and Skills

- [ ] 2.1 Create `discover_command_files()` function (similar to `discover_agent_files()`)
- [ ] 2.2 Create `discover_skill_files()` function (similar to `discover_agent_files()`)
- [ ] 2.3 Create `generate_skill_hooks()` function (similar to `generate_agent_hooks()`)
- [ ] 2.4 Create `inject_skill_hooks_into_file()` function (similar to `inject_agent_hooks_into_file()`)
- [ ] 2.5 Create `inject_skill_hooks()` function for batch injection
- [ ] 2.6 Integrate command/skill hook injection into `handle_init()`
- [ ] 2.7 Add user feedback messages for command/skill discovery
- [ ] 2.8 Support `--force` flag for re-injection

## 3. Configuration Schema Updates

- [ ] 3.1 Add `skill` field to `UnEditableFileRule` type
- [ ] 3.2 Add `skill` field to `ToolUsageRule` type
- [ ] 3.3 Add `skill` field to `StopCommand` type
- [ ] 3.4 Add `skill` field to `SubagentStopCommand` type (if applicable)
- [ ] 3.5 Add `matches_skill_pattern()` helper function (similar to `matches_agent_pattern()`)
- [ ] 3.6 Update field name lists in error suggestion system

## 4. Hook Processing Updates

- [ ] 4.1 Read `CONCLAUDE_SKILL` environment variable in hook handlers
- [ ] 4.2 Add skill context logging
- [ ] 4.3 Update rule matching to check skill patterns
- [ ] 4.4 Add skill information to notification messages
- [ ] 4.5 Update PreToolUse handler to check skill-specific rules
- [ ] 4.6 Update Stop handler to execute skill-specific commands

## 5. Default Configuration Updates

- [ ] 5.1 Add commented examples for skill-specific rules in `src/default-config.yaml`
- [ ] 5.2 Document `CONCLAUDE_SKILL` environment variable
- [ ] 5.3 Add skill pattern matching examples

## 6. Schema Generation

- [ ] 6.1 Update `src/schema.rs` to include skill fields
- [ ] 6.2 Run schema generation to update `conclaude-schema.json`

## 7. Documentation

- [ ] 7.1 Update README.md with slash command/skill hook information
- [ ] 7.2 Document `--skill` flag usage
- [ ] 7.3 Add examples of skill-specific configuration
- [ ] 7.4 Document `CONCLAUDE_SKILL` environment variable

## 8. Testing

- [ ] 8.1 Add unit tests for `discover_command_files()`
- [ ] 8.2 Add unit tests for `discover_skill_files()`
- [ ] 8.3 Add unit tests for `generate_skill_hooks()`
- [ ] 8.4 Add unit tests for `matches_skill_pattern()`
- [ ] 8.5 Add unit tests for skill field in configuration parsing
- [ ] 8.6 Add integration test for command file hook injection
- [ ] 8.7 Add integration test for skill file hook injection
- [ ] 8.8 Add tests for skill-specific rule matching
- [ ] 8.9 Verify backward compatibility with existing agent hooks

## 9. Finalization

- [ ] 9.1 Run `cargo clippy` and fix warnings
- [ ] 9.2 Run `cargo fmt` to format code
- [ ] 9.3 Run `cargo test` to verify all tests pass
- [ ] 9.4 Test `conclaude init` with sample command/skill files
- [ ] 9.5 Validate schema with example configurations
- [ ] 9.6 Run `spectr validate add-slash-command-skill-hooks`
