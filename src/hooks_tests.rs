use crate::config::{ConclaudeConfig, NotificationsConfig};
use crate::hooks::*;
use serde_json::Value;
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
fn test_notification_flag_gating_logic() {
    // Test configuration with all flags enabled (default behavior)
    let config_all_enabled = NotificationsConfig {
        enabled: true,
        hooks: vec!["*".to_string()],
        show_errors: true,
        show_success: true,
        show_system_events: true,
    };

    // All notification types should be allowed
    assert!(config_all_enabled.is_enabled_for("PreToolUse"));
    assert!(config_all_enabled.is_enabled_for("SessionStart"));

    // Test configuration with only errors enabled
    let config_errors_only = NotificationsConfig {
        enabled: true,
        hooks: vec!["*".to_string()],
        show_errors: true,
        show_success: false,
        show_system_events: false,
    };

    // This tests the is_enabled_for method - flags are checked in send_notification
    assert!(config_errors_only.is_enabled_for("PreToolUse"));
    assert!(config_errors_only.is_enabled_for("SessionStart"));

    // Test configuration with only success enabled
    let config_success_only = NotificationsConfig {
        enabled: true,
        hooks: vec!["*".to_string()],
        show_errors: false,
        show_success: true,
        show_system_events: false,
    };

    assert!(config_success_only.is_enabled_for("PreToolUse"));
    assert!(config_success_only.is_enabled_for("SessionStart"));

    // Test configuration with only system events enabled
    let config_system_only = NotificationsConfig {
        enabled: true,
        hooks: vec!["*".to_string()],
        show_errors: false,
        show_success: false,
        show_system_events: true,
    };

    assert!(config_system_only.is_enabled_for("PreToolUse"));
    assert!(config_system_only.is_enabled_for("SessionStart"));
}

#[test]
fn test_is_root_addition() {
    use std::env;

    // Get current working directory for testing
    let cwd = env::current_dir().unwrap();

    // Simulate config file in the current directory
    let config_path = cwd.join(".conclaude.yaml");

    // Files at the same level as config should be blocked
    assert!(is_root_addition("test.txt", "test.txt", &config_path));
    assert!(is_root_addition("newfile.rs", "newfile.rs", &config_path));

    // BREAKING CHANGE: Dotfiles are now also blocked at root level
    assert!(is_root_addition(".gitignore", ".gitignore", &config_path));
    assert!(is_root_addition(".env", ".env", &config_path));

    // BREAKING CHANGE: Config files are now also blocked at root level
    assert!(is_root_addition(
        "package.json",
        "package.json",
        &config_path
    ));
    assert!(is_root_addition("config.yaml", "config.yaml", &config_path));

    // Files in subdirectories should not be blocked
    assert!(!is_root_addition(
        "src/test.txt",
        "src/test.txt",
        &config_path
    ));
    assert!(!is_root_addition(
        "tests/foo.rs",
        "tests/foo.rs",
        &config_path
    ));
    assert!(!is_root_addition(
        "nested/deep/file.txt",
        "nested/deep/file.txt",
        &config_path
    ));
}

#[test]
fn test_matches_uneditable_pattern() {
    assert!(matches_uneditable_pattern(
        "package.json",
        "package.json",
        "/path/package.json",
        "package.json"
    )
    .unwrap());
    assert!(matches_uneditable_pattern("test.md", "test.md", "/path/test.md", "*.md").unwrap());
    assert!(matches_uneditable_pattern(
        "src/index.ts",
        "src/index.ts",
        "/path/src/index.ts",
        "src/**/*.ts"
    )
    .unwrap());
    assert!(
        !matches_uneditable_pattern("other.txt", "other.txt", "/path/other.txt", "*.md").unwrap()
    );
}

#[test]
fn test_extract_file_path() {
    let mut tool_input = std::collections::HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::String("test.txt".to_string()),
    );
    assert_eq!(extract_file_path(&tool_input), Some("test.txt".to_string()));

    tool_input.clear();
    tool_input.insert(
        "notebook_path".to_string(),
        Value::String("notebook.ipynb".to_string()),
    );
    assert_eq!(
        extract_file_path(&tool_input),
        Some("notebook.ipynb".to_string())
    );

    tool_input.clear();
    assert_eq!(extract_file_path(&tool_input), None);
}

