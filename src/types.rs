use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response structure returned by hook handlers to control execution flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Custom message to display to the user
    pub message: Option<String>,
    /// Whether to block the current operation from proceeding
    pub blocked: Option<bool>,
    /// Context to prepend to Claude's system prompt when context injection rules match
    pub system_prompt: Option<String>,
    /// Modified tool input parameters to replace the original input (PreToolUse only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Explicit permission decision for PreToolUse hooks: "allow", "deny", or "ask"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
}

impl HookResult {
    #[must_use]
    pub fn success() -> Self {
        Self {
            message: None,
            blocked: Some(false),
            system_prompt: None,
            updated_input: None,
            decision: None,
        }
    }

    pub fn blocked(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            blocked: Some(true),
            system_prompt: None,
            updated_input: None,
            decision: None,
        }
    }

    /// Create a success result with injected context
    #[must_use]
    pub fn with_context(context: impl Into<String>) -> Self {
        Self {
            message: None,
            blocked: Some(false),
            system_prompt: Some(context.into()),
            updated_input: None,
            decision: None,
        }
    }

    /// Create a result that asks user permission while providing modified input
    /// The updated_input will be used if the user approves the operation
    #[must_use]
    #[allow(dead_code)]
    pub fn ask_with_input(
        message: impl Into<String>,
        updated_input: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            message: Some(message.into()),
            blocked: None,
            system_prompt: None,
            updated_input: Some(updated_input),
            decision: Some("ask".to_string()),
        }
    }
}

/// Base fields present in all hook payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasePayload {
    /// Unique identifier for the current Claude session
    pub session_id: String,
    /// Path to the JSONL transcript file containing conversation history
    pub transcript_path: String,
    /// Hook event type identifier
    pub hook_event_name: String,
    /// Current working directory
    pub cwd: String,
    /// Current permission mode (e.g., "default", "acceptEdits", "bypassPermissions", "plan")
    pub permission_mode: Option<String>,
}

/// Payload for `PreToolUse` hook - fired before Claude executes a tool.
/// Allows blocking or modifying tool execution before it occurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolUsePayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Name of the tool about to be executed (e.g., "Edit", "Bash", "Read")
    pub tool_name: String,
    /// Input parameters that will be passed to the tool
    pub tool_input: HashMap<String, serde_json::Value>,
    /// Unique identifier for this tool invocation, allowing correlation between PreToolUse and PostToolUse events.
    pub tool_use_id: Option<String>,
}

/// Payload for `PostToolUse` hook - fired after Claude executes a tool.
/// Contains both the input and response data for analysis or logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolUsePayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Name of the tool that was executed
    pub tool_name: String,
    /// Input parameters that were passed to the tool
    pub tool_input: HashMap<String, serde_json::Value>,
    /// Unique identifier for this tool invocation, allowing correlation between PreToolUse and PostToolUse events.
    pub tool_use_id: Option<String>,
    /// Response data returned by the tool execution (can be any JSON value)
    pub tool_response: serde_json::Value,
}

/// Payload for `PermissionRequest` hook - fired when Claude requests permission to execute a tool.
/// Allows granting or denying permission based on the tool and its parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequestPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Name of the tool requesting permission (e.g., "Edit", "Bash", "Read")
    pub tool_name: String,
    /// Input parameters for the tool requesting permission
    pub tool_input: HashMap<String, serde_json::Value>,
}

/// Payload for Notification hook - fired when Claude sends system notifications.
/// Used for displaying messages or alerts to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// The notification message content
    pub message: String,
    /// Optional title for the notification
    pub title: Option<String>,
}

/// Payload for Stop hook - fired when a Claude session is terminating.
/// Allows for cleanup operations or final processing before session ends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Whether stop hooks are currently active for this session
    pub stop_hook_active: bool,
}

