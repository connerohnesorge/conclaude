use conclaude::types::*;
use std::collections::HashMap;

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
    assert_eq!(payload.prompt, Some("Hello Claude".to_string()));
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
    assert!(matches!(payload.trigger, CompactTrigger::Auto));
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
fn test_validate_subagent_stop_payload_invalid_base() {
    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: String::new(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "coder".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("session_id"));
}

#[test]
fn test_validate_subagent_stop_payload_agent_id_with_leading_trailing_spaces() {
    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "  coder  ".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    assert!(validate_subagent_stop_payload(&payload).is_ok());
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

// ============================================================================
// SubagentStart Payload Validation Tests
// ============================================================================

#[test]
fn test_validate_subagent_start_payload_valid() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "coder".to_string(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    assert!(validate_subagent_start_payload(&payload).is_ok());
}

#[test]
fn test_validate_subagent_start_payload_empty_agent_id() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: String::new(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_id cannot be empty");
}

#[test]
fn test_validate_subagent_start_payload_whitespace_agent_id() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "   ".to_string(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_id cannot be empty");
}

#[test]
fn test_validate_subagent_start_payload_empty_subagent_type() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "coder".to_string(),
        subagent_type: String::new(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "subagent_type cannot be empty");
}

#[test]
fn test_validate_subagent_start_payload_whitespace_subagent_type() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "coder".to_string(),
        subagent_type: "   ".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "subagent_type cannot be empty");
}

#[test]
fn test_validate_subagent_start_payload_empty_agent_transcript_path() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "coder".to_string(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: String::new(),
    };
    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_transcript_path cannot be empty");
}

#[test]
fn test_validate_subagent_start_payload_whitespace_agent_transcript_path() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "coder".to_string(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: "   ".to_string(),
    };
    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "agent_transcript_path cannot be empty");
}

#[test]
fn test_validate_subagent_start_payload_invalid_base() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: String::new(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "coder".to_string(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("session_id"));
}

#[test]
fn test_validate_subagent_start_payload_agent_id_with_leading_trailing_spaces() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/current/dir".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "  coder  ".to_string(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: "/path/to/agent/transcript".to_string(),
    };
    assert!(validate_subagent_start_payload(&payload).is_ok());
}

#[test]
fn test_subagent_start_payload_deserialize_from_valid_json() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "SubagentStart",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "agent_id": "coder",
        "subagent_type": "implementation",
        "agent_transcript_path": "/path/to/agent/transcript"
    }"#;

    let payload: SubagentStartPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.agent_id, "coder");
    assert_eq!(payload.subagent_type, "implementation");
    assert_eq!(payload.agent_transcript_path, "/path/to/agent/transcript");
    assert_eq!(payload.base.session_id, "test_session");
}

#[test]
fn test_subagent_start_payload_deserialize_missing_agent_id() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "SubagentStart",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "subagent_type": "implementation",
        "agent_transcript_path": "/path/to/agent/transcript"
    }"#;

    let result: Result<SubagentStartPayload, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_subagent_start_payload_deserialize_missing_subagent_type() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "SubagentStart",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "agent_id": "coder",
        "agent_transcript_path": "/path/to/agent/transcript"
    }"#;

    let result: Result<SubagentStartPayload, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_subagent_start_payload_deserialize_missing_agent_transcript_path() {
    let json = r#"{
        "session_id": "test_session",
        "transcript_path": "/path/to/transcript",
        "hook_event_name": "SubagentStart",
        "cwd": "/current/dir",
        "permission_mode": "default",
        "agent_id": "coder",
        "subagent_type": "implementation"
    }"#;

    let result: Result<SubagentStartPayload, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_subagent_start_payload_serialization_round_trip() {
    let payload = SubagentStartPayload {
        base: BasePayload {
            session_id: "test_session_789".to_string(),
            transcript_path: "/tmp/session_transcript.jsonl".to_string(),
            hook_event_name: "SubagentStart".to_string(),
            cwd: "/home/user/project".to_string(),
            permission_mode: Some("default".to_string()),
        },
        agent_id: "coder".to_string(),
        subagent_type: "implementation".to_string(),
        agent_transcript_path: "/tmp/coder_transcript.jsonl".to_string(),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&payload).expect("Failed to serialize");

    // Deserialize back
    let deserialized: SubagentStartPayload =
        serde_json::from_str(&json_str).expect("Failed to deserialize");

    // Verify round-trip preservation
    assert_eq!(deserialized.base.session_id, payload.base.session_id);
    assert_eq!(deserialized.agent_id, payload.agent_id);
    assert_eq!(deserialized.subagent_type, payload.subagent_type);
    assert_eq!(
        deserialized.agent_transcript_path,
        payload.agent_transcript_path
    );
}
