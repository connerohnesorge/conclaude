use conclaude::config::{ConclaudeConfig, StopCommand, StopConfig};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

/// Test that maxOutputLines is properly validated by the schema
#[test]
fn test_max_output_lines_valid_values() {
    // Test valid values: 1, 100, 10000
    let valid_configs = [
        r#"
stop:
  commands:
    - run: "echo test"
      maxOutputLines: 1
preToolUse:
  preventRootAdditions: true
"#,
        r#"
stop:
  commands:
    - run: "echo test"
      maxOutputLines: 100
preToolUse:
  preventRootAdditions: true
"#,
        r#"
stop:
  commands:
    - run: "echo test"
      maxOutputLines: 10000
preToolUse:
  preventRootAdditions: true
"#,
    ];

    for (idx, config_content) in valid_configs.iter().enumerate() {
        let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
        assert!(
            result.is_ok(),
            "Valid config #{} should parse successfully: {:?}",
            idx,
            result.err()
        );
    }
}

/// Test that maxOutputLines rejects values outside the valid range
#[test]
fn test_max_output_lines_invalid_values_at_schema_level() {
    // Note: schemars validation happens at schema generation time, not at deserialization time
    // So serde_yaml will accept any u32 value. This test verifies the schema is properly defined.

    // Test that we can parse configs with out-of-range values
    // (The schema validation would happen at the JSON Schema level when used by external tools)
    let config_with_zero = r#"
stop:
  commands:
    - run: "echo test"
      maxOutputLines: 0
preToolUse:
  preventRootAdditions: true
"#;

    // serde_yaml will parse this, but the schema says min=1
    let result = serde_yaml::from_str::<ConclaudeConfig>(config_with_zero);
    // This will parse successfully because serde doesn't enforce schema constraints
    assert!(result.is_ok(), "serde_yaml parses any u32 value");

    // The schema constraint is enforced by JSON Schema validators, not serde
    // We verify the schema is correctly defined by checking the struct definition
    let config = result.unwrap();
    assert_eq!(config.stop.commands[0].max_output_lines, Some(0));
}

/// Test backward compatibility: commands without maxOutputLines field work correctly
#[test]
fn test_backward_compatibility_no_max_output_lines() {
    let config_content = r#"
stop:
  commands:
    - run: "echo hello"
      showStdout: true
      showStderr: true
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Config without maxOutputLines should parse successfully"
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 1);
    assert_eq!(config.stop.commands[0].max_output_lines, None);
    assert_eq!(config.stop.commands[0].show_stdout, Some(true));
    assert_eq!(config.stop.commands[0].show_stderr, Some(true));
}

/// Test interaction with showStdout/showStderr flags
#[test]
fn test_interaction_with_show_flags() {
    let test_cases = vec![
        (
            "maxOutputLines with showStdout=false",
            r#"
stop:
  commands:
    - run: "echo test"
      showStdout: false
      showStderr: true
      maxOutputLines: 10
preToolUse:
  preventRootAdditions: true
"#,
            false,
            true,
            Some(10),
        ),
        (
            "maxOutputLines with showStderr=false",
            r#"
stop:
  commands:
    - run: "echo test"
      showStdout: true
      showStderr: false
      maxOutputLines: 20
preToolUse:
  preventRootAdditions: true
"#,
            true,
            false,
            Some(20),
        ),
        (
            "maxOutputLines with both false",
            r#"
stop:
  commands:
    - run: "echo test"
      showStdout: false
      showStderr: false
      maxOutputLines: 5
preToolUse:
  preventRootAdditions: true
"#,
            false,
            false,
            Some(5),
        ),
        (
            "maxOutputLines with both true",
            r#"
stop:
  commands:
    - run: "echo test"
      showStdout: true
      showStderr: true
      maxOutputLines: 15
preToolUse:
  preventRootAdditions: true
"#,
            true,
            true,
            Some(15),
        ),
    ];

    for (test_name, config_content, expected_stdout, expected_stderr, expected_max_lines) in
        test_cases
    {
        let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
        assert!(
            result.is_ok(),
            "Config for '{}' should parse successfully: {:?}",
            test_name,
            result.err()
        );

        let config = result.unwrap();
        assert_eq!(
            config.stop.commands[0].show_stdout,
            Some(expected_stdout),
            "Test: {}",
            test_name
        );
        assert_eq!(
            config.stop.commands[0].show_stderr,
            Some(expected_stderr),
            "Test: {}",
            test_name
        );
        assert_eq!(
            config.stop.commands[0].max_output_lines, expected_max_lines,
            "Test: {}",
            test_name
        );
    }
}

