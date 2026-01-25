# hook-payloads Specification Delta

## ADDED Requirements

### Requirement: UserPromptSubmit Payload Structure with Optional Prompt

The system SHALL support `UserPromptSubmitPayload` with an optional `prompt` field that accepts null values without validation failure.

#### Scenario: UserPromptSubmit with null prompt
- **GIVEN** Claude Code sends a UserPromptSubmit hook event with `"prompt": null`
- **WHEN** the payload is deserialized into UserPromptSubmitPayload
- **THEN** deserialization SHALL succeed
- **AND** the prompt field SHALL be `None` (Option type)
- **AND** the hook handler SHALL return success without error

#### Scenario: UserPromptSubmit with nil/missing prompt in JSON
- **GIVEN** Claude Code sends a UserPromptSubmit hook event with missing prompt field
- **WHEN** the payload is deserialized into UserPromptSubmitPayload
- **THEN** deserialization SHALL fail (serde requires the field to exist)
- **AND** an appropriate error SHALL indicate the missing field
- **NOTE** Claude Code should always include the prompt field; if it doesn't, that's a Claude Code issue

#### Scenario: UserPromptSubmit with empty string prompt
- **GIVEN** Claude Code sends a UserPromptSubmit hook event with `"prompt": ""`
- **WHEN** the payload is deserialized into UserPromptSubmitPayload
- **THEN** deserialization SHALL succeed
- **AND** the prompt field SHALL be `Some("")`
- **AND** the hook handler SHALL return success without error

#### Scenario: UserPromptSubmit with valid prompt text
- **GIVEN** Claude Code sends a UserPromptSubmit hook event with `"prompt": "actual prompt text"`
- **WHEN** the payload is deserialized into UserPromptSubmitPayload
- **THEN** deserialization SHALL succeed
- **AND** the prompt field SHALL be `Some("actual prompt text")`
- **AND** the hook handler SHALL process normally (context rules, commands)

#### Scenario: Hook handler graceful degradation with nil prompt
- **GIVEN** a UserPromptSubmit hook handler receives a payload with `prompt: None`
- **WHEN** evaluating context rules and commands
- **THEN** context rules SHALL NOT match (nil cannot match patterns)
- **AND** commands SHALL NOT be collected (nil has no content to match)
- **AND** the handler SHALL return HookResult::success()
- **AND** the hook SHALL NOT block Claude Code operation

#### Scenario: Round-trip serialization with null prompt
- **GIVEN** a UserPromptSubmitPayload with `prompt: None`
- **WHEN** the payload is serialized to JSON and then deserialized back
- **THEN** the JSON SHALL include `"prompt": null`
- **AND** the deserialized payload SHALL have `prompt: None`
- **AND** no data SHALL be lost in the round-trip

### Requirement: UserPromptSubmit Hook Handler Graceful Degradation

The `handle_user_prompt_submit()` hook handler SHALL process without error even when the prompt is null or empty.

#### Scenario: Handler succeeds with nil prompt
- **GIVEN** a UserPromptSubmit hook handler receives a payload with `prompt: None`
- **WHEN** the handler executes `handle_user_prompt_submit()`
- **THEN** it SHALL return HookResult::success()
- **AND** it SHALL NOT return an error with "Missing required field: prompt"
- **AND** context rules SHALL NOT be evaluated (nil prompt cannot match patterns)
- **AND** commands SHALL NOT be executed (nil prompt has no content to match)

#### Scenario: Handler succeeds with empty string prompt
- **GIVEN** a UserPromptSubmit hook handler receives a payload with `prompt: Some("")`
- **WHEN** the handler executes `handle_user_prompt_submit()`
- **THEN** it SHALL return HookResult::success()
- **AND** it SHALL NOT return an error
- **AND** context rules SHALL NOT be evaluated (empty string cannot match patterns)
- **AND** commands SHALL NOT be executed (empty prompt has no content to match)
