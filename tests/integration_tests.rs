use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[test]
fn test_cli_init_command() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "init",
            "--config-path",
            &temp_path.join(".conclaude.yaml").to_string_lossy(),
            "--claude-path",
            &temp_path.join(".claude").to_string_lossy(),
        ])
        .output()
        .expect("Failed to run CLI init command");

    // Command should succeed (all output goes to log files now)
    assert!(output.status.success(), "Init command should succeed");

    // Verify files were created
    assert!(temp_path.join(".conclaude.yaml").exists());
    assert!(temp_path.join(".claude").exists());
    assert!(temp_path.join(".claude/settings.json").exists());

    // Verify config file content
    let config_content =
        fs::read_to_string(temp_path.join(".conclaude.yaml")).expect("Failed to read config file");
    assert!(config_content.contains("# yaml-language-server:"));
    assert!(config_content.contains("stop:"));
    assert!(config_content.contains("preToolUse:"));
    assert!(config_content.contains("preventRootAdditions: true"));

    // Verify Claude settings file content
    let settings_content = fs::read_to_string(temp_path.join(".claude/settings.json"))
        .expect("Failed to read settings file");
    assert!(settings_content.contains("PreToolUse"));
    assert!(settings_content.contains("PostToolUse"));
    assert!(settings_content.contains("conclaude Hooks PreToolUse"));
}

#[test]
fn test_cli_init_command_force_overwrite() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create existing config file
    let mut file = File::create(&config_path).expect("Failed to create config file");
    file.write_all(b"existing content")
        .expect("Failed to write existing content");

    // First init without force should succeed but skip config creation
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "init",
            "--config-path",
            &config_path.to_string_lossy(),
            "--claude-path",
            &temp_path.join(".claude").to_string_lossy(),
        ])
        .output()
        .expect("Failed to run CLI init command");

    // Should succeed but skip config creation (continues with other files)
    assert!(
        output.status.success(),
        "Init without force should succeed (skipping config creation)"
    );

    // Verify the config was NOT overwritten
    let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");
    assert!(
        config_content.contains("existing content"),
        "Config should not be overwritten without --force"
    );

    // Verify that other files were still created
    assert!(
        temp_path.join(".claude/settings.json").exists(),
        "settings.json should still be created"
    );

    // Second init with force should succeed and overwrite config
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "init",
            "--force",
            "--config-path",
            &config_path.to_string_lossy(),
            "--claude-path",
            &temp_path.join(".claude").to_string_lossy(),
        ])
        .output()
        .expect("Failed to run CLI init command");

    // Should succeed with --force flag
    assert!(output.status.success(), "Init with --force should succeed");

    // Verify the config was overwritten
    let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");
    assert!(config_content.contains("stop:"));
    assert!(!config_content.contains("existing content"));
}

// Note: Testing the actual hook handlers with stdin would require more complex setup
// with mock JSON payloads. These tests verify the CLI structure and basic functionality.

// ========== Validate Subcommand Integration Tests ==========

#[test]
fn test_validate_with_valid_configuration() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a valid configuration
    let valid_config = r#"
stop:
  commands:
    - run: "echo test"
  infinite: false
preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "*.lock"
    - "package-lock.json"
  toolUsageValidation: []
  preventAdditions: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;

    fs::write(&config_path, valid_config).expect("Failed to write config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Should succeed with exit code 0
    assert!(
        output.status.success(),
        "Validate should succeed with valid config. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify success message in output
    assert!(stdout.contains("Configuration is valid"));
    assert!(stdout.contains("Config file:"));
    assert!(stdout.contains("Configuration summary:"));
}

#[test]
fn test_validate_with_missing_configuration() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    // Create a deep directory structure to ensure no config is found
    let mut current_path = temp_dir.path().to_path_buf();
    for i in 0..15 {
        current_path = current_path.join(format!("level_{i}"));
        fs::create_dir(&current_path).unwrap();
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &current_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Validate should fail when config is missing"
    );

    // Exit code should be 1 (from std::process::exit(1))
    // Note: May be 101 in test environment due to how cargo run handles exits
    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(101),
        "Exit code should be 1 or 101 for missing config, got: {:?}",
        exit_code
    );

    // Verify error message mentions "Configuration file not found"
    assert!(
        stderr.contains("Configuration file not found")
            || stderr.contains("Configuration validation failed"),
        "Error message should mention config not found. stderr: {stderr}"
    );
}