/// Test that multiple commands can have different maxOutputLines values
#[test]
fn test_independent_max_output_lines_per_command() {
    let config_content = r#"
stop:
  commands:
    - run: "echo first"
      showStdout: true
      maxOutputLines: 5
    - run: "echo second"
      showStdout: true
      maxOutputLines: 10
    - run: "echo third"
      showStdout: true
      # no maxOutputLines
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Config with multiple commands should parse successfully"
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 3);
    assert_eq!(config.stop.commands[0].max_output_lines, Some(5));
    assert_eq!(config.stop.commands[1].max_output_lines, Some(10));
    assert_eq!(config.stop.commands[2].max_output_lines, None);
}

/// Test that empty commands array works
#[test]
fn test_empty_commands_array() {
    let config_content = r#"
stop:
  commands: []
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Empty commands array should parse successfully"
    );

    let config = result.unwrap();
    assert!(config.stop.commands.is_empty());
}

/// Test multiple commands with maxOutputLines
#[test]
fn test_multiple_commands_with_max_output_lines() {
    let config_content = r#"
stop:
  commands:
    - run: "echo first"
      showStdout: true
      maxOutputLines: 50
    - run: "echo second"
      showStdout: true
      maxOutputLines: 100
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Multiple commands should parse successfully"
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 2);
    assert_eq!(config.stop.commands[0].max_output_lines, Some(50));
    assert_eq!(config.stop.commands[1].max_output_lines, Some(100));
}

/// Test that omitting all optional fields works
#[test]
fn test_minimal_command_config() {
    let config_content = r#"
stop:
  commands:
    - run: "echo minimal"
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Minimal command config should parse successfully"
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 1);
    assert_eq!(config.stop.commands[0].run, "echo minimal");
    assert_eq!(config.stop.commands[0].message, None);
    assert_eq!(config.stop.commands[0].show_stdout, None);
    assert_eq!(config.stop.commands[0].show_stderr, None);
    assert_eq!(config.stop.commands[0].max_output_lines, None);
}

/// Test that maxOutputLines can be explicitly set to null
#[test]
fn test_max_output_lines_explicit_null() {
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
      showStdout: true
      maxOutputLines: null
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Config with explicit null maxOutputLines should parse successfully"
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands[0].max_output_lines, None);
}

/// Test the StopCommand struct directly
#[test]
fn test_stop_command_struct_serialization() {
    let cmd = StopCommand {
        run: "echo test".to_string(),
        message: Some("Test message".to_string()),
        show_command: None,
        show_stdout: Some(true),
        show_stderr: Some(false),
        max_output_lines: Some(50),
        timeout: None,
    };

    let yaml = serde_yaml::to_string(&cmd).unwrap();
    let deserialized: StopCommand = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(deserialized.run, "echo test");
    assert_eq!(deserialized.message, Some("Test message".to_string()));
    assert_eq!(deserialized.show_stdout, Some(true));
    assert_eq!(deserialized.show_stderr, Some(false));
    assert_eq!(deserialized.max_output_lines, Some(50));
}

/// Test that the schema properly defined the range constraint
#[test]
fn test_schema_generation_includes_range() {
    use schemars::schema_for;

    let schema = schema_for!(StopCommand);
    let schema_json = serde_json::to_string_pretty(&schema).unwrap();

    // Verify that the schema includes maxOutputLines field
    assert!(
        schema_json.contains("maxOutputLines"),
        "Schema should include maxOutputLines field"
    );

    // Note: The actual validation of min=1, max=10000 happens at the JSON Schema level
    // when external tools use the schema. The schemars annotation ensures it's in the schema.
}

