# subagent-hooks Specification

## Purpose

This specification defines the SubagentStart hook support in conclaude, which provides visibility and control when subagents begin execution. The SubagentStart hook mirrors the existing SubagentStop hook, enabling full lifecycle tracking (start + stop) for all subagents. This allows users to initialize environments, run setup commands, log subagent starts, and send notifications when subagents begin their work.

## Requirements

### Requirement: SubagentStartPayload type definition

The system MUST define a `SubagentStartPayload` struct that represents the data received when a SubagentStart hook fires.

**Type:** Rust struct with serde Serialize/Deserialize derives
**Location:** `src/types.rs`

**Fields:**
- `base: BasePayload` - Standard hook payload fields (session_id, transcript_path, hook_event_name, cwd, permission_mode)
- `agent_id: String` - Unique identifier for the subagent starting execution (required, non-empty)
- `subagent_type: String` - Type or category of the subagent (required, non-empty)
- `agent_transcript_path: String` - Path to the subagent's transcript file (required, non-empty)

#### Scenario: Valid SubagentStartPayload deserialization

**Given** Claude Code fires a SubagentStart hook with a JSON payload containing:
```json
{
  "session_id": "abc123",
  "transcript_path": "/tmp/main.json",
  "hook_event_name": "SubagentStart",
  "cwd": "/home/user/project",
  "permission_mode": "default",
  "agent_id": "coder",
  "subagent_type": "coder",
  "agent_transcript_path": "/tmp/agent_coder.json"
}
```
**When** conclaude deserializes the payload into `SubagentStartPayload`
**Then** all fields MUST be populated correctly
**And** `base.session_id` equals "abc123"
**And** `agent_id` equals "coder"
**And** `subagent_type` equals "coder"
**And** `agent_transcript_path` equals "/tmp/agent_coder.json"

#### Scenario: Missing agent_id field

**Given** a SubagentStart hook JSON payload without an `agent_id` field
**When** conclaude attempts to deserialize the payload
**Then** deserialization MUST fail with a clear error message
**And** the error indicates that `agent_id` is a required field

#### Scenario: Empty agent_id value

**Given** a SubagentStart hook JSON payload with `"agent_id": ""`
**When** conclaude validates the payload
**Then** validation MUST fail
**And** the error message indicates agent_id cannot be empty

#### Scenario: Missing subagent_type field

**Given** a SubagentStart hook JSON payload without a `subagent_type` field
**When** conclaude attempts to deserialize the payload
**Then** deserialization MUST fail with a clear error message
**And** the error indicates that `subagent_type` is a required field

### Requirement: SubagentStart payload validation function

The system MUST provide a validation function that ensures all required SubagentStartPayload fields are present and non-empty.

**Function signature:** `validate_subagent_start_payload(payload: &SubagentStartPayload) -> Result<(), String>`
**Location:** `src/types.rs`

#### Scenario: Complete valid payload passes validation

**Given** a SubagentStartPayload with all required fields populated
**When** `validate_subagent_start_payload()` is called
**Then** validation MUST return `Ok(())`
**And** no error messages are generated

#### Scenario: Empty agent_id fails validation

**Given** a SubagentStartPayload with `agent_id` set to empty string or whitespace
**When** `validate_subagent_start_payload()` is called
**Then** validation MUST return `Err("agent_id cannot be empty")`

#### Scenario: Empty subagent_type fails validation

**Given** a SubagentStartPayload with `subagent_type` set to empty string or whitespace
**When** `validate_subagent_start_payload()` is called
**Then** validation MUST return `Err("subagent_type cannot be empty")`

#### Scenario: Empty agent_transcript_path fails validation

**Given** a SubagentStartPayload with `agent_transcript_path` set to empty string or whitespace
**When** `validate_subagent_start_payload()` is called
**Then** validation MUST return `Err("agent_transcript_path cannot be empty")`

#### Scenario: Invalid base payload fails validation

**Given** a SubagentStartPayload with invalid base fields (e.g., empty session_id)
**When** `validate_subagent_start_payload()` is called
**Then** validation MUST return an error from `validate_base_payload()`
**And** the SubagentStart-specific validation is NOT executed

### Requirement: SubagentStart hook handler function

The system MUST provide a hook handler function that processes SubagentStart events.

**Function signature:** `async fn handle_subagent_start() -> Result<HookResult>`
**Location:** `src/hooks.rs`

