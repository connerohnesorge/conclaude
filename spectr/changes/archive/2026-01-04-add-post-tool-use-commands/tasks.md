# Tasks: PostToolUse Hook Commands

## 1. Configuration Types

- [ ] 1.1 Add `PostToolUseCommand` struct to `src/config.rs` with fields:
  - `run`: String (required)
  - `tool`: Option<String> (glob pattern, defaults to "*")
  - `show_command`: Option<bool> (default true)
  - `show_stdout`: Option<bool> (default false)
  - `show_stderr`: Option<bool> (default false)
  - `max_output_lines`: Option<u32> (1-10000)
  - `timeout`: Option<u64> (1-3600 seconds)

- [ ] 1.2 Add `PostToolUseConfig` struct to `src/config.rs` with:
  - `commands`: Vec<PostToolUseCommand>

- [ ] 1.3 Add `post_tool_use` field to `ConclaudeConfig` struct

- [ ] 1.4 Derive `FieldList` for new config structs (auto-generated field suggestions)

## 2. Hook Implementation

- [ ] 2.1 Create `build_post_tool_use_env_vars()` function in `src/hooks.rs`:
  - `CONCLAUDE_TOOL_NAME` - from payload.tool_name
  - `CONCLAUDE_TOOL_INPUT` - JSON serialization of payload.tool_input
  - `CONCLAUDE_TOOL_OUTPUT` - JSON serialization of payload.tool_response
  - `CONCLAUDE_TOOL_TIMESTAMP` - ISO 8601 format of current time
  - `CONCLAUDE_TOOL_USE_ID` - from payload.tool_use_id (if present)
  - `CONCLAUDE_SESSION_ID` - from payload.base.session_id
  - `CONCLAUDE_CWD` - from payload.base.cwd
  - `CONCLAUDE_CONFIG_DIR` - from config path

- [ ] 2.2 Create `matches_tool_pattern()` function for glob matching tool names

- [ ] 2.3 Create `collect_post_tool_use_commands()` function:
  - Filter commands by tool pattern matching against payload.tool_name
  - Return filtered command configs

- [ ] 2.4 Create `execute_post_tool_use_commands()` function:
  - Similar to `execute_stop_commands()` but non-blocking (failures logged, not returned)
  - Run commands with environment variables
  - Respect timeout, showStdout, showStderr, maxOutputLines settings

- [ ] 2.5 Update `handle_post_tool_use()` to:
  - Load configuration
  - Build environment variables
  - Collect matching commands
  - Execute commands
  - Return success (never blocked)

## 3. Configuration Validation

- [ ] 3.1 Add validation for `postToolUse.commands[].timeout` range (1-3600)

- [ ] 3.2 Add validation for `postToolUse.commands[].maxOutputLines` range (1-10000)

- [ ] 3.3 Add validation for `postToolUse.commands[].tool` glob pattern syntax

- [ ] 3.4 Update `suggest_similar_fields()` to include postToolUse section fields

## 4. Default Configuration

- [ ] 4.1 Add commented `postToolUse` example to `src/default-config.yaml`:
  - Example with tool filtering for AskUserQuestion
  - Example with wildcard for all tools
  - Example with glob pattern (*Search*)

## 5. Tests

- [ ] 5.1 Unit test: `matches_tool_pattern()` with exact, glob, and wildcard patterns

- [ ] 5.2 Unit test: `build_post_tool_use_env_vars()` produces correct variables

- [ ] 5.3 Unit test: `collect_post_tool_use_commands()` filters by tool pattern

- [ ] 5.4 Integration test: PostToolUse hook executes commands and sets env vars

- [ ] 5.5 Integration test: Command failures don't block the hook result

- [ ] 5.6 Unit test: Config validation rejects invalid timeout/maxOutputLines

## 6. Documentation

- [ ] 6.1 Document postToolUse section in default-config.yaml comments

- [ ] 6.2 Verify JSON schema generates correctly for new config section

## Dependencies

- Tasks 1.x must complete before 2.x (config types needed for hook implementation)
- Tasks 1.x and 3.x can run in parallel
- Task 2.5 depends on 2.1-2.4
- Task 5.x depends on corresponding implementation tasks
- Task 6.x can run after 1.x and 4.x complete