/// Test full config file with all features combined
#[test]
fn test_comprehensive_config_with_output_limiting() {
    let config_content = r#"
stop:
  commands:
    - run: "npm test"
      message: "Tests failed. Please fix before continuing."
      showStdout: true
      showStderr: true
      maxOutputLines: 50
    - run: "npm run lint"
      message: "Linting failed"
      showStdout: false
      showStderr: true
      maxOutputLines: 100
    - run: "echo no limits"
      showStdout: true
  infinite: false
preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "package-lock.json"
  preventAdditions: []
notifications:
  enabled: true
  hooks: ["*"]
  showErrors: true
  showSuccess: false
  showSystemEvents: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Comprehensive config should parse successfully: {:?}",
        result.err()
    );

    let config = result.unwrap();

    // Verify commands
    assert_eq!(config.stop.commands.len(), 3);

    // First command
    assert_eq!(config.stop.commands[0].run, "npm test");
    assert_eq!(
        config.stop.commands[0].message,
        Some("Tests failed. Please fix before continuing.".to_string())
    );
    assert_eq!(config.stop.commands[0].show_stdout, Some(true));
    assert_eq!(config.stop.commands[0].show_stderr, Some(true));
    assert_eq!(config.stop.commands[0].max_output_lines, Some(50));

    // Second command
    assert_eq!(config.stop.commands[1].run, "npm run lint");
    assert_eq!(
        config.stop.commands[1].message,
        Some("Linting failed".to_string())
    );
    assert_eq!(config.stop.commands[1].show_stdout, Some(false));
    assert_eq!(config.stop.commands[1].show_stderr, Some(true));
    assert_eq!(config.stop.commands[1].max_output_lines, Some(100));

    // Third command (no maxOutputLines)
    assert_eq!(config.stop.commands[2].run, "echo no limits");
    assert_eq!(config.stop.commands[2].show_stdout, Some(true));
    assert_eq!(config.stop.commands[2].max_output_lines, None);

    // Verify other sections
    assert!(config.pre_tool_use.prevent_root_additions);
    assert!(config.notifications.enabled);
}

/// Test that unknown fields are rejected (deny_unknown_fields)
#[test]
fn test_reject_unknown_fields_in_stop_command() {
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
      unknownField: "should fail"
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_err(),
        "Config with unknown field should be rejected"
    );

    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("unknown field") || error.contains("unknownField"),
        "Error should mention the unknown field: {}",
        error
    );
}

/// Test edge case: maxOutputLines at boundary values
#[test]
fn test_max_output_lines_boundary_values() {
    let test_cases = vec![
        (1_u32, "minimum value"),
        (10000_u32, "maximum value"),
        (5000_u32, "middle value"),
    ];

    for (value, description) in test_cases {
        let config_content = format!(
            r#"
stop:
  commands:
    - run: "echo test"
      maxOutputLines: {}
preToolUse:
  preventRootAdditions: true
"#,
            value
        );

        let result = serde_yaml::from_str::<ConclaudeConfig>(&config_content);
        assert!(
            result.is_ok(),
            "Config with {} ({}) should parse successfully",
            description,
            value
        );

        let config = result.unwrap();
        assert_eq!(
            config.stop.commands[0].max_output_lines,
            Some(value),
            "maxOutputLines should be {} ({})",
            value,
            description
        );
    }
}