#### Scenario: Successful SubagentStart hook execution

**Given** a valid SubagentStartPayload is provided via stdin
**When** `handle_subagent_start()` is called
**Then** the function MUST:
- Read and deserialize the payload from stdin
- Validate the payload using `validate_subagent_start_payload()`
- Log the event with session_id and agent_id
- Export environment variables (CONCLAUDE_AGENT_ID, CONCLAUDE_SUBAGENT_TYPE, CONCLAUDE_AGENT_TRANSCRIPT_PATH)
- Send a system notification (if configured)
- Return `HookResult::success()`

#### Scenario: Invalid payload causes handler failure

**Given** an invalid SubagentStartPayload (missing agent_id) is provided via stdin
**When** `handle_subagent_start()` is called
**Then** the function MUST return an error
**And** the error message indicates which field is invalid
**And** no environment variables are exported
**And** no notification is sent

#### Scenario: Environment variables exported on success

**Given** a valid SubagentStartPayload with:
- `agent_id` = "tester"
- `subagent_type` = "tester"
- `agent_transcript_path` = "/tmp/tester.json"
**When** `handle_subagent_start()` executes successfully
**Then** the following environment variables MUST be set:
- `CONCLAUDE_AGENT_ID` = "tester"
- `CONCLAUDE_SUBAGENT_TYPE` = "tester"
- `CONCLAUDE_AGENT_TRANSCRIPT_PATH` = "/tmp/tester.json"
**And** standard environment variables are also set (SESSION_ID, TRANSCRIPT_PATH, CWD, HOOK_EVENT)

### Requirement: HookPayload enum includes SubagentStart variant

The `HookPayload` enum MUST include a `SubagentStart` variant for type-safe payload handling.

**Location:** `src/types.rs`

#### Scenario: SubagentStart variant added to enum

**Given** the HookPayload enum in `src/types.rs`
**When** a SubagentStart hook event occurs
**Then** the enum MUST have a variant defined as:
```rust
#[serde(rename = "SubagentStart")]
SubagentStart(SubagentStartPayload)
```
**And** the variant supports serde serialization/deserialization
**And** helper methods (session_id(), transcript_path(), hook_event_name()) work for SubagentStart

### Requirement: CLI command for SubagentStart hook

The CLI MUST provide a `SubagentStart` command that accepts JSON payloads via stdin.

**Command:** `conclaude SubagentStart`
**Location:** `src/main.rs`

#### Scenario: SubagentStart command processes valid input

