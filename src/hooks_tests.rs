use crate::config::{CommandAction, ConclaudeConfig};
use crate::hooks::*;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn test_is_system_event_hook() {
    // Test system event hooks
    assert!(is_system_event_hook("SessionStart"));
    assert!(is_system_event_hook("SessionEnd"));
    assert!(is_system_event_hook("UserPromptSubmit"));
    assert!(is_system_event_hook("SubagentStart"));
    assert!(is_system_event_hook("SubagentStop"));
    assert!(is_system_event_hook("PreCompact"));

    // Test non-system event hooks
    assert!(!is_system_event_hook("PreToolUse"));
    assert!(!is_system_event_hook("PostToolUse"));
    assert!(!is_system_event_hook("Notification"));
    assert!(!is_system_event_hook("Stop"));
}

#[test]
fn test_truncate_output_no_truncation() {
    // Test no truncation needed (fewer lines than limit)
    let output = "line1\nline2\nline3";
    let (truncated, is_truncated, omitted) = truncate_output(output, 10);
    assert_eq!(truncated, "line1\nline2\nline3");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);

    // Test exact limit
    let output = "line1\nline2\nline3";
    let (truncated, is_truncated, omitted) = truncate_output(output, 3);
    assert_eq!(truncated, "line1\nline2\nline3");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);

    // Test single line
    let output = "single line";
    let (truncated, is_truncated, omitted) = truncate_output(output, 1);
    assert_eq!(truncated, "single line");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);

    // Test large limit
    let output = "line1\nline2";
    let (truncated, is_truncated, omitted) = truncate_output(output, 10000);
    assert_eq!(truncated, "line1\nline2");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);
}

#[test]
fn test_truncate_output_with_truncation() {
    let output = "line1\nline2\nline3\nline4\nline5";
    let (truncated, is_truncated, omitted) = truncate_output(output, 2);
    assert_eq!(truncated, "line1\nline2");
    assert!(is_truncated);
    assert_eq!(omitted, 3);
}

#[test]
fn test_truncate_output_empty() {
    let output = "";
    let (truncated, is_truncated, omitted) = truncate_output(output, 10);
    assert_eq!(truncated, "");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);
}

#[test]
fn test_truncate_output_multiple_lines_just_over_limit() {
    let output = "line1\nline2\nline3\nline4\nline5\nline6";
    let (truncated, is_truncated, omitted) = truncate_output(output, 5);
    assert_eq!(truncated, "line1\nline2\nline3\nline4\nline5");
    assert!(is_truncated);
    assert_eq!(omitted, 1);
}

#[test]
fn test_truncate_output_preserves_content() {
    let output = "Line with special chars: !@#$%^&*()\nAnother line\n\nEmpty line above";
    let (truncated, is_truncated, omitted) = truncate_output(output, 2);
    assert_eq!(
        truncated,
        "Line with special chars: !@#$%^&*()\nAnother line"
    );
    assert!(is_truncated);
    assert_eq!(omitted, 2);
}

#[test]
fn test_collect_stop_commands_with_output_config() {
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![
                StopCommand {
                    run: Some("echo hello".to_string()),
                    rg: None,
                    action: CommandAction::Block,
                    message: Some("Custom message".to_string()),
                    show_command: None,
                    show_stdout: Some(true),
                    show_stderr: Some(false),
                    max_output_lines: Some(10),
                    timeout: None,
                    notify_per_command: None,
                },
                StopCommand {
                    run: Some("ls -la".to_string()),
                    rg: None,
                    action: CommandAction::Block,
                    message: None,
                    show_command: None,
                    show_stdout: Some(false),
                    show_stderr: Some(true),
                    max_output_lines: Some(5),
                    timeout: None,
                    notify_per_command: None,
                },
            ],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config).unwrap();
    assert_eq!(collected.shell_commands.len(), 2);

    assert_eq!(collected.shell_commands[0].command, "echo hello");
    assert!(collected.shell_commands[0].show_stdout);
    assert!(!collected.shell_commands[0].show_stderr);
    assert_eq!(collected.shell_commands[0].max_output_lines, Some(10));
    assert_eq!(collected.shell_commands[0].message, Some("Custom message".to_string()));

    assert_eq!(collected.shell_commands[1].command, "ls -la");
    assert!(!collected.shell_commands[1].show_stdout);
    assert!(collected.shell_commands[1].show_stderr);
    assert_eq!(collected.shell_commands[1].max_output_lines, Some(5));
    assert_eq!(collected.shell_commands[1].message, None);
}

