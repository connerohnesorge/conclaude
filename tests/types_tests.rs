use conclaude::types::*;
use std::collections::HashMap;

#[test]
fn test_hook_result_success() {
    let result = HookResult::success();
    assert_eq!(result.blocked, Some(false));
    assert_eq!(result.message, None);
}

#[test]
fn test_hook_result_blocked() {
    let result = HookResult::blocked("Test blocking message");
    assert_eq!(result.blocked, Some(true));
    assert_eq!(result.message, Some("Test blocking message".to_string()));
}

#[test]
fn test_validate_base_payload_valid() {
    let valid_base = BasePayload {
        session_id: "test_session".to_string(),
        transcript_path: "/path/to/transcript".to_string(),
        hook_event_name: "PreToolUse".to_string(),
        cwd: "/current/dir".to_string(),
        permission_mode: Some("default".to_string()),
    };
    assert!(validate_base_payload(&valid_base).is_ok());
}

#[test]
fn test_validate_base_payload_missing_session_id() {
    let invalid_base = BasePayload {
        session_id: String::new(),
        transcript_path: "/path/to/transcript".to_string(),
        hook_event_name: "PreToolUse".to_string(),
        cwd: "/current/dir".to_string(),
        permission_mode: Some("default".to_string()),
    };
    let result = validate_base_payload(&invalid_base);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("session_id"));
}

#[test]
fn test_validate_base_payload_missing_transcript_path() {
    let invalid_base = BasePayload {
        session_id: "test_session".to_string(),
        transcript_path: String::new(),
        hook_event_name: "PreToolUse".to_string(),
        cwd: "/current/dir".to_string(),
        permission_mode: Some("default".to_string()),
    };
    let result = validate_base_payload(&invalid_base);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("transcript_path"));
}

#[test]
fn test_validate_base_payload_missing_hook_event_name() {
    let invalid_base = BasePayload {
        session_id: "test_session".to_string(),
        transcript_path: "/path/to/transcript".to_string(),
        hook_event_name: String::new(),
        cwd: "/current/dir".to_string(),
        permission_mode: Some("default".to_string()),
    };
    let result = validate_base_payload(&invalid_base);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("hook_event_name"));
}

#[test]
fn test_compact_trigger_serialization() {
    let trigger = CompactTrigger::Manual;
    let json = serde_json::to_string(&trigger).unwrap();
    assert_eq!(json, "\"manual\"");

    let trigger = CompactTrigger::Auto;
    let json = serde_json::to_string(&trigger).unwrap();
    assert_eq!(json, "\"auto\"");
}

#[test]
fn test_compact_trigger_deserialization() {
    let json = "\"manual\"";
    let trigger: CompactTrigger = serde_json::from_str(json).unwrap();
    matches!(trigger, CompactTrigger::Manual);

    let json = "\"auto\"";
    let trigger: CompactTrigger = serde_json::from_str(json).unwrap();
    matches!(trigger, CompactTrigger::Auto);
}

#[test]
fn test_pre_tool_use_payload_serialization() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        serde_json::Value::String("test.txt".to_string()),
    );

    let payload = PreToolUsePayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "PreToolUse".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        tool_name: "Edit".to_string(),
        tool_input,
        tool_use_id: Some("test-tool-use-id".to_string()),
    };

    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("test_session"));
    assert!(json.contains("Edit"));
    assert!(json.contains("test.txt"));
}

#[test]
fn test_notification_payload_deserialization() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "Notification",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "message": "Test notification",
        "title": "Test Title"
    }"#;

    let payload: NotificationPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.base.session_id, "test_session");
    assert_eq!(payload.message, "Test notification");
    assert_eq!(payload.title, Some("Test Title".to_string()));
}

#[test]
fn test_stop_payload_deserialization() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "Stop",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "stop_hook_active": true
    }"#;

    let payload: StopPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.base.session_id, "test_session");
    assert!(payload.stop_hook_active);
}