/// Payload for `SubagentStart` hook - fired when a Claude subagent is launched.
/// Subagents are spawned for complex tasks and this fires when they begin execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentStartPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Unique identifier for the subagent being started (e.g., "coder", "tester", "stuck")
    pub agent_id: String,
    /// Type of subagent being started
    pub subagent_type: String,
    /// Path to the subagent's specific transcript file for conversation history
    pub agent_transcript_path: String,
}

/// Payload for `SubagentStop` hook - fired when a Claude subagent terminates.
/// Subagents are spawned for complex tasks and this fires when they complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentStopPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Whether stop hooks are currently active for this session
    pub stop_hook_active: bool,
    /// Unique identifier for the subagent that completed (e.g., "coder", "tester", "stuck")
    pub agent_id: String,
    /// Path to the subagent's specific transcript file containing conversation history
    pub agent_transcript_path: String,
}

/// Payload for `UserPromptSubmit` hook - fired when user submits input to Claude.
/// Allows processing or validation of user input before Claude processes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptSubmitPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// The user's input prompt text (can be null if Claude Code sends no prompt)
    pub prompt: Option<String>,
}

/// Payload for `PreCompact` hook - fired before transcript compaction occurs.
/// Transcript compaction reduces conversation history size to manage context limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCompactPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Whether compaction was triggered manually by user or automatically by system
    pub trigger: CompactTrigger,
    /// Custom instructions provided for compaction (if any)
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompactTrigger {
    Manual,
    Auto,
}

/// Payload for `SessionStart` hook - fired when a new Claude session begins.
/// Allows initialization or setup operations at the start of a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Source that initiated the session (e.g., CLI, IDE integration)
    pub source: String,
}

/// Payload for `SessionEnd` hook - fired when a Claude session terminates.
/// Allows cleanup operations or final logging at the end of a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndPayload {
    #[serde(flatten)]
    pub base: BasePayload,
    /// Reason for session termination (e.g., "user_exit", "error", "completion")
    pub reason: String,
}

/// Validates that a payload contains all required base fields.
///
/// # Errors
///
/// Returns an error if any required base field is missing or empty.
pub fn validate_base_payload(base: &BasePayload) -> Result<(), String> {
    if base.session_id.is_empty() {
        return Err("Missing required field: session_id".to_string());
    }
    if base.transcript_path.is_empty() {
        return Err("Missing required field: transcript_path".to_string());
    }
    if base.hook_event_name.is_empty() {
        return Err("Missing required field: hook_event_name".to_string());
    }
    if base.cwd.is_empty() {
        return Err("Missing required field: cwd".to_string());
    }
    Ok(())
}

/// Validates that a PermissionRequestPayload contains all required fields.
///
/// # Errors
///
/// Returns an error if any required field is missing or empty (after trimming whitespace).
pub fn validate_permission_request_payload(
    payload: &PermissionRequestPayload,
) -> Result<(), String> {
    // First validate the base payload
    validate_base_payload(&payload.base)?;

    // Validate tool_name
    if payload.tool_name.trim().is_empty() {
        return Err("tool_name cannot be empty".to_string());
    }

    Ok(())
}

/// Validates that a SubagentStartPayload contains all required fields.
///
/// # Errors
///
/// Returns an error if any required field is missing or empty (after trimming whitespace).
pub fn validate_subagent_start_payload(payload: &SubagentStartPayload) -> Result<(), String> {
    // First validate the base payload
    validate_base_payload(&payload.base)?;

    // Validate agent_id
    if payload.agent_id.trim().is_empty() {
        return Err("agent_id cannot be empty".to_string());
    }

    // Validate subagent_type
    if payload.subagent_type.trim().is_empty() {
        return Err("subagent_type cannot be empty".to_string());
    }

    // Validate agent_transcript_path
    if payload.agent_transcript_path.trim().is_empty() {
        return Err("agent_transcript_path cannot be empty".to_string());
    }

    Ok(())
}