#[test]
fn test_truncate_output_no_truncation_needed() {
    let output = "line1\nline2\nline3";
    let (truncated, is_truncated, omitted) = truncate_output(output, 10);
    assert_eq!(truncated, "line1\nline2\nline3");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);
}

#[test]
fn test_truncate_output_exact_limit() {
    let output = "line1\nline2\nline3";
    let (truncated, is_truncated, omitted) = truncate_output(output, 3);
    assert_eq!(truncated, "line1\nline2\nline3");
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
fn test_truncate_output_single_line() {
    let output = "single line";
    let (truncated, is_truncated, omitted) = truncate_output(output, 1);
    assert_eq!(truncated, "single line");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);
}

#[test]
fn test_truncate_output_large_limit() {
    let output = "line1\nline2";
    let (truncated, is_truncated, omitted) = truncate_output(output, 10000);
    assert_eq!(truncated, "line1\nline2");
    assert!(!is_truncated);
    assert_eq!(omitted, 0);
}

#[test]
fn test_truncate_output_multiple_lines_exact_boundary() {
    let output = "line1\nline2\nline3\nline4\nline5";
    let (truncated, is_truncated, omitted) = truncate_output(output, 5);
    assert_eq!(truncated, "line1\nline2\nline3\nline4\nline5");
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
                    run: "echo hello".to_string(),
                    message: Some("Custom message".to_string()),
                    show_stdout: Some(true),
                    show_stderr: Some(false),
                    max_output_lines: Some(10),
                    timeout: None,
                },
                StopCommand {
                    run: "ls -la".to_string(),
                    message: None,
                    show_stdout: Some(false),
                    show_stderr: Some(true),
                    max_output_lines: Some(5),
                    timeout: None,
                },
            ],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let commands = collect_stop_commands(&config).unwrap();
    assert_eq!(commands.len(), 2);

    assert_eq!(commands[0].command, "echo hello");
    assert!(commands[0].show_stdout);
    assert!(!commands[0].show_stderr);
    assert_eq!(commands[0].max_output_lines, Some(10));
    assert_eq!(commands[0].message, Some("Custom message".to_string()));

    assert_eq!(commands[1].command, "ls -la");
    assert!(!commands[1].show_stdout);
    assert!(commands[1].show_stderr);
    assert_eq!(commands[1].max_output_lines, Some(5));
    assert_eq!(commands[1].message, None);
}

