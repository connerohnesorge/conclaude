# Tasks: UserPromptSubmit Hook Commands

## 1. Configuration Types

- [x] 1.1 Add `UserPromptSubmitCommand` struct to `src/config.rs` with fields:
  - `run`: String (required)
  - `pattern`: Option<String> (regex pattern)
  - `case_insensitive`: Option<bool> (default false)
  - `show_command`: Option<bool> (default true)
  - `show_stdout`: Option<bool> (default false)
  - `show_stderr`: Option<bool> (default false)
  - `max_output_lines`: Option<u32> (1-10000)
  - `timeout`: Option<u64> (1-3600 seconds)

- [x] 1.2 Add `commands` field to `UserPromptSubmitConfig` struct:
  - `commands`: Vec<UserPromptSubmitCommand>

- [x] 1.3 Derive `FieldList` for `UserPromptSubmitCommand` struct (auto-generated field suggestions)

- [x] 1.4 Update `suggest_similar_fields()` to include userPromptSubmit command fields

## 2. Hook Implementation

- [x] 2.1 Create `UserPromptSubmitCommandConfig` internal struct in `src/hooks.rs`:
  - Similar to `StopCommandConfig` but with regex pattern field
  - Include compiled regex for efficient matching

- [x] 2.2 Create `build_user_prompt_submit_env_vars()` function in `src/hooks.rs`:
  - `CONCLAUDE_USER_PROMPT` - from payload.prompt
  - `CONCLAUDE_SESSION_ID` - from payload.base.session_id
  - `CONCLAUDE_CWD` - from payload.base.cwd
  - `CONCLAUDE_CONFIG_DIR` - from config path
  - `CONCLAUDE_HOOK_EVENT` - "UserPromptSubmit"

- [x] 2.3 Create `collect_user_prompt_submit_commands()` function:
  - Compile regex patterns for each command
  - Filter commands by pattern matching against payload.prompt
  - Return filtered command configs

- [x] 2.4 Create `execute_user_prompt_submit_commands()` function:
  - Similar to `execute_stop_commands()` but non-blocking (failures logged, not returned)
  - Run commands with environment variables
  - Respect timeout, showStdout, showStderr, maxOutputLines settings
  - Commands continue executing even if one fails

- [x] 2.5 Update `handle_user_prompt_submit()` to:
  - After contextRules processing, check for commands configuration
  - Build environment variables
  - Collect matching commands
  - Execute commands (non-blocking on failure)
  - Preserve existing context injection return behavior

## 3. Configuration Validation

- [x] 3.1 Add validation for `userPromptSubmit.commands[].pattern` regex syntax in `validate_config_constraints()`

- [x] 3.2 Add validation for `userPromptSubmit.commands[].timeout` range (1-3600)

- [x] 3.3 Add validation for `userPromptSubmit.commands[].maxOutputLines` range (1-10000)

## 4. Default Configuration

- [x] 4.1 Add commented `commands` example to `src/default-config.yaml` under userPromptSubmit section:
  - Example with pattern filtering
  - Example running for all prompts
  - Example with output display options

## 5. Tests

- [x] 5.1 Unit test: `collect_user_prompt_submit_commands()` filters by regex pattern

- [x] 5.2 Unit test: `build_user_prompt_submit_env_vars()` produces correct variables

- [x] 5.3 Unit test: Command with no pattern runs for all prompts

- [x] 5.4 Unit test: Case-insensitive pattern matching works

- [x] 5.5 Integration test: Commands execute after contextRules processing

- [x] 5.6 Integration test: Command failures don't block the hook result

- [x] 5.7 Unit test: Config validation rejects invalid regex patterns

- [x] 5.8 Unit test: Config validation rejects invalid timeout/maxOutputLines values

## 6. Documentation

- [x] 6.1 Update `docs/src/content/docs/reference/config/user-prompt-submit.md`:
  - Add `commands` section documentation
  - Document `UserPromptSubmitCommand` type properties
  - Add complete examples showing commands usage

- [x] 6.2 Verify JSON schema generates correctly for updated config section

## Dependencies

- Tasks 1.x must complete before 2.x (config types needed for hook implementation)
- Tasks 1.x and 3.x can run in parallel
- Task 2.5 depends on 2.1-2.4
- Task 5.x depends on corresponding implementation tasks
- Task 6.x can run after 1.x and 4.x complete