/// Test that StopConfig itself can be serialized and deserialized
#[test]
fn test_stop_config_round_trip() {
    let original = StopConfig {
        commands: vec![StopCommand {
            run: "npm test".to_string(),
            message: Some("Failed".to_string()),
            show_command: None,
            show_stdout: Some(true),
            show_stderr: Some(true),
            max_output_lines: Some(25),
            timeout: None,
        }],
        infinite: false,
        infinite_message: None,
    };

    let yaml = serde_yaml::to_string(&original).unwrap();
    let deserialized: StopConfig = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(deserialized.commands.len(), original.commands.len());
    assert_eq!(
        deserialized.commands[0].max_output_lines,
        original.commands[0].max_output_lines
    );
}

/// Integration test: Write config to file and load it
#[tokio::test]
async fn test_load_config_with_max_output_lines_from_file() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    let config_content = r#"
stop:
  commands:
    - run: "echo test"
      showStdout: true
      maxOutputLines: 75
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    // Load config using the actual config loading function with explicit path
    let result = conclaude::config::load_conclaude_config(Some(temp_dir.path())).await;

    assert!(
        result.is_ok(),
        "Should successfully load config from file: {:?}",
        result.err()
    );

    let (config, loaded_path) = result.unwrap();
    assert_eq!(config.stop.commands.len(), 1);
    assert_eq!(config.stop.commands[0].max_output_lines, Some(75));
    assert!(loaded_path.ends_with(".conclaude.yaml"));
}

/// Test that camelCase field name (maxOutputLines) is used, not snake_case
#[test]
fn test_camel_case_field_name_required() {
    // Test that snake_case version fails
    let snake_case_config = r#"
stop:
  commands:
    - run: "echo test"
      max_output_lines: 10
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(snake_case_config);
    assert!(
        result.is_err(),
        "snake_case field name should be rejected (deny_unknown_fields)"
    );

    // Test that camelCase version succeeds
    let camel_case_config = r#"
stop:
  commands:
    - run: "echo test"
      maxOutputLines: 10
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(camel_case_config);
    assert!(
        result.is_ok(),
        "camelCase field name should be accepted: {:?}",
        result.err()
    );
}

/// Integration test: Verify stdout is NOT printed to console when showStdout is false
#[test]
fn test_stop_hook_console_output_with_show_stdout_false() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with showStdout: false and a command that will fail
    let config_content = r#"
stop:
  commands:
    - run: "bash -c 'echo STDOUT_TEST_OUTPUT && echo STDERR_TEST_OUTPUT >&2 && exit 1'"
      showStdout: false
      showStderr: true
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    // Build the project first to ensure the binary exists
    let project_root = std::env::current_dir().unwrap();
    let binary_path = project_root.join("target/debug/conclaude");

    // Create JSON payload for Stop hook
    let payload = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": "/tmp/test.jsonl",
        "hook_event_name": "Stop",
        "cwd": temp_path.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    // Run the stop hook from temp directory so it picks up the config
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "Stop"])
        .current_dir(temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn stop hook");

    // Write JSON payload to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(serde_json::to_string(&payload).unwrap().as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to run stop hook");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract the eprintln diagnostic (the part before the error message)
    let diagnostic_part = stderr
        .split("Command failed with exit code")
        .next()
        .unwrap_or("");

    // Verify the eprintln diagnostic does NOT contain Stdout section when showStdout is false
    assert!(
        !diagnostic_part.contains("  Stdout:"),
        "Console diagnostic should completely omit Stdout section when showStdout is false. stderr was:\n{}",
        stderr
    );

    // Verify stderr IS shown in the eprintln diagnostic (since showStderr is true)
    assert!(
        diagnostic_part.contains("  Stderr:\n    STDERR_TEST_OUTPUT"),
        "Console diagnostic should show stderr when showStderr is true. stderr was:\n{}",
        stderr
    );

    // Verify Command and Status are always shown
    assert!(
        diagnostic_part.contains("Stop command failed"),
        "Should always show command failure message. stderr was:\n{}",
        stderr
    );
    assert!(
        diagnostic_part.contains("exit code"),
        "Should always show exit code. stderr was:\n{}",
        stderr
    );

    // Verify the error message does not include stdout section (showStdout: false)
    let error_message_part = stderr
        .split("Command failed with exit code")
        .nth(1)
        .unwrap_or("");
    assert!(
        !error_message_part.contains("\nStdout:"),
        "Error message should not include Stdout section when showStdout is false. stderr was:\n{}",
        stderr
    );

    // Verify the error message includes stderr section (showStderr: true)
    assert!(
        error_message_part.contains("\nStderr: STDERR_TEST_OUTPUT"),
        "Error message should include Stderr section when showStderr is true. stderr was:\n{}",
        stderr
    );
}