#[test]
fn test_collect_stop_commands_default_values() {
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![StopCommand {
                run: Some("echo test".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None,
            }],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config).unwrap();
    assert_eq!(collected.shell_commands.len(), 1);

    // Defaults should be false for show flags and None for max_output_lines
    assert!(!collected.shell_commands[0].show_stdout);
    assert!(!collected.shell_commands[0].show_stderr);
    assert_eq!(collected.shell_commands[0].max_output_lines, None);
}

#[test]
fn test_extract_bash_command_valid() {
    let mut tool_input = std::collections::HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("echo hello".to_string()),
    );
    assert_eq!(
        extract_bash_command(&tool_input),
        Some("echo hello".to_string())
    );
}

#[test]
fn test_extract_bash_command_missing() {
    let tool_input: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    assert_eq!(extract_bash_command(&tool_input), None);
}

#[test]
fn test_extract_bash_command_empty() {
    let mut tool_input = std::collections::HashMap::new();
    tool_input.insert("command".to_string(), Value::String("".to_string()));
    assert_eq!(extract_bash_command(&tool_input), None);
}

#[test]
fn test_extract_bash_command_whitespace_only() {
    let mut tool_input = std::collections::HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("   \n\t   ".to_string()),
    );
    assert_eq!(extract_bash_command(&tool_input), None);
}

#[test]
fn test_extract_bash_command_trims_whitespace() {
    let mut tool_input = std::collections::HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("  echo test  ".to_string()),
    );
    assert_eq!(
        extract_bash_command(&tool_input),
        Some("echo test".to_string())
    );
}

#[test]
fn test_extract_bash_command_non_string_value() {
    let mut tool_input = std::collections::HashMap::new();
    tool_input.insert("command".to_string(), Value::Number(42.into()));
    assert_eq!(extract_bash_command(&tool_input), None);
}

// Tests for subagent stop pattern matching
#[test]
fn test_match_subagent_patterns_prefix_glob() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "test*".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo test".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    let matches = match_subagent_patterns("tester", &config).unwrap();
    assert_eq!(matches, vec!["test*"]);

    let matches = match_subagent_patterns("test-runner", &config).unwrap();
    assert_eq!(matches, vec!["test*"]);

    let matches = match_subagent_patterns("testing", &config).unwrap();
    assert_eq!(matches, vec!["test*"]);

    // Should NOT match runner-test
    let matches = match_subagent_patterns("runner-test", &config).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_match_subagent_patterns_suffix_glob() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "*coder".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo coder".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    let matches = match_subagent_patterns("coder", &config).unwrap();
    assert_eq!(matches, vec!["*coder"]);

    let matches = match_subagent_patterns("auto-coder", &config).unwrap();
    assert_eq!(matches, vec!["*coder"]);

    let matches = match_subagent_patterns("smart-coder", &config).unwrap();
    assert_eq!(matches, vec!["*coder"]);

    // Should NOT match coder-agent
    let matches = match_subagent_patterns("coder-agent", &config).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_match_subagent_patterns_character_class() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "agent_[0-9]*".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo agent".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    let matches = match_subagent_patterns("agent_1", &config).unwrap();
    assert_eq!(matches, vec!["agent_[0-9]*"]);

    let matches = match_subagent_patterns("agent_2x", &config).unwrap();
    assert_eq!(matches, vec!["agent_[0-9]*"]);

    let matches = match_subagent_patterns("agent_99test", &config).unwrap();
    assert_eq!(matches, vec!["agent_[0-9]*"]);

    // Should NOT match agent_x or agent
    let matches = match_subagent_patterns("agent_x", &config).unwrap();
    assert!(matches.is_empty());

    let matches = match_subagent_patterns("agent", &config).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_match_subagent_patterns_multiple_matches() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "*".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo all".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo coder".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );
    commands.insert(
        "*coder".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo suffix-coder".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    // "coder" should match all three patterns
    let matches = match_subagent_patterns("coder", &config).unwrap();
    // Wildcard first, then sorted other matches
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0], "*");
    // The remaining matches should be sorted
    assert!(matches.contains(&"coder"));
    assert!(matches.contains(&"*coder"));
}

#[test]
fn test_match_subagent_patterns_wildcard_first() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo coder".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );
    commands.insert(
        "*".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo all".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    let matches = match_subagent_patterns("coder", &config).unwrap();
    // Wildcard should always be first
    assert_eq!(matches[0], "*");
    assert_eq!(matches[1], "coder");
}