#[test]
fn test_user_prompt_submit_payload_deserialization() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "UserPromptSubmit",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "prompt": "Hello Claude"
    }"#;

    let payload: UserPromptSubmitPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.base.session_id, "test_session");
    assert_eq!(payload.prompt, "Hello Claude");
}

#[test]
fn test_pre_compact_payload_deserialization() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "PreCompact",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "trigger": "auto",
        "custom_instructions": null
    }"#;

    let payload: PreCompactPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.base.session_id, "test_session");
    matches!(payload.trigger, CompactTrigger::Auto);
}

#[test]
fn test_session_start_payload_deserialization() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "SessionStart",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "source": "CLI"
    }"#;

    let payload: SessionStartPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.base.session_id, "test_session");
    assert_eq!(payload.source, "CLI");
}

#[test]
fn test_subagent_stop_payload_deserialization() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "SubagentStop",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_id": "coder",
        "agent_transcript_path": "/path/to/agent/transcript"
    }"#;

    let payload: SubagentStopPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.base.session_id, "test_session");
    assert_eq!(payload.agent_id, "coder");
    assert_eq!(payload.agent_transcript_path, "/path/to/agent/transcript");
    assert!(payload.stop_hook_active);
}

#[test]
fn test_validate_subagent_stop_payload_valid() {
    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "coder".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_validate_subagent_stop_payload_empty_agent_id() {
    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: String::new(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_id cannot be empty");
}

#[test]
fn test_validate_subagent_stop_payload_whitespace_agent_id() {
    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "   ".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_id cannot be empty");
}

#[test]
fn test_validate_subagent_stop_payload_empty_agent_transcript_path() {
    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "coder".to_string(),
        agent_transcript_path: String::new(),
    };
    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_transcript_path cannot be empty");
}

#[test]
fn test_validate_subagent_stop_payload_whitespace_agent_transcript_path() {
    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "coder".to_string(),
        agent_transcript_path: "   ".to_string(),
    };
    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_transcript_path cannot be empty");
}

#[test]
fn test_permission_request_payload_deserialization() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "PermissionRequest",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "tool_name": "Bash",
        "tool_input": {"command": "ls -la"}
    }"#;

    let payload: PermissionRequestPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.base.session_id, "test_session");
    assert_eq!(payload.base.transcript_path, "/path/to/transcript");
    assert_eq!(payload.base.hook_event_name, "PermissionRequest");
    assert_eq!(payload.base.cwd, "/current/dir");
    assert_eq!(payload.base.permission_mode, Some("default".to_string()));
    assert_eq!(payload.tool_name, "Bash");
    assert_eq!(
        payload.tool_input.get("command"),
        Some(&serde_json::Value::String("ls -la".to_string()))
    );
}

#[test]
fn test_permission_request_payload_serialization() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("ls -la".to_string()),
    );

    let payload = PermissionRequestPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "PermissionRequest".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        tool_name: "Bash".to_string(),
        tool_input,
    };

    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("test_session"));
    assert!(json.contains("PermissionRequest"));
    assert!(json.contains("Bash"));
    assert!(json.contains("ls -la"));
}

#[test]
fn test_validate_permission_request_payload_valid() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("ls -la".to_string()),
    );

    let payload = PermissionRequestPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "PermissionRequest".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        tool_name: "Bash".to_string(),
        tool_input,
    };
    assert!(validate_permission_request_payload(&payload).is_ok());
}

#[test]
fn test_validate_permission_request_payload_empty_tool_name() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("ls -la".to_string()),
    );

    let payload = PermissionRequestPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "PermissionRequest".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        tool_name: String::new(),
        tool_input,
    };
    let result = validate_permission_request_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "tool_name cannot be empty");
}

#[test]
fn test_validate_permission_request_payload_whitespace_tool_name() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("ls -la".to_string()),
    );

    let payload = PermissionRequestPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "PermissionRequest".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        tool_name: "   ".to_string(),
        tool_input,
    };
    let result = validate_permission_request_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "tool_name cannot be empty");
}