**Given** a valid SubagentStart JSON payload is piped to stdin
**When** user executes `conclaude SubagentStart`
**Then** the command MUST invoke `handle_subagent_start()`
**And** return exit code 0 on success
**And** return exit code 1 on validation error
**And** return exit code 2 if blocked (though SubagentStart doesn't block)

#### Scenario: SubagentStart command with invalid JSON

**Given** invalid JSON is piped to stdin
**When** user executes `conclaude SubagentStart`
**Then** the command MUST return exit code 1
**And** error message indicates JSON parsing failed

### Requirement: SubagentStart classified as system event hook

The `is_system_event_hook()` function MUST classify "SubagentStart" as a system event hook for notification routing.

**Location:** `src/hooks.rs`

#### Scenario: SubagentStart recognized as system event

**Given** the `is_system_event_hook()` function
**When** called with hook_name "SubagentStart"
**Then** the function MUST return `true`
**And** notifications with `showSystemEvents: true` will fire for SubagentStart

#### Scenario: SubagentStart notifications respect showSystemEvents flag

**Given** notifications config with `showSystemEvents: false`
**When** a SubagentStart hook executes successfully
**Then** no system notification is sent
**And** the hook still completes successfully

### Requirement: Notification support for SubagentStart events

System notifications MUST support SubagentStart hooks when configured.

**Configuration:** `notifications.hooks` array in `.conclaude.yaml`

#### Scenario: SubagentStart in hooks list enables notifications

**Given** notifications config with:
```yaml
notifications:
  enabled: true
  hooks: ["SubagentStart"]
  showSystemEvents: true
```
**When** a SubagentStart hook executes
**Then** a system notification MUST be sent
**And** the notification title includes "SubagentStart"
**And** the notification body includes the agent_id

#### Scenario: Wildcard hooks config includes SubagentStart

**Given** notifications config with `hooks: ["*"]`
**When** a SubagentStart hook executes
**Then** a system notification MUST be sent (subject to showSystemEvents flag)

#### Scenario: SubagentStart not in hooks list suppresses notifications

**Given** notifications config with `hooks: ["Stop", "PreToolUse"]` (SubagentStart not listed)
**When** a SubagentStart hook executes
**Then** no system notification is sent
**And** the hook still executes successfully

### Requirement: SubagentStart logging includes agent context

Hook execution logs MUST include agent_id and subagent_type for debugging and monitoring.

#### Scenario: SubagentStart log message includes agent details

**Given** a SubagentStart payload with agent_id "coder"
**When** `handle_subagent_start()` processes the payload
**Then** the log output MUST include:
- "Processing SubagentStart hook"
- session_id value
- agent_id value
**And** the log format is consistent with other hook handlers

### Requirement: Test coverage for SubagentStart functionality

The test suite MUST include comprehensive tests for SubagentStart hook processing.

**Location:** `tests/hooks_tests.rs` or `src/types.rs` (for unit tests)

#### Scenario: Unit tests validate payload fields

**Given** the test suite in `src/types.rs`
**When** tests run
**Then** the following test cases MUST exist and pass:
- Valid payload passes validation
- Empty agent_id fails validation
- Whitespace-only agent_id fails validation
- Empty subagent_type fails validation
- Empty agent_transcript_path fails validation
- Invalid base payload fails validation
- Payload with leading/trailing spaces in agent_id passes (after trim check)

#### Scenario: Integration tests verify handler execution

**Given** integration tests for SubagentStart hook
**When** tests run
**Then** the following scenarios MUST be tested:
- Valid payload returns HookResult::success()
- Invalid payload returns error
- Environment variables are set correctly
- Notifications fire when configured
- Different agent types (coder, tester, stuck) are handled correctly

### Requirement: Subagent Stop Command Configuration
The system SHALL provide a `subagentStop` configuration section that maps subagent name patterns to lists of commands to execute when matching subagents terminate.

#### Scenario: Wildcard pattern configuration
- **WHEN** config includes `subagentStop.commands["*"]` with a list of commands
- **THEN** those commands SHALL execute for every subagent that stops
- **AND** each command SHALL include run, message, showStdout, showStderr, maxOutputLines fields

#### Scenario: Exact match pattern configuration
- **WHEN** config includes `subagentStop.commands["coder"]` with a list of commands
- **THEN** those commands SHALL execute only when a subagent named "coder" stops
- **AND** commands SHALL NOT execute for subagents with different names

#### Scenario: Glob pattern configuration
- **WHEN** config includes `subagentStop.commands["test*"]` with commands
- **THEN** those commands SHALL execute for subagents matching the glob pattern
- **AND** pattern matching SHALL support `*`, `?`, `[...]` glob syntax

#### Scenario: Multiple pattern configuration
- **WHEN** config includes both wildcard and specific patterns (e.g., `*` and `coder`)
- **THEN** the configuration SHALL be valid
- **AND** both patterns SHALL be evaluated independently during matching

### Requirement: Agent ID from Payload
The system SHALL use the `agent_id` field from SubagentStopPayload (provided by `add-subagent-stop-payload-fields`) to identify the subagent for pattern matching.

#### Scenario: Agent ID provided in payload
- **WHEN** SubagentStop hook fires with valid SubagentStopPayload
- **AND** payload contains `agent_id` field (e.g., "coder", "tester", "stuck")
- **THEN** the system SHALL use the agent_id value for pattern matching
- **AND** SHALL NOT need to parse transcript files

#### Scenario: Agent ID used for all pattern matching
- **WHEN** agent_id is "coder"
- **AND** configuration has patterns like `*`, `coder`, `*coder`
- **THEN** all pattern matching SHALL be performed against the agent_id value "coder"
- **AND** no transcript parsing or file reading is required

### Requirement: Glob Pattern Matching
The system SHALL match subagent names against configured patterns using glob syntax, supporting wildcard, exact, and glob patterns.

#### Scenario: Wildcard matches all subagents
- **WHEN** pattern is `*`
- **AND** any subagent stops (e.g., "coder", "tester", "stuck")
- **THEN** the pattern SHALL match
- **AND** associated commands SHALL be queued for execution

#### Scenario: Exact match pattern
- **WHEN** pattern is `coder`
- **AND** subagent name is exactly "coder"
- **THEN** the pattern SHALL match
- **AND** pattern SHALL NOT match "auto-coder" or "coder-agent"

#### Scenario: Prefix glob pattern
- **WHEN** pattern is `test*`
- **AND** subagent name is "tester", "test-runner", or "testing"
- **THEN** the pattern SHALL match
- **AND** pattern SHALL NOT match "runner-test"

#### Scenario: Suffix glob pattern
- **WHEN** pattern is `*coder`
- **AND** subagent name is "coder", "auto-coder", or "smart-coder"
- **THEN** the pattern SHALL match
- **AND** pattern SHALL NOT match "coder-agent"

#### Scenario: Character class glob pattern
- **WHEN** pattern is `agent_[0-9]*`
- **AND** subagent name is "agent_1", "agent_2x", or "agent_99test"
- **THEN** the pattern SHALL match
- **AND** pattern SHALL NOT match "agent_x" or "agent"

#### Scenario: Multiple patterns match same subagent
- **WHEN** subagent name is "coder"
- **AND** config has patterns `*`, `coder`, and `*coder`
- **THEN** all three patterns SHALL match
- **AND** commands from all matching patterns SHALL be collected for execution

### Requirement: Command Execution Order
The system SHALL execute commands in a deterministic order when multiple patterns match the same subagent.

#### Scenario: Wildcard and specific pattern both match
- **WHEN** subagent "coder" stops
- **AND** config has both `*` and `coder` patterns with commands
- **THEN** wildcard (`*`) commands SHALL execute first
- **AND** specific (`coder`) commands SHALL execute second
- **AND** all commands SHALL complete before hook returns

#### Scenario: Multiple glob patterns match
- **WHEN** subagent "auto-coder" stops
- **AND** config has patterns `*coder`, `auto*`, and `*`
- **THEN** wildcard (`*`) commands SHALL execute first
- **AND** other matching patterns SHALL execute in stable order
- **AND** execution order SHALL be consistent across runs

#### Scenario: No patterns match
- **WHEN** subagent "unknown-agent" stops
- **AND** config only has specific patterns like `coder` and `tester`
- **THEN** no commands SHALL execute
- **AND** SubagentStop hook SHALL complete successfully
- **AND** notification (if configured) SHALL still be sent

### Requirement: Environment Variable Context
The system SHALL pass subagent context to command execution via environment variables.

#### Scenario: Environment variables available in commands
- **WHEN** a subagent stop command executes
- **THEN** the following environment variables SHALL be available:
  - `CONCLAUDE_AGENT_ID` - Raw agent identifier from payload (e.g., "adb0a8b")
  - `CONCLAUDE_AGENT_NAME` - Semantic agent name extracted from main transcript (e.g., "coder", "tester", "stuck"), falls back to agent_id if extraction fails
  - `CONCLAUDE_AGENT_TRANSCRIPT_PATH` - Agent's transcript path from payload (provided by add-subagent-stop-payload-fields)
  - `CONCLAUDE_SESSION_ID` - Session ID from payload
  - `CONCLAUDE_TRANSCRIPT_PATH` - Main transcript file path
  - `CONCLAUDE_HOOK_EVENT` - Always "SubagentStop"
  - `CONCLAUDE_CWD` - Current working directory

#### Scenario: Agent ID in environment variable
- **WHEN** agent with agent_id "coder" stops and command executes
- **THEN** `CONCLAUDE_AGENT_ID` environment variable SHALL equal "coder"
- **AND** command can access this via `$CONCLAUDE_AGENT_ID` in bash

#### Scenario: Agent name extracted from main transcript
- **WHEN** agent with agent_id "abc123" stops
- **AND** main transcript contains Task tool call with subagent_type "coder"
- **THEN** `CONCLAUDE_AGENT_NAME` environment variable SHALL equal "coder"
- **AND** `CONCLAUDE_AGENT_ID` environment variable SHALL equal "abc123"
- **AND** commands can use the semantic name via `$CONCLAUDE_AGENT_NAME`

#### Scenario: Agent name extraction fails, fallback to agent_id
- **WHEN** agent with agent_id "xyz789" stops
- **AND** main transcript does not contain matching Task tool call
- **THEN** `CONCLAUDE_AGENT_NAME` environment variable SHALL equal "xyz789"
- **AND** `CONCLAUDE_AGENT_ID` environment variable SHALL equal "xyz789"
- **AND** both variables have the same value (fallback behavior)

#### Scenario: Agent transcript path in environment variable
- **WHEN** agent stops with agent_transcript_path "/tmp/agent_coder.json"
- **THEN** `CONCLAUDE_AGENT_TRANSCRIPT_PATH` SHALL equal "/tmp/agent_coder.json"
- **AND** commands can read the agent-specific transcript if needed
- **AND** this is separate from `CONCLAUDE_TRANSCRIPT_PATH` which points to the main session transcript

#### Scenario: Environment variables do not conflict with system
- **WHEN** commands execute with environment variables
- **THEN** all conclaude variables SHALL use `CONCLAUDE_` prefix
- **AND** SHALL NOT override standard environment variables (PATH, HOME, etc.)

### Requirement: Command Output Handling
The system SHALL respect showStdout, showStderr, and maxOutputLines settings for subagent stop commands.

#### Scenario: Command with showStdout enabled
- **WHEN** subagent stop command has `showStdout: true`
- **AND** command produces stdout
- **THEN** stdout SHALL be displayed to user/Claude
- **AND** maxOutputLines limit SHALL apply if configured

#### Scenario: Command with showStderr enabled
- **WHEN** subagent stop command has `showStderr: true`
- **AND** command produces stderr
- **THEN** stderr SHALL be displayed to user/Claude
- **AND** maxOutputLines limit SHALL apply if configured

#### Scenario: Command with output disabled
- **WHEN** subagent stop command has `showStdout: false` and `showStderr: false`
- **THEN** no command output SHALL be shown to user/Claude
- **AND** command SHALL still execute fully

### Requirement: Graceful Command Failure Handling
The system SHALL handle command failures without blocking SubagentStop hook completion.

#### Scenario: Command execution fails
- **WHEN** a subagent stop command fails (non-zero exit code)
- **THEN** the failure SHALL be logged
- **AND** SubagentStop hook SHALL continue processing
- **AND** remaining commands SHALL still execute

#### Scenario: Command spawning fails
- **WHEN** a subagent stop command cannot be spawned (command not found, permission denied)
- **THEN** the error SHALL be logged
- **AND** SubagentStop hook SHALL continue processing
- **AND** SubagentStop notification SHALL still be sent (if configured)

#### Scenario: All commands complete despite individual failures
- **WHEN** multiple subagent stop commands are configured
- **AND** some commands fail during execution
- **THEN** all commands SHALL be attempted
- **AND** SubagentStop hook SHALL complete with success status

### Requirement: Backward Compatibility
The system SHALL maintain existing SubagentStop notification behavior when no subagentStop config is present.

#### Scenario: Config without subagentStop section
- **WHEN** configuration does not include `subagentStop` section
- **AND** SubagentStop hook fires
- **THEN** existing notification behavior SHALL work unchanged
- **AND** no commands SHALL attempt to execute
- **AND** hook SHALL complete successfully

#### Scenario: Empty subagentStop configuration
- **WHEN** configuration includes `subagentStop: {}` with no commands
- **AND** SubagentStop hook fires
- **THEN** no commands SHALL execute
- **AND** existing notification behavior SHALL work unchanged

#### Scenario: subagentStop config does not affect other hooks
- **WHEN** subagentStop configuration is present
- **THEN** Stop, SessionEnd, and other hooks SHALL be unaffected
- **AND** only SubagentStop hook SHALL use subagentStop config

### Requirement: Configuration Validation
The system SHALL validate subagentStop configuration at load time to catch errors early.

#### Scenario: Valid subagentStop configuration
- **WHEN** config includes valid subagentStop section with patterns and commands
- **THEN** configuration SHALL load successfully
- **AND** schema validation SHALL pass

#### Scenario: Invalid command structure
- **WHEN** subagentStop command is missing required `run` field
- **THEN** configuration loading SHALL fail with validation error
- **AND** error message SHALL indicate missing field

#### Scenario: Invalid maxOutputLines value
- **WHEN** subagentStop command has `maxOutputLines: 0` or negative value
- **THEN** configuration loading SHALL fail with validation error
- **AND** error message SHALL indicate invalid value range

#### Scenario: Pattern is empty string
- **WHEN** config includes pattern key as empty string ("")
- **THEN** configuration loading SHALL fail with validation error
- **AND** error message SHALL indicate invalid pattern