#[test]
fn test_match_subagent_patterns_no_match() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo coder".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );
    commands.insert(
        "tester".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo tester".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    let matches = match_subagent_patterns("unknown-agent", &config).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_match_subagent_patterns_empty_config() {
    use crate::config::SubagentStopConfig;

    let config = SubagentStopConfig::default();

    let matches = match_subagent_patterns("coder", &config).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_build_subagent_env_vars() {
    use crate::types::{BasePayload, SubagentStopPayload};

    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "test-session-123".to_string(),
            transcript_path: "/path/to/main/transcript.json".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/home/user/project".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "coder".to_string(),
        agent_transcript_path: "/path/to/agent/transcript.json".to_string(),
    };

    let env_vars = build_subagent_env_vars(&payload, Path::new("/test/config"), Some("coder"));

    assert_eq!(
        env_vars.get("CONCLAUDE_AGENT_ID"),
        Some(&"coder".to_string())
    );
    assert_eq!(
        env_vars.get("CONCLAUDE_AGENT_NAME"),
        Some(&"coder".to_string())
    );
    assert_eq!(
        env_vars.get("CONCLAUDE_AGENT_TRANSCRIPT_PATH"),
        Some(&"/path/to/agent/transcript.json".to_string())
    );
    assert_eq!(
        env_vars.get("CONCLAUDE_SESSION_ID"),
        Some(&"test-session-123".to_string())
    );
    assert_eq!(
        env_vars.get("CONCLAUDE_TRANSCRIPT_PATH"),
        Some(&"/path/to/main/transcript.json".to_string())
    );
    assert_eq!(
        env_vars.get("CONCLAUDE_HOOK_EVENT"),
        Some(&"SubagentStop".to_string())
    );
    assert_eq!(
        env_vars.get("CONCLAUDE_CWD"),
        Some(&"/home/user/project".to_string())
    );
    assert_eq!(
        env_vars.get("CONCLAUDE_CONFIG_DIR"),
        Some(&"/test/config".to_string())
    );

    // Verify CONCLAUDE_PAYLOAD_JSON contains the full payload
    let payload_json = env_vars.get("CONCLAUDE_PAYLOAD_JSON").unwrap();
    assert!(payload_json.contains("\"agent_id\":\"coder\""));
    assert!(payload_json.contains("\"session_id\":\"test-session-123\""));
}

#[test]
fn test_build_subagent_env_vars_all_expected_keys() {
    use crate::types::{BasePayload, SubagentStopPayload};

    let payload = SubagentStopPayload {
        base: BasePayload {
            session_id: "session".to_string(),
            transcript_path: "/transcript".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/cwd".to_string(),
            permission_mode: None,
        },
        stop_hook_active: true,
        agent_id: "tester".to_string(),
        agent_transcript_path: "/agent/transcript".to_string(),
    };

    let env_vars = build_subagent_env_vars(&payload, Path::new("."), None);

    // Verify all expected keys are present
    let expected_keys = [
        "CONCLAUDE_AGENT_ID",
        "CONCLAUDE_AGENT_NAME",
        "CONCLAUDE_AGENT_TRANSCRIPT_PATH",
        "CONCLAUDE_SESSION_ID",
        "CONCLAUDE_TRANSCRIPT_PATH",
        "CONCLAUDE_HOOK_EVENT",
        "CONCLAUDE_CWD",
        "CONCLAUDE_CONFIG_DIR",
        "CONCLAUDE_PAYLOAD_JSON",
    ];

    for key in &expected_keys {
        assert!(env_vars.contains_key(*key), "Missing expected key: {}", key);
    }

    assert_eq!(env_vars.len(), expected_keys.len());

    // When agent_name is None, it should fall back to agent_id
    assert_eq!(
        env_vars.get("CONCLAUDE_AGENT_NAME"),
        Some(&"tester".to_string())
    );
}