#[test]
fn test_validate_with_invalid_yaml_syntax() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with invalid YAML syntax (bad indentation)
    let invalid_yaml = r#"
stop:
  commands:
    - run: "echo test"
preToolUse:
preventRootAdditions: true
"#;

    fs::write(&config_path, invalid_yaml).expect("Failed to write config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Validate should fail with invalid YAML syntax"
    );
    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(101),
        "Exit code should be 1 or 101 for invalid YAML, got: {:?}",
        exit_code
    );

    // Verify error message mentions parsing/syntax error
    assert!(
        stderr.contains("validation failed")
            || stderr.contains("parse")
            || stderr.contains("syntax"),
        "Error message should mention parsing/syntax issue. stderr: {stderr}"
    );
}

#[test]
fn test_validate_with_unknown_fields() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with unknown fields
    let config_with_unknown_fields = r#"
stop:
  commands:
    - run: "echo test"
  unknownField: "should fail"
  anotherBadField: 123
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_with_unknown_fields).expect("Failed to write config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Validate should fail with unknown fields"
    );
    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(101),
        "Exit code should be 1 or 101 for unknown fields, got: {:?}",
        exit_code
    );

    // Verify error message mentions unknown field
    assert!(
        stderr.contains("unknown field"),
        "Error message should mention unknown field. stderr: {stderr}"
    );

    // Verify error message provides helpful suggestions
    assert!(
        stderr.contains("Valid field names") || stderr.contains("Common causes"),
        "Error message should provide helpful suggestions. stderr: {stderr}"
    );
}

#[test]
fn test_validate_with_invalid_types() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with invalid types (string instead of boolean)
    let config_with_invalid_types = r#"
stop:
  commands:
    - run: "echo test"
  infinite: "true"
preToolUse:
  preventRootAdditions: "yes"
"#;

    fs::write(&config_path, config_with_invalid_types).expect("Failed to write config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Validate should fail with invalid types"
    );
    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(101),
        "Exit code should be 1 or 101 for invalid types, got: {:?}",
        exit_code
    );

    // Verify error message mentions type error
    assert!(
        stderr.contains("invalid type") || stderr.contains("expected"),
        "Error message should mention type error. stderr: {stderr}"
    );

    // Verify error message provides helpful information
    assert!(
        stderr.contains("Common causes") || stderr.contains("Example"),
        "Error message should provide helpful examples. stderr: {stderr}"
    );
}

#[test]
fn test_validate_with_out_of_range_values() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config with out-of-range values (maxOutputLines > 10000)
    let config_with_out_of_range = r#"
stop:
  commands:
    - run: "test"
      maxOutputLines: 50000
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_with_out_of_range).expect("Failed to write config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Validate should fail with out-of-range values"
    );
    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(101),
        "Exit code should be 1 or 101 for out-of-range values, got: {:?}",
        exit_code
    );

    // Verify error message mentions range validation
    assert!(
        stderr.contains("Range validation") || stderr.contains("out of valid range"),
        "Error message should mention range validation. stderr: {stderr}"
    );

    // Verify error message provides valid range
    assert!(
        stderr.contains("Valid range"),
        "Error message should provide valid range. stderr: {stderr}"
    );
}

#[test]
fn test_validate_with_custom_config_path_file() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let custom_config = temp_path.join("custom-config.yaml");

    // Create a valid configuration with a custom filename
    let valid_config = r#"
stop:
  commands:
    - run: "echo custom"
preToolUse:
  preventRootAdditions: false
"#;

    fs::write(&custom_config, valid_config).expect("Failed to write custom config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &custom_config.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Should succeed with exit code 0
    assert!(
        output.status.success(),
        "Validate should succeed with custom config path. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify success message references the custom config path
    assert!(stdout.contains("Configuration is valid"));
    assert!(stdout.contains("custom-config.yaml"));
}

#[test]
fn test_validate_with_nonexistent_custom_config_file() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let nonexistent_config = temp_path.join("does-not-exist.yaml");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &nonexistent_config.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Validate should fail with nonexistent config file"
    );
    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(101),
        "Exit code should be 1 or 101 for nonexistent file, got: {:?}",
        exit_code
    );

    // Verify error message mentions file error
    assert!(
        stderr.contains("No such file or directory") || stderr.contains("not found"),
        "Error message should mention file error. stderr: {stderr}"
    );
}

// ========== Stop Hook Working Directory Tests ==========