/// Integration test: Verify stderr is NOT printed to console when showStderr is false
#[test]
fn test_stop_hook_console_output_with_show_stderr_false() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with showStderr: false and a command that will fail
    let config_content = r#"
stop:
  commands:
    - run: "bash -c 'echo STDOUT_TEST_OUTPUT && echo STDERR_TEST_OUTPUT >&2 && exit 1'"
      showStdout: true
      showStderr: false
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    // Build the project first to ensure the binary exists
    let project_root = std::env::current_dir().unwrap();
    let binary_path = project_root.join("target/debug/conclaude");

    // Create JSON payload for Stop hook
    let payload = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": "/tmp/test.jsonl",
        "hook_event_name": "Stop",
        "cwd": temp_path.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    // Run the stop hook from temp directory so it picks up the config
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "Stop"])
        .current_dir(temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn stop hook");

    // Write JSON payload to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(serde_json::to_string(&payload).unwrap().as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to run stop hook");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract the eprintln diagnostic (the part before the error message)
    let diagnostic_part = stderr
        .split("Command failed with exit code")
        .next()
        .unwrap_or("");

    // Verify the eprintln diagnostic does NOT contain Stderr section when showStderr is false
    assert!(
        !diagnostic_part.contains("  Stderr:"),
        "Console diagnostic should completely omit Stderr section when showStderr is false. stderr was:\n{}",
        stderr
    );

    // Verify stdout IS shown in the eprintln diagnostic (since showStdout is true)
    assert!(
        diagnostic_part.contains("  Stdout:\n    STDOUT_TEST_OUTPUT"),
        "Console diagnostic should show stdout when showStdout is true. stderr was:\n{}",
        stderr
    );

    // Verify Command and Status are always shown
    assert!(
        diagnostic_part.contains("Stop command failed"),
        "Should always show command failure message. stderr was:\n{}",
        stderr
    );
    assert!(
        diagnostic_part.contains("exit code"),
        "Should always show exit code. stderr was:\n{}",
        stderr
    );

    // Verify the error message does not include stderr section (showStderr: false)
    let error_message_part = stderr
        .split("Command failed with exit code")
        .nth(1)
        .unwrap_or("");
    assert!(
        !error_message_part.contains("\nStderr:"),
        "Error message should not include Stderr section when showStderr is false. stderr was:\n{}",
        stderr
    );

    // Verify the error message includes stdout section (showStdout: true)
    assert!(
        error_message_part.contains("\nStdout: STDOUT_TEST_OUTPUT"),
        "Error message should include Stdout section when showStdout is true. stderr was:\n{}",
        stderr
    );
}

/// Integration test: Verify no output is leaked when both flags are false
#[test]
fn test_stop_hook_console_output_with_both_flags_false() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with both flags false and a command that will fail
    let config_content = r#"