#[test]
fn test_collect_subagent_stop_commands_single_pattern() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![
            SubagentStopCommand {
                run: Some("echo first".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: Some("First command".to_string()),
                show_command: None,
                show_stdout: Some(true),
                show_stderr: Some(false),
                max_output_lines: Some(10),
                timeout: None,
                notify_per_command: None,
            },
            SubagentStopCommand {
                run: Some("echo second".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None,
            },
        ],
    );

    let config = SubagentStopConfig { commands };
    let matching_patterns = vec!["coder"];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();

    assert_eq!(collected.shell_commands.len(), 2);
    assert_eq!(collected.shell_commands[0].command, "echo first");
    assert!(collected.shell_commands[0].show_stdout);
    assert!(!collected.shell_commands[0].show_stderr);
    assert_eq!(collected.shell_commands[0].max_output_lines, Some(10));
    assert_eq!(collected.shell_commands[0].message, Some("First command".to_string()));

    assert_eq!(collected.shell_commands[1].command, "echo second");
    assert!(!collected.shell_commands[1].show_stdout);
    assert!(!collected.shell_commands[1].show_stderr);
    assert_eq!(collected.shell_commands[1].max_output_lines, None);
    assert_eq!(collected.shell_commands[1].message, None);
}

#[test]
fn test_collect_subagent_stop_commands_multiple_patterns() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "*".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo wildcard".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo coder".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };
    // Wildcard first, then specific pattern (as match_subagent_patterns returns)
    let matching_patterns = vec!["*", "coder"];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();

    assert_eq!(collected.shell_commands.len(), 2);
    // Commands should be in order of patterns
    assert_eq!(collected.shell_commands[0].command, "echo wildcard");
    assert_eq!(collected.shell_commands[1].command, "echo coder");
}

#[test]
fn test_collect_subagent_stop_commands_no_matching_patterns() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo coder".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }],
    );

    let config = SubagentStopConfig { commands };
    let matching_patterns: Vec<&str> = vec![];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();
    assert!(collected.shell_commands.is_empty());
}

// ============================================================================
// Tests for Per-Command Notifications Feature
// ============================================================================

#[test]
fn test_collect_stop_commands_with_notify_per_command_true() {
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![StopCommand {
                run: Some("echo test".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: Some("Test message".to_string()),
                show_command: Some(true),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(true),
            }],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config).unwrap();
    assert_eq!(collected.shell_commands.len(), 1);

    assert_eq!(collected.shell_commands[0].command, "echo test");
    assert!(collected.shell_commands[0].show_command);
    assert!(collected.shell_commands[0].notify_per_command);
    assert_eq!(collected.shell_commands[0].message, Some("Test message".to_string()));
}

#[test]
fn test_collect_stop_commands_with_notify_per_command_false() {
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![StopCommand {
                run: Some("echo test".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: Some(false),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(false),
            }],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config).unwrap();
    assert_eq!(collected.shell_commands.len(), 1);

    assert_eq!(collected.shell_commands[0].command, "echo test");
    assert!(!collected.shell_commands[0].show_command);
    assert!(!collected.shell_commands[0].notify_per_command);
}

#[test]
fn test_collect_stop_commands_notify_per_command_defaults_to_false() {
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![StopCommand {
                run: Some("echo test".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None, // Not specified - should default to false
            }],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config).unwrap();
    assert_eq!(collected.shell_commands.len(), 1);

    // notify_per_command should default to false
    assert!(!collected.shell_commands[0].notify_per_command);
}

#[test]
fn test_collect_stop_commands_mixed_notify_per_command_settings() {
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![
                StopCommand {
                    run: Some("echo first".to_string()),
                    rg: None,
                    action: CommandAction::Block,
                    message: None,
                    show_command: Some(true),
                    show_stdout: None,
                    show_stderr: None,
                    max_output_lines: None,
                    timeout: None,
                    notify_per_command: Some(true),
                },
                StopCommand {
                    run: Some("echo second".to_string()),
                    rg: None,
                    action: CommandAction::Block,
                    message: None,
                    show_command: Some(false),
                    show_stdout: None,
                    show_stderr: None,
                    max_output_lines: None,
                    timeout: None,
                    notify_per_command: Some(false),
                },
                StopCommand {
                    run: Some("echo third".to_string()),
                    rg: None,
                    action: CommandAction::Block,
                    message: None,
                    show_command: None,
                    show_stdout: None,
                    show_stderr: None,
                    max_output_lines: None,
                    timeout: None,
                    notify_per_command: None, // Should default to false
                },
            ],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config).unwrap();
    assert_eq!(collected.shell_commands.len(), 3);

    // First command has notify_per_command: true
    assert_eq!(collected.shell_commands[0].command, "echo first");
    assert!(collected.shell_commands[0].show_command);
    assert!(collected.shell_commands[0].notify_per_command);

    // Second command has notify_per_command: false
    assert_eq!(collected.shell_commands[1].command, "echo second");
    assert!(!collected.shell_commands[1].show_command);
    assert!(!collected.shell_commands[1].notify_per_command);

    // Third command should default to notify_per_command: false
    assert_eq!(collected.shell_commands[2].command, "echo third");
    assert!(!collected.shell_commands[2].notify_per_command);
}