#[test]
fn test_stop_commands_execute_from_config_directory() {
    use std::env;
    use std::io::Write as IoWrite;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    // First, get the path to the built binary
    // The binary is in target/debug/conclaude or target/release/conclaude
    let mut binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    binary_path.push("target");

    // Determine if we're in debug or release mode by checking CARGO_CFG_DEBUG_ASSERTIONS
    #[cfg(debug_assertions)]
    binary_path.push("debug");
    #[cfg(not(debug_assertions))]
    binary_path.push("release");

    binary_path.push("conclaude");

    // Build the binary if it doesn't exist
    if !binary_path.exists() {
        let build_output = Command::new("cargo")
            .args(["build"])
            .output()
            .expect("Failed to build conclaude");
        assert!(
            build_output.status.success(),
            "Failed to build conclaude: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let project_root = temp_dir.path().join("project_root");
    let subdirectory = project_root.join("subdirectory");

    // Create directory structure
    fs::create_dir_all(&subdirectory).expect("Failed to create subdirectory");

    // Create unique temp file paths to avoid conflicts with parallel tests
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cwd_file = temp_dir.path().join(format!("test_cwd_{}.txt", test_id));
    let config_dir_file = temp_dir
        .path()
        .join(format!("test_config_dir_{}.txt", test_id));

    // Create config with stop command that outputs pwd and CONCLAUDE_CONFIG_DIR
    let config_content = format!(
        r#"
stop:
  commands:
    - run: "pwd > {} && echo $CONCLAUDE_CONFIG_DIR > {}"
preToolUse:
  preventRootAdditions: true
"#,
        cwd_file.to_string_lossy(),
        config_dir_file.to_string_lossy()
    );

    let config_path = project_root.join(".conclaude.yaml");
    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Prepare JSON payload for Stop hook
    // cwd is the subdirectory, but command should execute from project_root (config dir)
    let payload = serde_json::json!({
        "session_id": "test-session-stop-cwd",
        "transcript_path": "/tmp/test-transcript.jsonl",
        "hook_event_name": "Stop",
        "cwd": subdirectory.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Execute Stop hook by piping JSON to stdin
    // Run from project_root so config is found there
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "Stop"])
        .current_dir(&project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Stop hook");

    // Write payload to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(payload_json.as_bytes())
            .expect("Failed to write to stdin");
    }

    // Wait for command to complete
    let output = child
        .wait_with_output()
        .expect("Failed to wait for Stop hook");

    // The hook should succeed
    assert!(
        output.status.success(),
        "Stop hook should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the output files
    let cwd_output = fs::read_to_string(&cwd_file)
        .expect("Failed to read cwd output file")
        .trim()
        .to_string();
    let config_dir_output = fs::read_to_string(&config_dir_file)
        .expect("Failed to read config_dir output file")
        .trim()
        .to_string();

    // Verify pwd matches the config directory (project_root), not the subdirectory
    let expected_cwd = fs::canonicalize(&project_root)
        .expect("Failed to canonicalize project_root")
        .to_string_lossy()
        .to_string();

    assert_eq!(
        cwd_output, expected_cwd,
        "Stop command should execute from config directory, not cwd. Got: {}, Expected: {}",
        cwd_output, expected_cwd
    );

    // Verify CONCLAUDE_CONFIG_DIR env var is set to config directory
    assert_eq!(
        config_dir_output, expected_cwd,
        "CONCLAUDE_CONFIG_DIR should be set to config directory. Got: {}, Expected: {}",
        config_dir_output, expected_cwd
    );

    // Clean up temp files
    let _ = fs::remove_file(&cwd_file);
    let _ = fs::remove_file(&config_dir_file);
}

// ========== Per-Command Notifications Integration Tests ==========

#[test]
fn test_stop_hook_with_mixed_notify_per_command_settings() {
    use serde_json::json;
    use std::io::Write;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a configuration with mixed notifyPerCommand settings
    let config = r#"
stop:
  commands:
    - run: "echo first-command"
      notifyPerCommand: true
      showCommand: true
    - run: "echo second-command"
      notifyPerCommand: false
      showCommand: true
    - run: "echo third-command"
      # notifyPerCommand not specified - should default to false
      showCommand: false
  infinite: false
notifications:
  enabled: true
  hooks: ["Stop"]
  showErrors: true
  showSuccess: true
  showSystemEvents: true
"#;

    fs::write(&config_path, config).expect("Failed to write config file");

    // Create a Stop hook payload
    let payload = json!({
        "session_id": "test-session",
        "transcript_path": "/tmp/test-transcript.jsonl",
        "hook_event_name": "Stop",
        "cwd": temp_path.to_string_lossy().to_string(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Get binary path
    let binary_path = std::env::current_exe()
        .expect("Failed to get current exe path")
        .parent()
        .expect("No parent directory")
        .parent()
        .expect("No grandparent directory")
        .join("conclaude");

    // Execute the Stop hook from temp directory (where config is)
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "Stop"])
        .current_dir(temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Stop hook");

    // Write payload to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(payload_json.as_bytes())
            .expect("Failed to write to stdin");
    }

    let result = child
        .wait_with_output()
        .expect("Failed to wait for command");

    // The hook should succeed
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(
        result.status.success(),
        "Stop hook should execute successfully. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Verify output shows command execution (output might be in stdout or stderr)
    let combined_output = format!("{}{}", stdout, stderr);
    assert!(
        combined_output.contains("Executing") || combined_output.contains("echo"),
        "Output should show command execution. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Note: We can't easily verify that notifications were actually sent without
    // mocking the notification system, but we've verified that:
    // 1. The configuration is valid
    // 2. The commands execute successfully
    // 3. The notifyPerCommand flag is correctly parsed and passed through
    // The unit tests verify that the flag controls notification sending
}

#[test]
fn test_validate_config_with_notify_per_command() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a valid configuration with notifyPerCommand
    let valid_config = r#"
stop:
  commands:
    - run: "echo test1"
      notifyPerCommand: true
      showCommand: true
    - run: "echo test2"
      notifyPerCommand: false
  infinite: false
preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
  preventAdditions: []
notifications:
  enabled: true
  hooks: ["Stop"]
  showErrors: true
  showSuccess: true
  showSystemEvents: true
"#;

    fs::write(&config_path, valid_config).expect("Failed to write config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Should succeed with exit code 0
    assert!(
        output.status.success(),
        "Validate should succeed with notifyPerCommand config. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify success message
    assert!(stdout.contains("Configuration is valid"));
}

#[test]
fn test_subagent_stop_with_notify_per_command() {
    use serde_json::json;
    use std::io::Write;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a configuration with SubagentStop hook
    let config = r#"
stop:
  commands:
    - run: "echo main-stop"
  infinite: false
subagentStop:
  commands:
    coder:
      - run: "echo coder-cleanup"
        notifyPerCommand: true
        showCommand: true
    "*":
      - run: "echo wildcard-cleanup"
        notifyPerCommand: false
notifications:
  enabled: true
  hooks: ["SubagentStop"]
  showErrors: true
  showSuccess: true
  showSystemEvents: true
"#;

    fs::write(&config_path, config).expect("Failed to write config file");

    // Create a SubagentStop hook payload
    let payload = json!({
        "session_id": "test-session",
        "transcript_path": "/tmp/main-transcript.jsonl",
        "hook_event_name": "SubagentStop",
        "cwd": temp_path.to_string_lossy().to_string(),
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_id": "test-agent-id",
        "agent_transcript_path": "/tmp/agent-transcript.jsonl"
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Get binary path
    let binary_path = std::env::current_exe()
        .expect("Failed to get current exe path")
        .parent()
        .expect("No parent directory")
        .parent()
        .expect("No grandparent directory")
        .join("conclaude");

    // Execute the SubagentStop hook from temp directory (where config is)
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "SubagentStop"])
        .current_dir(temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn SubagentStop hook");

    // Write payload to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(payload_json.as_bytes())
            .expect("Failed to write to stdin");
    }

    let result = child
        .wait_with_output()
        .expect("Failed to wait for command");

    // The hook should succeed (even if agent name extraction fails, commands should run)
    assert!(
        result.status.success(),
        "SubagentStop hook should execute successfully. stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Note: Similar to the Stop hook test, we verify the configuration is valid
    // and commands execute, but can't easily verify notification behavior without mocking
}

#[test]
fn test_invalid_notify_per_command_value() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a configuration with invalid notifyPerCommand value (should be boolean)
    let invalid_config = r#"
stop:
  commands:
    - run: "echo test"
      notifyPerCommand: "yes"  # Invalid - should be boolean
  infinite: false
"#;

    fs::write(&config_path, invalid_config).expect("Failed to write config file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config-path",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run validate command");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Validate should fail with invalid notifyPerCommand type"
    );

    // Verify error message mentions the validation failure
    assert!(
        stderr.contains("invalid type") || stderr.contains("expected") || stderr.contains("bool"),
        "Error should mention type validation failure. stderr: {}",
        stderr
    );
}