#[test]
fn test_collect_stop_commands_default_values() {
    use crate::config::StopCommand;

    let config = ConclaudeConfig {
        stop: crate::config::StopConfig {
            commands: vec![StopCommand {
                run: "echo test".to_string(),
                message: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
            }],
            infinite: false,
            infinite_message: None,
        },
        ..Default::default()
    };

    let commands = collect_stop_commands(&config).unwrap();
    assert_eq!(commands.len(), 1);

    // Defaults should be false for show flags and None for max_output_lines
    assert!(!commands[0].show_stdout);
    assert!(!commands[0].show_stderr);
    assert_eq!(commands[0].max_output_lines, None);
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

#[test]
fn test_bash_command_full_match_exact() {
    use glob::Pattern;

    // Exact match
    let pattern = Pattern::new("rm -rf /").unwrap();
    assert!(pattern.matches("rm -rf /"));

    // With wildcard at end
    let pattern2 = Pattern::new("rm -rf /*").unwrap();
    assert!(pattern2.matches("rm -rf /"));
    assert!(pattern2.matches("rm -rf /tmp"));

    // Doesn't match with prefix
    let pattern3 = Pattern::new("rm -rf /*").unwrap();
    assert!(!pattern3.matches("sudo rm -rf /"));
}

#[test]
fn test_bash_command_full_match_wildcard() {
    use glob::Pattern;

    let pattern = Pattern::new("git push --force*").unwrap();
    assert!(pattern.matches("git push --force"));
    assert!(pattern.matches("git push --force origin main"));
    assert!(!pattern.matches("git push origin main"));
}

#[test]
fn test_bash_command_prefix_match_at_start() {
    use glob::Pattern;

    let pattern = Pattern::new("curl *").unwrap();
    let command = "curl https://example.com && echo done";

    // Simulate prefix matching logic
    let words: Vec<&str> = command.split_whitespace().collect();
    let matches = (1..=words.len()).any(|i| {
        let prefix = words[..i].join(" ");
        pattern.matches(&prefix)
    });

    assert!(matches, "Should match 'curl https://example.com' at start");
}

#[test]
fn test_bash_command_prefix_no_match_middle() {
    use glob::Pattern;

    let pattern = Pattern::new("curl *").unwrap();
    let command = "echo test && curl https://example.com";

    let words: Vec<&str> = command.split_whitespace().collect();
    let matches = (1..=words.len()).any(|i| {
        let prefix = words[..i].join(" ");
        pattern.matches(&prefix)
    });

    assert!(!matches, "Should not match 'curl' in middle of command");
}

#[test]
fn test_bash_command_wildcard_variations() {
    use glob::Pattern;

    let test_cases = vec![
        ("rm -rf*", "rm -rf /", true),
        ("rm -rf*", "rm -rf /tmp", true),
        ("git push --force*", "git push --force", true),
        ("git push --force*", "git push --force origin main", true),
        ("docker run*", "docker start", false),
        ("npm install*", "npm install", true),
        ("npm install*", "npm ci", false),
    ];

    for (pattern_str, command, expected) in test_cases {
        let pattern = Pattern::new(pattern_str).unwrap();
        assert_eq!(
            pattern.matches(command),
            expected,
            "Pattern: '{}', Command: '{}', Expected: {}",
            pattern_str,
            command,
            expected
        );
    }
}

// Tests for subagent stop pattern matching
#[test]
fn test_match_subagent_patterns_wildcard() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "*".to_string(),
        vec![SubagentStopCommand {
            run: "echo all".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    let matches = match_subagent_patterns("coder", &config).unwrap();
    assert_eq!(matches, vec!["*"]);

    let matches = match_subagent_patterns("tester", &config).unwrap();
    assert_eq!(matches, vec!["*"]);

    let matches = match_subagent_patterns("any-agent-name", &config).unwrap();
    assert_eq!(matches, vec!["*"]);
}

#[test]
fn test_match_subagent_patterns_exact() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: "echo coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );

    let config = SubagentStopConfig { commands };

    let matches = match_subagent_patterns("coder", &config).unwrap();
    assert_eq!(matches, vec!["coder"]);

    // Should NOT match auto-coder or coder-agent
    let matches = match_subagent_patterns("auto-coder", &config).unwrap();
    assert!(matches.is_empty());

    let matches = match_subagent_patterns("coder-agent", &config).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_match_subagent_patterns_prefix_glob() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "test*".to_string(),
        vec![SubagentStopCommand {
            run: "echo test".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
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
            run: "echo coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
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
            run: "echo agent".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
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
            run: "echo all".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: "echo coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );
    commands.insert(
        "*coder".to_string(),
        vec![SubagentStopCommand {
            run: "echo suffix-coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
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
            run: "echo coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );
    commands.insert(
        "*".to_string(),
        vec![SubagentStopCommand {
            run: "echo all".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
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
            run: "echo coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );
    commands.insert(
        "tester".to_string(),
        vec![SubagentStopCommand {
            run: "echo tester".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
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
                run: "echo first".to_string(),
                message: Some("First command".to_string()),
                show_stdout: Some(true),
                show_stderr: Some(false),
                max_output_lines: Some(10),
                timeout: None,
            },
            SubagentStopCommand {
                run: "echo second".to_string(),
                message: None,
                show_stdout: None,
                show_stderr: None,
                max_output_lines: None,
                timeout: None,
            },
        ],
    );

    let config = SubagentStopConfig { commands };
    let matching_patterns = vec!["coder"];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].command, "echo first");
    assert!(collected[0].show_stdout);
    assert!(!collected[0].show_stderr);
    assert_eq!(collected[0].max_output_lines, Some(10));
    assert_eq!(collected[0].message, Some("First command".to_string()));

    assert_eq!(collected[1].command, "echo second");
    assert!(!collected[1].show_stdout);
    assert!(!collected[1].show_stderr);
    assert_eq!(collected[1].max_output_lines, None);
    assert_eq!(collected[1].message, None);
}

#[test]
fn test_collect_subagent_stop_commands_multiple_patterns() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "*".to_string(),
        vec![SubagentStopCommand {
            run: "echo wildcard".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: "echo coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );

    let config = SubagentStopConfig { commands };
    // Wildcard first, then specific pattern (as match_subagent_patterns returns)
    let matching_patterns = vec!["*", "coder"];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();

    assert_eq!(collected.len(), 2);
    // Commands should be in order of patterns
    assert_eq!(collected[0].command, "echo wildcard");
    assert_eq!(collected[1].command, "echo coder");
}

#[test]
fn test_collect_subagent_stop_commands_no_matching_patterns() {
    use crate::config::{SubagentStopCommand, SubagentStopConfig};

    let mut commands = std::collections::HashMap::new();
    commands.insert(
        "coder".to_string(),
        vec![SubagentStopCommand {
            run: "echo coder".to_string(),
            message: None,
            show_stdout: None,
            show_stderr: None,
            max_output_lines: None,
            timeout: None,
        }],
    );

    let config = SubagentStopConfig { commands };
    let matching_patterns: Vec<&str> = vec![];

    let collected = collect_subagent_stop_commands(&config, &matching_patterns).unwrap();
    assert!(collected.is_empty());
}

#[test]
fn test_extract_agent_name_from_transcript_success() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary transcript file with realistic content
    let mut temp_file = NamedTempFile::new().unwrap();

    // Write a Task tool_use line
    writeln!(
        temp_file,
        r#"{{"message":{{"role":"assistant","content":[{{"type":"tool_use","id":"toolu_01424YNSBt1xf2XzWa3NBN4b","name":"Task","input":{{"subagent_type":"coder","instructions":"Implement the feature"}}}}]}}}}"#
    ).unwrap();

    // Write some other lines
    writeln!(
        temp_file,
        r#"{{"message":{{"role":"user","content":"Test message"}}}}"#
    ).unwrap();

    // Write a tool_result line with matching agentId
    writeln!(
        temp_file,
        r#"{{"message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"toolu_01424YNSBt1xf2XzWa3NBN4b","content":"Result"}}]}},"toolUseResult":{{"agentId":"adb0a8b","status":"completed"}}}}"#
    ).unwrap();

    temp_file.flush().unwrap();

    let result = extract_agent_name_from_transcript(
        temp_file.path().to_str().unwrap(),
        "adb0a8b"
    ).unwrap();

    assert_eq!(result, Some("coder".to_string()));
}

#[test]
fn test_extract_agent_name_from_transcript_agent_not_found() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary transcript file
    let mut temp_file = NamedTempFile::new().unwrap();

    writeln!(
        temp_file,
        r#"{{"message":{{"role":"assistant","content":[{{"type":"tool_use","id":"toolu_123","name":"Task","input":{{"subagent_type":"coder"}}}}]}}}}"#
    ).unwrap();

    writeln!(
        temp_file,
        r#"{{"message":{{"content":[{{"type":"tool_result","tool_use_id":"toolu_123"}}]}},"toolUseResult":{{"agentId":"different_id"}}}}"#
    ).unwrap();

    temp_file.flush().unwrap();

    let result = extract_agent_name_from_transcript(
        temp_file.path().to_str().unwrap(),
        "nonexistent_id"
    ).unwrap();

    assert_eq!(result, None);
}

#[test]
fn test_extract_agent_name_from_transcript_file_not_found() {
    let result = extract_agent_name_from_transcript(
        "/nonexistent/path/to/transcript.jsonl",
        "some_id"
    ).unwrap();

    assert_eq!(result, None);
}

#[test]
fn test_extract_agent_name_from_transcript_different_agent_types() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let test_cases = vec![
        ("coder", "agent123"),
        ("tester", "agent456"),
        ("stuck", "agent789"),
    ];

    for (agent_type, agent_id) in test_cases {
        let mut temp_file = NamedTempFile::new().unwrap();

        writeln!(
            temp_file,
            r#"{{"message":{{"role":"assistant","content":[{{"type":"tool_use","id":"tool_use_id","name":"Task","input":{{"subagent_type":"{}"}}}}]}}}}"#,
            agent_type
        ).unwrap();

        writeln!(
            temp_file,
            r#"{{"message":{{"content":[{{"type":"tool_result","tool_use_id":"tool_use_id"}}]}},"toolUseResult":{{"agentId":"{}"}}}}"#,
            agent_id
        ).unwrap();

        temp_file.flush().unwrap();

        let result = extract_agent_name_from_transcript(
            temp_file.path().to_str().unwrap(),
            agent_id
        ).unwrap();

        assert_eq!(result, Some(agent_type.to_string()));
    }
}