#[test]
fn test_collect_subagent_stop_commands_with_notify_per_command() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![
            SubagentStopCommand {
                run: Some("echo coder first".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: Some(true),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(true),
            },
            SubagentStopCommand {
                run: Some("echo coder second".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: Some(false),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(false),
            },
        ],
    );

    let config = SubagentStopConfig { commands };
    let matching_patterns = vec!["coder"];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();

    assert_eq!(collected.shell_commands.len(), 2);

    // First command has notify_per_command: true
    assert_eq!(collected.shell_commands[0].command, "echo coder first");
    assert!(collected.shell_commands[0].show_command);
    assert!(collected.shell_commands[0].notify_per_command);

    // Second command has notify_per_command: false
    assert_eq!(collected.shell_commands[1].command, "echo coder second");
    assert!(!collected.shell_commands[1].show_command);
    assert!(!collected.shell_commands[1].notify_per_command);
}

#[test]
fn test_collect_subagent_stop_commands_notify_per_command_defaults_to_false() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "tester".to_string(),
        vec![SubagentStopCommand {
            run: Some("echo test".to_string()),
            rg: None,
            action: CommandAction::Block,
            message: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None, // Not specified - should default to false
        }],
    );

    let config = SubagentStopConfig { commands };
    let matching_patterns = vec!["tester"];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();

    assert_eq!(collected.shell_commands.len(), 1);
    assert_eq!(collected.shell_commands[0].command, "echo test");
    assert!(!collected.shell_commands[0].notify_per_command);
}

// Tests for notification filter logic
// Note: These tests verify that the notify_per_command flag is correctly passed through
// the configuration pipeline. The actual notification filtering (showErrors, showSuccess)
// is handled by the send_notification() function which checks the NotificationsConfig.
// These tests document that per-command notifications respect the same filtering logic.

#[test]
fn test_notify_per_command_respects_show_command_flag() {
    use crate::config::StopCommand;

    // Test that notify_per_command works with show_command: true
    let config_show_command = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![StopCommand {
                run: Some("echo test".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: Some(true),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(true),
            }],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config_show_command).unwrap();
    assert_eq!(collected.shell_commands.len(), 1);
    assert!(collected.shell_commands[0].show_command);
    assert!(collected.shell_commands[0].notify_per_command);

    // Test that notify_per_command works with show_command: false
    let config_hide_command = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![StopCommand {
                run: Some("echo test".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: Some(false),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(true),
            }],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config_hide_command).unwrap();
    assert_eq!(collected.shell_commands.len(), 1);
    assert!(!collected.shell_commands[0].show_command);
    assert!(collected.shell_commands[0].notify_per_command);
    // The implementation will show "Running command" instead of "Running: {command}"
}

#[test]
fn test_per_command_notification_flag_propagation() {
    // This test verifies that the notify_per_command flag is correctly
    // propagated from the config to the StopCommandConfig struct,
    // which is used by execute_stop_commands to determine whether to send notifications.
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![
                StopCommand {
                    run: Some("echo with-notifications".to_string()),
                    rg: None,
                    action: CommandAction::Block,
                    message: None,
                    show_command: Some(true),
                    show_stdout: None,
                    show_stderr: None,
                    max_output_lines: None,
                    timeout: None,
                    notify_per_command: Some(true),
                },
                StopCommand {
                    run: Some("echo without-notifications".to_string()),
                    rg: None,
                    action: CommandAction::Block,
                    message: None,
                    show_command: Some(true),
                    show_stdout: None,
                    show_stderr: None,
                    max_output_lines: None,
                    timeout: None,
                    notify_per_command: Some(false),
                },
            ],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let collected = collect_stop_commands(&config).unwrap();
    assert_eq!(collected.shell_commands.len(), 2);

    // Verify first command has notifications enabled
    assert_eq!(collected.shell_commands[0].command, "echo with-notifications");
    assert!(collected.shell_commands[0].notify_per_command);

    // Verify second command has notifications disabled
    assert_eq!(collected.shell_commands[1].command, "echo without-notifications");
    assert!(!collected.shell_commands[1].notify_per_command);
}

