# Tasks for add-subagent-start-hook

This document tracks implementation tasks for the SubagentStart hook support feature.

## Type System (4 tasks)

- [x] Add `SubagentStartPayload` struct to `src/types.rs` with fields: base, agent_id, subagent_type, agent_transcript_path
- [x] Derive Debug, Clone, Serialize, Deserialize for SubagentStartPayload
- [x] Add `SubagentStart` variant to `HookPayload` enum with #[serde(rename = "SubagentStart")]
- [x] Update HookPayload helper methods (session_id, transcript_path, hook_event_name) to handle SubagentStart variant

## Payload Validation (5 tasks)

- [x] Implement `validate_subagent_start_payload(payload: &SubagentStartPayload) -> Result<(), String>` in `src/types.rs`
- [x] Validate base payload fields by calling `validate_base_payload(&payload.base)`
- [x] Validate `agent_id` is non-empty (after trimming whitespace)
- [x] Validate `subagent_type` is non-empty (after trimming whitespace)
- [x] Validate `agent_transcript_path` is non-empty (after trimming whitespace)

## Hook Handler Implementation (5 tasks)

- [x] Create `handle_subagent_start()` async function in `src/hooks.rs`
- [x] Read and deserialize SubagentStartPayload from stdin using `read_payload_from_stdin()`
- [x] Validate payload using `validate_subagent_start_payload()`
- [x] Log SubagentStart event with session_id and agent_id
- [x] Return HookResult::success() on successful processing

## Environment Variables (3 tasks)

- [x] Export `CONCLAUDE_AGENT_ID` environment variable with payload.agent_id value
- [x] Export `CONCLAUDE_SUBAGENT_TYPE` environment variable with payload.subagent_type value
- [x] Export `CONCLAUDE_AGENT_TRANSCRIPT_PATH` environment variable with payload.agent_transcript_path value

## Notification Support (3 tasks)

- [x] Add "SubagentStart" to `is_system_event_hook()` function in `src/hooks.rs`
- [x] Call `send_notification()` with "SubagentStart", "success", and agent_id context in `handle_subagent_start()`
- [x] Verify SubagentStart respects `notifications.showSystemEvents` configuration flag

## CLI Integration (3 tasks)

- [x] Add `SubagentStart` command variant to CLI enum in `src/main.rs`
- [x] Wire `SubagentStart` command to call `handle_subagent_start()` in match statement
- [x] Use `handle_hook_result()` wrapper to standardize exit codes (0 success, 1 error, 2 blocked)

## Unit Tests - Payload Validation (7 tasks)

- [x] Test valid SubagentStartPayload passes validation
- [x] Test empty agent_id fails validation with "agent_id cannot be empty" error
- [x] Test whitespace-only agent_id fails validation
- [x] Test empty subagent_type fails validation with "subagent_type cannot be empty" error
- [x] Test empty agent_transcript_path fails validation with "agent_transcript_path cannot be empty" error
- [x] Test payload with leading/trailing spaces in agent_id passes validation (trim behavior)
- [x] Test invalid base payload (empty session_id) fails validation

## Unit Tests - Type System (3 tasks)

- [x] Test SubagentStartPayload deserialization from valid JSON
- [x] Test SubagentStartPayload deserialization failure with missing required fields
- [x] Test HookPayload::SubagentStart variant serialization and deserialization

## Integration Tests - Handler (5 tasks)

- [x] Test `handle_subagent_start()` with valid payload returns HookResult::success()
- [x] Test `handle_subagent_start()` with invalid payload (missing agent_id) returns error
- [x] Test `handle_subagent_start()` with different agent types (coder, tester, stuck)
- [x] Verify environment variables (CONCLAUDE_AGENT_ID, CONCLAUDE_SUBAGENT_TYPE) are set after successful execution
- [x] Verify logging output includes session_id and agent_id

## Integration Tests - Notifications (3 tasks)

- [x] Test SubagentStart sends notification when hooks includes "SubagentStart"
- [x] Test SubagentStart sends notification when hooks includes "*"
- [x] Test SubagentStart does NOT send notification when hooks does not include "SubagentStart" or "*"

## CLI Tests (4 tasks)

- [x] Test `conclaude SubagentStart` with valid JSON payload returns exit code 0
- [x] Test `conclaude SubagentStart` with invalid JSON returns exit code 1
- [x] Test `conclaude SubagentStart` with missing required field returns exit code 1
- [x] Verify CLI help text includes SubagentStart command

## Documentation (4 tasks)

- [x] Update README.md with SubagentStart hook description
- [x] Add SubagentStart payload structure documentation with field descriptions
- [x] Add SubagentStart configuration examples for notifications
- [x] Add manual testing example with sample JSON payload

## Validation & Polish (4 tasks)

- [x] Run `cargo fmt` to format all modified code
- [x] Run `cargo clippy` and fix all warnings
- [x] Run `cargo test` and verify all tests pass
- [x] Run `openspec validate add-subagent-start-hook --strict` and fix any issues

---

**Total: 52 tasks**

**Status:** Complete (52/52 complete)

## Task Dependencies

- Type System tasks MUST be completed before Payload Validation
- Payload Validation MUST be completed before Hook Handler Implementation
- Hook Handler Implementation MUST be completed before CLI Integration
- All implementation tasks SHOULD be completed before Documentation
- Validation & Polish tasks are final and depend on all previous tasks