/// Validates that a SubagentStopPayload contains all required fields.
///
/// # Errors
///
/// Returns an error if any required field is missing or empty (after trimming whitespace).
pub fn validate_subagent_stop_payload(payload: &SubagentStopPayload) -> Result<(), String> {
    // First validate the base payload
    validate_base_payload(&payload.base)?;

    // Validate agent_id
    if payload.agent_id.trim().is_empty() {
        return Err("agent_id cannot be empty".to_string());
    }

    // Validate agent_transcript_path
    if payload.agent_transcript_path.trim().is_empty() {
        return Err("agent_transcript_path cannot be empty".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_tool_use_payload_deserialization_without_tool_use_id() {
        let json = r#"{
            "session_id": "test_session",
            "transcript_path": "/path/to/transcript",
            "hook_event_name": "PreToolUse",
            "cwd": "/current/dir",
            "permission_mode": "default",
            "tool_name": "Edit",
            "tool_input": {"param1": "value1"}
        }"#;

        let payload: PreToolUsePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.tool_use_id, None);
        assert_eq!(payload.tool_name, "Edit");
        assert_eq!(payload.base.session_id, "test_session");
    }

    #[test]
    fn test_post_tool_use_payload_deserialization_without_tool_use_id() {
        let json = r#"{
            "session_id": "test_session",
            "transcript_path": "/path/to/transcript",
            "hook_event_name": "PostToolUse",
            "cwd": "/current/dir",
            "permission_mode": "default",
            "tool_name": "Edit",
            "tool_input": {"param1": "value1"},
            "tool_response": {"status": "success"}
        }"#;

        let payload: PostToolUsePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.tool_use_id, None);
        assert_eq!(payload.tool_name, "Edit");
        assert_eq!(payload.base.session_id, "test_session");
    }

    #[test]
    fn test_tool_use_id_round_trip_serialization() {
        // Test PreToolUsePayload round-trip
        let mut tool_input = HashMap::new();
        tool_input.insert("param1".to_string(), serde_json::json!("value1"));

        let pre_payload = PreToolUsePayload {
            base: BasePayload {
                session_id: "test_session".to_string(),
                transcript_path: "/path/to/transcript".to_string(),
                hook_event_name: "PreToolUse".to_string(),
                cwd: "/current/dir".to_string(),
                permission_mode: Some("default".to_string()),
            },
            tool_name: "Edit".to_string(),
            tool_input: tool_input.clone(),
            tool_use_id: Some("round-trip-id".to_string()),
        };

        let json = serde_json::to_string(&pre_payload).unwrap();
        let deserialized: PreToolUsePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_use_id, Some("round-trip-id".to_string()));

        // Test PostToolUsePayload round-trip
        let post_payload = PostToolUsePayload {
            base: BasePayload {
                session_id: "test_session".to_string(),
                transcript_path: "/path/to/transcript".to_string(),
                hook_event_name: "PostToolUse".to_string(),
                cwd: "/current/dir".to_string(),
                permission_mode: Some("default".to_string()),
            },
            tool_name: "Edit".to_string(),
            tool_input,
            tool_use_id: Some("round-trip-id-2".to_string()),
            tool_response: serde_json::json!({"status": "success"}),
        };

        let json = serde_json::to_string(&post_payload).unwrap();
        let deserialized: PostToolUsePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.tool_use_id,
            Some("round-trip-id-2".to_string())
        );
    }

    // Tests for context injection - verifying HookResult behavior

    #[test]
    fn test_hook_result_with_context_serialization() {
        let result = HookResult::with_context("Injected context");
        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains("\"system_prompt\":\"Injected context\""));
        assert!(json.contains("\"blocked\":false"));
    }

    #[test]
    fn test_hook_result_with_context_deserialization() {
        let json = r#"{
            "message": null,
            "blocked": false,
            "system_prompt": "Test context"
        }"#;

        let result: HookResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.system_prompt, Some("Test context".to_string()));
        assert_eq!(result.blocked, Some(false));
        assert_eq!(result.message, None);
    }

    #[test]
    fn test_user_prompt_submit_payload_serialization() {
        let payload = UserPromptSubmitPayload {
            base: BasePayload {
                session_id: "test_session".to_string(),
                transcript_path: "/path/to/transcript".to_string(),
                hook_event_name: "UserPromptSubmit".to_string(),
                cwd: "/current/dir".to_string(),
                permission_mode: Some("default".to_string()),
            },
            prompt: Some("test user prompt".to_string()),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"prompt\":\"test user prompt\""));
        assert!(json.contains("\"session_id\":\"test_session\""));
    }

    #[test]
    fn test_user_prompt_submit_payload_deserialization() {
        let json = r#"{
            "session_id": "test_session",
            "transcript_path": "/path/to/transcript",
            "hook_event_name": "UserPromptSubmit",
            "cwd": "/current/dir",
            "permission_mode": "default",
            "prompt": "test prompt"
        }"#;

        let payload: UserPromptSubmitPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.prompt, Some("test prompt".to_string()));
        assert_eq!(payload.base.session_id, "test_session");
    }

    #[test]
    fn test_user_prompt_submit_payload_with_nil_prompt() {
        let json = r#"{
            "session_id": "test_session",
            "transcript_path": "/path/to/transcript",
            "hook_event_name": "UserPromptSubmit",
            "cwd": "/current/dir",
            "permission_mode": "default",
            "prompt": null
        }"#;

        let payload: UserPromptSubmitPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.prompt, None);
        assert_eq!(payload.base.session_id, "test_session");
    }

    #[test]
    fn test_user_prompt_submit_payload_with_empty_string_prompt() {
        let json = r#"{
            "session_id": "test_session",
            "transcript_path": "/path/to/transcript",
            "hook_event_name": "UserPromptSubmit",
            "cwd": "/current/dir",
            "permission_mode": "default",
            "prompt": ""
        }"#;

        let payload: UserPromptSubmitPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.prompt, Some("".to_string()));
        assert_eq!(payload.base.session_id, "test_session");
    }

    #[test]
    fn test_user_prompt_submit_payload_nil_prompt_round_trip() {
        let payload = UserPromptSubmitPayload {
            base: BasePayload {
                session_id: "test_session".to_string(),
                transcript_path: "/path/to/transcript".to_string(),
                hook_event_name: "UserPromptSubmit".to_string(),
                cwd: "/current/dir".to_string(),
                permission_mode: Some("default".to_string()),
            },
            prompt: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"prompt\":null"));

        let deserialized: UserPromptSubmitPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prompt, None);
    }

    // Tests for new HookResult fields: updated_input and decision

    #[test]
    fn test_hook_result_updated_input_serialization() {
        let mut updated_input = HashMap::new();
        updated_input.insert("param1".to_string(), serde_json::json!("modified_value"));
        updated_input.insert("param2".to_string(), serde_json::json!(42));

        let result = HookResult {
            message: Some("Input modified".to_string()),
            blocked: None,
            system_prompt: None,
            updated_input: Some(updated_input),
            decision: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"updated_input\""));
        assert!(json.contains("\"param1\":\"modified_value\""));
        assert!(json.contains("\"param2\":42"));
    }

    #[test]
    fn test_hook_result_updated_input_deserialization() {
        let json = r#"{
            "message": "Input modified",
            "blocked": null,
            "system_prompt": null,
            "updated_input": {
                "file_path": "/new/path.txt",
                "content": "new content"
            },
            "decision": null
        }"#;

        let result: HookResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.message, Some("Input modified".to_string()));
        assert!(result.updated_input.is_some());

        let updated = result.updated_input.unwrap();
        assert_eq!(updated.get("file_path").unwrap(), "/new/path.txt");
        assert_eq!(updated.get("content").unwrap(), "new content");
    }

    #[test]
    fn test_hook_result_decision_serialization() {
        let result = HookResult {
            message: Some("Permission required".to_string()),
            blocked: None,
            system_prompt: None,
            updated_input: None,
            decision: Some("ask".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"decision\":\"ask\""));
    }

    #[test]
    fn test_hook_result_decision_deserialization() {
        let json_ask =
            r#"{"message": null, "blocked": null, "system_prompt": null, "decision": "ask"}"#;
        let result_ask: HookResult = serde_json::from_str(json_ask).unwrap();
        assert_eq!(result_ask.decision, Some("ask".to_string()));

        let json_allow =
            r#"{"message": null, "blocked": null, "system_prompt": null, "decision": "allow"}"#;
        let result_allow: HookResult = serde_json::from_str(json_allow).unwrap();
        assert_eq!(result_allow.decision, Some("allow".to_string()));

        let json_deny =
            r#"{"message": null, "blocked": null, "system_prompt": null, "decision": "deny"}"#;
        let result_deny: HookResult = serde_json::from_str(json_deny).unwrap();
        assert_eq!(result_deny.decision, Some("deny".to_string()));
    }

    #[test]
    fn test_hook_result_ask_with_input_constructor() {
        let mut updated_input = HashMap::new();
        updated_input.insert("file_path".to_string(), serde_json::json!("/safe/path.txt"));
        updated_input.insert("content".to_string(), serde_json::json!("safe content"));

        let result = HookResult::ask_with_input(
            "Confirm operation with modified input",
            updated_input.clone(),
        );

        assert_eq!(
            result.message,
            Some("Confirm operation with modified input".to_string())
        );
        assert_eq!(result.blocked, None);
        assert_eq!(result.system_prompt, None);
        assert_eq!(result.decision, Some("ask".to_string()));
        assert!(result.updated_input.is_some());

        let result_input = result.updated_input.unwrap();
        assert_eq!(result_input.get("file_path").unwrap(), "/safe/path.txt");
        assert_eq!(result_input.get("content").unwrap(), "safe content");
    }

    #[test]
    fn test_hook_result_skip_serializing_if_none() {
        // Test that updated_input and decision are omitted when None
        let result = HookResult::success();
        let json = serde_json::to_string(&result).unwrap();

        // Should NOT contain updated_input or decision fields
        assert!(!json.contains("\"updated_input\""));
        assert!(!json.contains("\"decision\""));

        // Should still contain the other fields
        assert!(json.contains("\"blocked\":false"));
    }

    #[test]
    fn test_hook_result_constructors_initialize_new_fields() {
        // Test success() constructor
        let success = HookResult::success();
        assert_eq!(success.updated_input, None);
        assert_eq!(success.decision, None);

        // Test blocked() constructor
        let blocked = HookResult::blocked("Test blocked");
        assert_eq!(blocked.updated_input, None);
        assert_eq!(blocked.decision, None);

        // Test with_context() constructor
        let with_context = HookResult::with_context("Test context");
        assert_eq!(with_context.updated_input, None);
        assert_eq!(with_context.decision, None);
    }

    #[test]
    fn test_hook_result_combined_fields_serialization() {
        let mut updated_input = HashMap::new();
        updated_input.insert("key".to_string(), serde_json::json!("value"));

        let result = HookResult {
            message: Some("Combined test".to_string()),
            blocked: Some(false),
            system_prompt: Some("Context".to_string()),
            updated_input: Some(updated_input),
            decision: Some("allow".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: HookResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.message, Some("Combined test".to_string()));
        assert_eq!(deserialized.blocked, Some(false));
        assert_eq!(deserialized.system_prompt, Some("Context".to_string()));
        assert_eq!(deserialized.decision, Some("allow".to_string()));
        assert!(deserialized.updated_input.is_some());
    }
}