#[test]
fn test_subagent_stop_notify_per_command_with_show_command() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![
            SubagentStopCommand {
                run: Some("echo visible".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: Some(true),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(true),
            },
            SubagentStopCommand {
                run: Some("echo hidden".to_string()),
                rg: None,
                action: CommandAction::Block,
                message: None,
                show_command: Some(false),
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(true),
            },
        ],
    );

    let config = SubagentStopConfig { commands };
    let matching_patterns = vec!["coder"];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();

    assert_eq!(collected.shell_commands.len(), 2);

    // First command shows command name in notifications
    assert_eq!(collected.shell_commands[0].command, "echo visible");
    assert!(collected.shell_commands[0].show_command);
    assert!(collected.shell_commands[0].notify_per_command);

    // Second command hides command name in notifications
    assert_eq!(collected.shell_commands[1].command, "echo hidden");
    assert!(!collected.shell_commands[1].show_command);
    assert!(collected.shell_commands[1].notify_per_command);
}

#[cfg(test)]
mod prompt_context_tests {
    use super::*;
    use crate::config::ContextInjectionRule;
    use std::fs;
    use tempfile::TempDir;

    // Tests for Task 4.2: Regex pattern matching

    #[test]
    fn test_regex_pattern_matching() {
        // Test 1: Simple string pattern matching
        let simple_rule = ContextInjectionRule {
            pattern: "sidebar".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        let simple_regex = compile_rule_pattern(&simple_rule).unwrap();
        assert!(
            simple_regex.is_match("update the sidebar"),
            "Should match 'sidebar' in phrase"
        );
        assert!(
            simple_regex.is_match("sidebar component"),
            "Should match 'sidebar' at start"
        );
        assert!(
            !simple_regex.is_match("side bar"),
            "Should not match 'side bar' (two words)"
        );
        assert!(
            !simple_regex.is_match("update the navigation"),
            "Should not match unrelated text"
        );

        // Test 2: Alternation pattern matching
        let alt_rule = ContextInjectionRule {
            pattern: "auth|login|authentication".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        let alt_regex = compile_rule_pattern(&alt_rule).unwrap();
        assert!(
            alt_regex.is_match("fix auth bug"),
            "Should match 'auth' alternative"
        );
        assert!(
            alt_regex.is_match("update login page"),
            "Should match 'login' alternative"
        );
        assert!(
            alt_regex.is_match("add authentication"),
            "Should match 'authentication' alternative"
        );
        assert!(
            !alt_regex.is_match("update the navbar"),
            "Should not match unrelated text"
        );

        // Test 3: Multiple patterns - both match
        let sidebar_regex = compile_rule_pattern(&simple_rule).unwrap();
        let auth_rule = ContextInjectionRule {
            pattern: "auth".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        let auth_regex = compile_rule_pattern(&auth_rule).unwrap();
        assert!(
            sidebar_regex.is_match("update the auth sidebar"),
            "Sidebar pattern should match"
        );
        assert!(
            auth_regex.is_match("update the auth sidebar"),
            "Auth pattern should match"
        );

        // Test 4: Multiple patterns - only one matches
        assert!(
            sidebar_regex.is_match("update the sidebar"),
            "Sidebar should match"
        );
        assert!(
            !auth_regex.is_match("update the sidebar"),
            "Auth should not match"
        );

        // Test 5: Multiple patterns - none match
        assert!(
            !sidebar_regex.is_match("update the navigation"),
            "Sidebar should not match"
        );
        assert!(
            !auth_regex.is_match("update the navigation"),
            "Auth should not match"
        );

        // Test 6: Invalid regex patterns return None
        let invalid_bracket = ContextInjectionRule {
            pattern: "[invalid".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        assert!(
            compile_rule_pattern(&invalid_bracket).is_none(),
            "Invalid bracket should return None"
        );

        let invalid_paren = ContextInjectionRule {
            pattern: "(unclosed".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        assert!(
            compile_rule_pattern(&invalid_paren).is_none(),
            "Unclosed paren should return None"
        );
    }

    #[test]
    fn test_regex_case_insensitive_with_flag_in_pattern() {
        let rule = ContextInjectionRule {
            pattern: "(?i)database".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };

        let regex = compile_rule_pattern(&rule).unwrap();
        assert!(regex.is_match("DATABASE connection"));
        assert!(regex.is_match("database query"));
        assert!(regex.is_match("Database setup"));
    }

    #[test]
    fn test_regex_case_insensitive_with_config_field() {
        let rule = ContextInjectionRule {
            pattern: "database".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: Some(true),
        };

        let regex = compile_rule_pattern(&rule).unwrap();
        assert!(regex.is_match("DATABASE connection"));
        assert!(regex.is_match("database query"));
        assert!(regex.is_match("Database setup"));
    }

    #[test]
    fn test_expand_file_references_valid_file() {
        // Create temporary directory and file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "This is test content").unwrap();

        let prompt = format!("Read @{}", file_path.file_name().unwrap().to_str().unwrap());
        let expanded = expand_file_references(&prompt, temp_dir.path());

        assert_eq!(expanded, "Read This is test content");
    }

    #[test]
    fn test_expand_file_references_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let prompt = "Read @missing-file.md";
        let expanded = expand_file_references(prompt, temp_dir.path());

        // Missing file reference should be left as-is
        assert_eq!(expanded, "Read @missing-file.md");
    }

    #[test]
    fn test_expand_file_references_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file1, "Content 1").unwrap();
        fs::write(&file2, "Content 2").unwrap();

        let prompt = "Read @file1.md and @file2.md";
        let expanded = expand_file_references(prompt, temp_dir.path());

        assert_eq!(expanded, "Read Content 1 and Content 2");
    }

    #[test]
    fn test_expand_file_references_no_references() {
        let temp_dir = TempDir::new().unwrap();
        let prompt = "This is a normal prompt without file references";
        let expanded = expand_file_references(prompt, temp_dir.path());

        // Prompt should be unchanged
        assert_eq!(expanded, "This is a normal prompt without file references");
    }
}

#[cfg(test)]
mod agent_session_tests {
    use super::*;

    #[test]
    fn test_read_agent_from_session_file_not_exists() {
        // Should return "main" when no session file exists
        let result = read_agent_from_session_file("nonexistent-session-id-12345");
        assert_eq!(result, "main");
    }

    #[test]
    fn test_write_and_read_agent_session_file() {
        let session_id = format!("test-session-{}", std::process::id());

        // Write session file
        write_agent_session_file(&session_id, "coder").expect("Failed to write session file");

        // Read it back
        let result = read_agent_from_session_file(&session_id);
        assert_eq!(result, "coder");

        // Cleanup the file
        let path = get_agent_session_file_path(&session_id);
        let _ = fs::remove_file(&path);

        // Verify reading after cleanup returns "main"
        let result_after = read_agent_from_session_file(&session_id);
        assert_eq!(result_after, "main");
    }
}

#[cfg(test)]
mod user_prompt_submit_command_tests {
    use super::*;
    use crate::config::UserPromptSubmitCommand;
    use crate::types::{BasePayload, UserPromptSubmitPayload};
    use tempfile::TempDir;

    // Test: collect_user_prompt_submit_commands() filters by regex pattern
    #[test]
    fn test_collect_commands_filters_by_regex_pattern() {
        let commands = vec![
            UserPromptSubmitCommand {
                run: "echo deploy".to_string(),
                pattern: Some("deploy|release".to_string()),
                case_insensitive: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None,
            },
            UserPromptSubmitCommand {
                run: "echo test".to_string(),
                pattern: Some("test".to_string()),
                case_insensitive: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None,
            },
        ];

        // Should match only deploy command
        let result =
            collect_user_prompt_submit_commands(&commands, "let's deploy to production").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].command.contains("deploy"));

        // Should match only test command
        let result = collect_user_prompt_submit_commands(&commands, "run the test suite").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].command.contains("test"));

        // Should match no commands
        let result = collect_user_prompt_submit_commands(&commands, "fix the bug").unwrap();
        assert_eq!(result.len(), 0);

        // Should match deploy via "release" pattern
        let result = collect_user_prompt_submit_commands(&commands, "prepare for release").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].command.contains("deploy"));
    }

    // Test: build_user_prompt_submit_env_vars() produces correct variables
    #[test]
    fn test_build_env_vars_produces_correct_variables() {
        let temp_dir = TempDir::new().unwrap();
        let payload = UserPromptSubmitPayload {
            base: BasePayload {
                session_id: "test-session-123".to_string(),
                cwd: "/home/user/project".to_string(),
                transcript_path: "/tmp/transcript.jsonl".to_string(),
                hook_event_name: "UserPromptSubmit".to_string(),
                permission_mode: Some("default".to_string()),
            },
            prompt: "update the sidebar component".to_string(),
        };

        let env_vars = build_user_prompt_submit_env_vars(&payload, temp_dir.path());

        // Verify all expected environment variables are set
        assert_eq!(
            env_vars.get("CONCLAUDE_USER_PROMPT"),
            Some(&"update the sidebar component".to_string())
        );
        assert_eq!(
            env_vars.get("CONCLAUDE_SESSION_ID"),
            Some(&"test-session-123".to_string())
        );
        assert_eq!(
            env_vars.get("CONCLAUDE_CWD"),
            Some(&"/home/user/project".to_string())
        );
        assert_eq!(
            env_vars.get("CONCLAUDE_HOOK_EVENT"),
            Some(&"UserPromptSubmit".to_string())
        );
        assert!(env_vars.contains_key("CONCLAUDE_CONFIG_DIR"));
    }

    // Test: Command with no pattern runs for all prompts
    #[test]
    fn test_command_without_pattern_runs_for_all_prompts() {
        let commands = vec![UserPromptSubmitCommand {
            run: "echo always".to_string(),
            pattern: None, // No pattern = match all
            case_insensitive: None,
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }];

        // Should match any prompt
        let result = collect_user_prompt_submit_commands(&commands, "random prompt").unwrap();
        assert_eq!(result.len(), 1);

        let result = collect_user_prompt_submit_commands(&commands, "deploy now").unwrap();
        assert_eq!(result.len(), 1);

        let result = collect_user_prompt_submit_commands(&commands, "test the code").unwrap();
        assert_eq!(result.len(), 1);

        let result = collect_user_prompt_submit_commands(&commands, "").unwrap();
        assert_eq!(result.len(), 1);
    }

    // Test: Case-insensitive pattern matching works
    #[test]
    fn test_case_insensitive_pattern_matching() {
        let commands = vec![UserPromptSubmitCommand {
            run: "echo database".to_string(),
            pattern: Some("database".to_string()),
            case_insensitive: Some(true),
            show_command: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
            notify_per_command: None,
        }];

        // Should match with different cases
        let result =
            collect_user_prompt_submit_commands(&commands, "update DATABASE config").unwrap();
        assert_eq!(result.len(), 1);

        let result = collect_user_prompt_submit_commands(&commands, "database query").unwrap();
        assert_eq!(result.len(), 1);

        let result = collect_user_prompt_submit_commands(&commands, "Database setup").unwrap();
        assert_eq!(result.len(), 1);

        let result = collect_user_prompt_submit_commands(&commands, "DaTaBaSe").unwrap();
        assert_eq!(result.len(), 1);

        // Should not match unrelated text
        let result = collect_user_prompt_submit_commands(&commands, "update the API").unwrap();
        assert_eq!(result.len(), 0);
    }

    // Test: Multiple commands with different patterns
    #[test]
    fn test_multiple_commands_different_patterns() {
        let commands = vec![
            UserPromptSubmitCommand {
                run: "echo deploy".to_string(),
                pattern: Some("deploy".to_string()),
                case_insensitive: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None,
            },
            UserPromptSubmitCommand {
                run: "echo always".to_string(),
                pattern: None, // Match all
                case_insensitive: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None,
            },
        ];

        // Both should match for "deploy"
        let result = collect_user_prompt_submit_commands(&commands, "deploy now").unwrap();
        assert_eq!(result.len(), 2);

        // Only the "always" command should match for non-deploy
        let result = collect_user_prompt_submit_commands(&commands, "fix bug").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].command.contains("always"));
    }

    // Test: notifyPerCommand flag is correctly passed through
    #[test]
    fn test_notify_per_command_flag_passed_through() {
        let commands = vec![
            UserPromptSubmitCommand {
                run: "echo with-notify".to_string(),
                pattern: None,
                case_insensitive: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(true),
            },
            UserPromptSubmitCommand {
                run: "echo without-notify".to_string(),
                pattern: None,
                case_insensitive: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: Some(false),
            },
            UserPromptSubmitCommand {
                run: "echo default-notify".to_string(),
                pattern: None,
                case_insensitive: None,
                show_command: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
                notify_per_command: None, // Should default to false
            },
        ];

        let result = collect_user_prompt_submit_commands(&commands, "test prompt").unwrap();
        assert_eq!(result.len(), 3);

        // First command has notifyPerCommand: true
        assert!(
            result[0].notify_per_command,
            "First command should have notify_per_command: true"
        );

        // Second command has notifyPerCommand: false
        assert!(
            !result[1].notify_per_command,
            "Second command should have notify_per_command: false"
        );

        // Third command has notifyPerCommand: None (defaults to false)
        assert!(
            !result[2].notify_per_command,
            "Third command should have notify_per_command: false (default)"
        );
    }
}
