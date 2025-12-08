# Hooks System Specification

## Purpose

Defines the hook payload types and enum structure for handling Claude Code hook events, including PermissionRequest hooks for tool permission decisions.

## Requirements

### Requirement: PermissionRequest Hook Payload Type
The system SHALL define and support a `PermissionRequestPayload` struct for representing tool permission request events.

#### Scenario: Payload with tool name and input
- **WHEN** a tool permission request event is received from Claude Agent
- **THEN** a `PermissionRequestPayload` SHALL be created with:
  - `tool_name`: The name of the tool being requested (e.g., "Bash", "Edit")
  - `tool_input`: A JSON map of input parameters for the tool
  - `base`: Standard hook payload fields (session_id, transcript_path, hook_event_name, cwd, permission_mode)

#### Scenario: Deserializing PermissionRequest event JSON
- **WHEN** a JSON hook event with `"hook_event_name": "PermissionRequest"` is received
- **THEN** it SHALL deserialize to `PermissionRequestPayload` struct
- **AND** all fields SHALL be correctly parsed and preserved

#### Scenario: Accessing base payload fields
- **WHEN** a `PermissionRequestPayload` is available
- **THEN** calling `.session_id()` SHALL return the session ID
- **AND** calling `.transcript_path()` SHALL return the transcript path
- **AND** calling `.hook_event_name()` SHALL return "PermissionRequest"

### Requirement: HookPayload Enum Extension
The system SHALL extend the `HookPayload` enum to include `PermissionRequest` variant.

#### Scenario: PermissionRequest variant in enum
- **WHEN** the `HookPayload` enum is used
- **THEN** it SHALL have a `PermissionRequest(PermissionRequestPayload)` variant
- **AND** the variant SHALL be tagged with `#[serde(rename = "PermissionRequest")]`

#### Scenario: Pattern matching on hook payload
- **WHEN** pattern matching on `HookPayload`
- **THEN** matching the `PermissionRequest` variant SHALL work correctly
- **AND** accessing the inner `PermissionRequestPayload` SHALL be straightforward

#### Scenario: Helper methods work on PermissionRequest
- **WHEN** calling helper methods on `HookPayload::PermissionRequest(...)`
- **THEN** `session_id()` SHALL return the session ID
- **AND** `transcript_path()` SHALL return the transcript path
- **AND** `hook_event_name()` SHALL return "PermissionRequest"

### Requirement: Backward Compatibility with Existing Hooks
The system SHALL maintain full backward compatibility with existing hook types.

#### Scenario: PreToolUse hooks continue working
- **WHEN** PreToolUse hooks are configured and triggered
- **THEN** they SHALL function identically to before PermissionRequest support
- **AND** no changes to PreToolUse behavior SHALL occur

#### Scenario: Other hook types unaffected
- **WHEN** Stop, SubagentStop, Notification, or other hooks are triggered
- **THEN** they SHALL function identically to before PermissionRequest support
- **AND** PermissionRequest addition SHALL not affect other hook processing

#### Scenario: Mixed hook types in same session
- **WHEN** a session triggers multiple hook types (PermissionRequest, PreToolUse, PostToolUse)
- **THEN** each hook type SHALL be routed correctly
- **AND** processing of one hook type SHALL not interfere with others

### Requirement: Test Organization
Unit tests for the hooks module SHALL be organized in a dedicated test file (`src/hooks_tests.rs`) rather than inline within the implementation file.

#### Scenario: Test file structure
- **WHEN** examining the hooks test code
- **THEN** tests SHALL reside in `src/hooks_tests.rs`
- **AND** the test module SHALL be conditionally compiled with `#[cfg(test)]`
- **AND** all existing test coverage SHALL be preserved
