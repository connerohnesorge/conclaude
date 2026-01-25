# Change: Allow nil prompt field in UserPromptSubmit hook payload

## Why

Issue #227 reports that Claude Code sometimes sends a `UserPromptSubmit` hook event with a `null` (nil) `prompt` field instead of an empty string or valid prompt text. This causes conclaude to fail validation with "Missing required field: prompt", blocking the hook and preventing Claude Code from proceeding.

Rather than failing hard on a nil prompt, conclaude should gracefully handle this case by treating a nil prompt as valid input (just with no content to process) and allowing the hook to succeed without error.

## What Changes

### Modified Behavior: UserPromptSubmit Prompt Field

The `prompt` field in `UserPromptSubmitPayload` will change from a required non-empty `String` to an `Option<String>` that accepts `null` values.

**Before:**
- `prompt: String` (required, must be non-empty)
- Validation fails if prompt is missing or empty
- Hook returns error: "Missing required field: prompt"

**After:**
- `prompt: Option<String>` (optional, can be null)
- Validation succeeds even if prompt is null or empty
- Hook processes with nil prompt gracefully (no context injection, no command execution)

### Configuration Examples

With nil prompt, the UserPromptSubmit hook will:
1. Pass validation (no longer required)
2. Skip context rule matching (nil prompt cannot match patterns)
3. Skip command execution (no prompt to match against)
4. Return success with no side effects
5. Allow Claude Code to proceed normally

## Impact

- Affected specs: `hook-payloads`
- Affected code: `src/types.rs`, `src/hooks.rs`
- Breaking changes: None (makes validation more lenient, not stricter)
- Compatibility: Fixes interoperability with Claude Code's hook payload format

## Technical Notes

The `UserPromptSubmitPayload.prompt` field will be deserializable as `null`, allowing serde to silently accept JSON like:
```json
{
  "session_id": "...",
  "transcript_path": "...",
  "hook_event_name": "UserPromptSubmit",
  "cwd": "...",
  "prompt": null
}
```

Handler logic remains unchanged:
- If prompt is null or empty, context rules and commands simply don't execute
- No special error handling needed; Option type provides safe defaults