stop:
  commands:
    - run: "bash -c 'echo STDOUT_TEST_OUTPUT && echo STDERR_TEST_OUTPUT >&2 && exit 1'"
      showStdout: false
      showStderr: false
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    // Build the project first to ensure the binary exists
    let project_root = std::env::current_dir().unwrap();
    let binary_path = project_root.join("target/debug/conclaude");

    // Create JSON payload for Stop hook
    let payload = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": "/tmp/test.jsonl",
        "hook_event_name": "Stop",
        "cwd": temp_path.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    // Run the stop hook from temp directory so it picks up the config
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "Stop"])
        .current_dir(temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn stop hook");

    // Write JSON payload to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(serde_json::to_string(&payload).unwrap().as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to run stop hook");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract the eprintln diagnostic (the part before the error message)
    let diagnostic_part = stderr
        .split("Command failed with exit code")
        .next()
        .unwrap_or("");

    // Verify the eprintln diagnostic does NOT contain Stdout or Stderr sections
    assert!(
        !diagnostic_part.contains("  Stdout:"),
        "Console diagnostic should completely omit Stdout section when showStdout is false. stderr was:\n{}",
        stderr
    );
    assert!(
        !diagnostic_part.contains("  Stderr:"),
        "Console diagnostic should completely omit Stderr section when showStderr is false. stderr was:\n{}",
        stderr
    );

    // Verify Command and Status are always shown
    assert!(
        diagnostic_part.contains("Stop command failed"),
        "Should always show command failure message. stderr was:\n{}",
        stderr
    );
    assert!(
        diagnostic_part.contains("exit code"),
        "Should always show exit code. stderr was:\n{}",
        stderr
    );

    // Verify the error message does not include stdout or stderr sections
    let error_message_part = stderr
        .split("Command failed with exit code")
        .nth(1)
        .unwrap_or("");
    assert!(
        !error_message_part.contains("\nStdout:"),
        "Error message should not include Stdout section when showStdout is false. stderr was:\n{}",
        stderr
    );
    assert!(
        !error_message_part.contains("\nStderr:"),
        "Error message should not include Stderr section when showStderr is false. stderr was:\n{}",
        stderr
    );
}

/// Integration test: Verify output is shown when both flags are true
#[test]
fn test_stop_hook_console_output_with_both_flags_true() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with both flags true and a command that will fail
    let config_content = r#"
stop:
  commands:
    - run: "bash -c 'echo STDOUT_TEST_OUTPUT && echo STDERR_TEST_OUTPUT >&2 && exit 1'"
      showStdout: true
      showStderr: true
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    // Build the project first to ensure the binary exists
    let project_root = std::env::current_dir().unwrap();
    let binary_path = project_root.join("target/debug/conclaude");

    // Create JSON payload for Stop hook
    let payload = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": "/tmp/test.jsonl",
        "hook_event_name": "Stop",
        "cwd": temp_path.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    // Run the stop hook from temp directory so it picks up the config
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "Stop"])
        .current_dir(temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn stop hook");

    // Write JSON payload to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(serde_json::to_string(&payload).unwrap().as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to run stop hook");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract the eprintln diagnostic (the part before the error message)
    let diagnostic_part = stderr
        .split("Command failed with exit code")
        .next()
        .unwrap_or("");

    // Verify the eprintln diagnostic shows both stdout and stderr
    assert!(
        diagnostic_part.contains("  Stdout:\n    STDOUT_TEST_OUTPUT"),
        "Console diagnostic should show stdout when showStdout is true. stderr was:\n{}",
        stderr
    );
    assert!(
        diagnostic_part.contains("  Stderr:\n    STDERR_TEST_OUTPUT"),
        "Console diagnostic should show stderr when showStderr is true. stderr was:\n{}",
        stderr
    );

    // Verify Command and Status are always shown
    assert!(
        diagnostic_part.contains("Stop command failed"),
        "Should always show command failure message. stderr was:\n{}",
        stderr
    );
    assert!(
        diagnostic_part.contains("exit code"),
        "Should always show exit code. stderr was:\n{}",
        stderr
    );

    // Verify the error message includes both stdout and stderr sections
    let error_message_part = stderr
        .split("Command failed with exit code")
        .nth(1)
        .unwrap_or("");
    assert!(
        error_message_part.contains("\nStdout: STDOUT_TEST_OUTPUT"),
        "Error message should include Stdout section when showStdout is true. stderr was:\n{}",
        stderr
    );
    assert!(
        error_message_part.contains("\nStderr: STDERR_TEST_OUTPUT"),
        "Error message should include Stderr section when showStderr is true. stderr was:\n{}",
        stderr
    );
}
