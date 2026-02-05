# Tasks: Slash Commands and Skills Hooks Support

## 1. CLI Flag Implementation

- [x] 1.1 Add `--skill` flag to `PreToolUse` hook command
- [x] 1.2 Add `--skill` flag to `PostToolUse` hook command
- [x] 1.3 Add `--skill` flag to `Stop` hook command
- [x] 1.4 Add `--skill` flag to `SessionStart` hook command
- [x] 1.5 Add `--skill` flag to `SessionEnd` hook command
- [x] 1.6 Add `--skill` flag to `UserPromptSubmit` hook command
- [x] 1.7 Add `--skill` flag to `Notification` hook command
- [x] 1.8 Add `--skill` flag to `SubagentStart` hook command
- [x] 1.9 Add `--skill` flag to `SubagentStop` hook command
- [x] 1.10 Add `--skill` flag to `PreCompact` hook command
- [x] 1.11 Add `--skill` flag to `PermissionRequest` hook command
- [x] 1.12 Add `CONCLAUDE_SKILL` environment variable handling (similar to `CONCLAUDE_AGENT`)

## 2. Hook Injection for Commands and Skills

- [x] 2.1 Create `discover_command_files()` function (similar to `discover_agent_files()`)
- [x] 2.2 Create `discover_skill_files()` function (similar to `discover_agent_files()`)
- [x] 2.3 Create `generate_skill_hooks()` function (similar to `generate_agent_hooks()`)
- [x] 2.4 Create `inject_skill_hooks_into_file()` function (similar to `inject_agent_hooks_into_file()`)
- [x] 2.5 Create `inject_skill_hooks()` function for batch injection
- [x] 2.6 Integrate command/skill hook injection into `handle_init()`
- [x] 2.7 Add user feedback messages for command/skill discovery
- [x] 2.8 Support `--force` flag for re-injection

## 3. Configuration Schema Updates

- [x] 3.1 Add `skill` field to `UnEditableFileRule` type
- [x] 3.2 Add `skill` field to `ToolUsageRule` type
- [x] 3.3 Add `skill` field to `StopCommand` type
- [x] 3.4 Add `skill` field to `SubagentStopCommand` type (if applicable)
- [x] 3.5 Add `matches_skill_pattern()` helper function (similar to `matches_agent_pattern()`)
- [x] 3.6 Update field name lists in error suggestion system

## 4. Hook Processing Updates

- [x] 4.1 Read `CONCLAUDE_SKILL` environment variable in hook handlers
- [x] 4.2 Add skill context logging
- [x] 4.3 Update rule matching to check skill patterns
- [x] 4.4 Add skill information to notification messages
- [x] 4.5 Update PreToolUse handler to check skill-specific rules
- [x] 4.6 Update Stop handler to execute skill-specific commands

## 5. Default Configuration Updates

- [x] 5.1 Add commented examples for skill-specific rules in `src/default-config.yaml`
- [x] 5.2 Document `CONCLAUDE_SKILL` environment variable
- [x] 5.3 Add skill pattern matching examples

## 6. Schema Generation

- [x] 6.1 Update `src/schema.rs` to include skill fields
- [x] 6.2 Run schema generation to update `conclaude-schema.json`

## 7. Documentation

- [x] 7.1 Update README.md with slash command/skill hook information
- [x] 7.2 Document `--skill` flag usage
- [x] 7.3 Add examples of skill-specific configuration
- [x] 7.4 Document `CONCLAUDE_SKILL` environment variable

## 8. Testing

- [x] 8.1 Add unit tests for `discover_command_files()`
- [x] 8.2 Add unit tests for `discover_skill_files()`
- [x] 8.3 Add unit tests for `generate_skill_hooks()`
- [x] 8.4 Add unit tests for `matches_skill_pattern()`
- [x] 8.5 Add unit tests for skill field in configuration parsing
- [x] 8.6 Add integration test for command file hook injection
- [x] 8.7 Add integration test for skill file hook injection
- [x] 8.8 Add tests for skill-specific rule matching
- [x] 8.9 Verify backward compatibility with existing agent hooks

## 9. Finalization

- [x] 9.1 Run `cargo clippy` and fix warnings
- [x] 9.2 Run `cargo fmt` to format code
- [x] 9.3 Run `cargo test` to verify all tests pass
- [x] 9.4 Test `conclaude init` with sample command/skill files
- [x] 9.5 Validate schema with example configurations
- [x] 9.6 Run `spectr validate add-slash-command-skill-hooks`
