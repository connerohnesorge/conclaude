# PreToolUse Specification Delta

## ADDED Requirements

### Requirement: Updated Input with Ask Permission Decision

The system SHALL support returning `updatedInput` alongside an `ask` permission decision in PreToolUse hook responses, enabling hooks to act as middleware that modifies tool inputs while still requesting user consent.

#### Scenario: Ask decision with updated input
- **WHEN** a PreToolUse hook returns `decision: "ask"` with `updated_input` containing modified parameters
- **THEN** the system SHALL serialize both the decision and updated input in the response
- **AND** the user SHALL be prompted to approve the modified operation
- **AND** if approved, the tool SHALL execute with the modified input parameters

#### Scenario: Ask decision with reason and updated input
- **GIVEN** a PreToolUse hook that sanitizes bash commands
- **WHEN** the hook returns:
  ```json
  {
    "decision": "ask",
    "message": "Command modified for safety",
    "updated_input": {
      "command": "sanitized-command"
    }
  }
  ```
- **THEN** the user SHALL see the reason message
- **AND** if the user approves, the sanitized command SHALL be executed
- **AND** if the user denies, the operation SHALL be blocked

#### Scenario: Ask decision without updated input (backward compatible)
- **WHEN** a PreToolUse hook returns `decision: "ask"` without `updated_input`
- **THEN** the system SHALL prompt for user approval with the original tool input
- **AND** behavior SHALL be identical to the legacy permission flow

#### Scenario: Updated input partial replacement
- **GIVEN** a tool input with fields: `{ "command": "rm -rf /", "timeout": 30 }`
- **WHEN** a hook returns `updated_input: { "command": "echo hello" }`
- **THEN** only the `command` field SHALL be replaced
- **AND** the `timeout` field SHALL remain unchanged at 30
- **AND** the final input SHALL be `{ "command": "echo hello", "timeout": 30 }`

### Requirement: Permission Decision Field

The system SHALL support an explicit `decision` field in PreToolUse hook responses with values: "allow", "deny", or "ask".

#### Scenario: Decision field allow
- **WHEN** a PreToolUse hook returns `decision: "allow"`
- **THEN** the tool execution SHALL proceed without user prompt
- **AND** any `updated_input` SHALL be applied to the tool parameters
- **AND** behavior SHALL be equivalent to returning `blocked: false`

#### Scenario: Decision field deny
- **WHEN** a PreToolUse hook returns `decision: "deny"`
- **THEN** the tool execution SHALL be blocked
- **AND** the `message` field SHALL be shown to Claude as the reason
- **AND** behavior SHALL be equivalent to returning `blocked: true`

#### Scenario: Decision field ask
- **WHEN** a PreToolUse hook returns `decision: "ask"`
- **THEN** the user SHALL be prompted to approve or deny the operation
- **AND** the `message` field SHALL be shown to the user (not Claude)
- **AND** any `updated_input` SHALL be applied if the user approves

#### Scenario: Decision field takes precedence over blocked
- **WHEN** a PreToolUse hook returns both `decision: "allow"` and `blocked: true`
- **THEN** the `decision` field SHALL take precedence
- **AND** the tool SHALL be allowed to execute
- **AND** a warning MAY be logged about conflicting fields

#### Scenario: Missing decision field uses blocked field
- **WHEN** a PreToolUse hook returns without a `decision` field
- **THEN** the system SHALL use the `blocked` field to determine behavior
- **AND** `blocked: true` SHALL block the operation
- **AND** `blocked: false` SHALL allow the operation
- **AND** backward compatibility SHALL be maintained

### Requirement: Hook Result Serialization

The system SHALL correctly serialize HookResult with updated_input and decision fields to JSON for Claude Code consumption.

#### Scenario: Serialization includes all fields
- **GIVEN** a HookResult with decision = "ask", message = "Review this", and updated_input = { "command": "safe-cmd" }
- **WHEN** the result is serialized to JSON
- **THEN** the JSON SHALL include:
  ```json
  {
    "decision": "ask",
    "message": "Review this",
    "blocked": false,
    "system_prompt": null,
    "updated_input": {
      "command": "safe-cmd"
    }
  }
  ```

#### Scenario: Null fields omitted or null
- **GIVEN** a HookResult with only `blocked: false` set
- **WHEN** the result is serialized to JSON
- **THEN** optional fields (decision, updated_input) SHALL be null or omitted
- **AND** the JSON SHALL be valid for Claude Code consumption

#### Scenario: Updated input preserves value types
- **GIVEN** updated_input containing various types: string, number, boolean, array, object
- **WHEN** the result is serialized and deserialized
- **THEN** all value types SHALL be preserved correctly
- **AND** no type coercion SHALL occur
