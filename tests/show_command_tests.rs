use std::env;
use std::fs;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::tempdir;

/// Helper function to get the path to the built conclaude binary
fn get_binary_path() -> PathBuf {
    let mut binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    binary_path.push("target");

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

    binary_path
}

#[test]
fn test_stop_show_command_true_prints_command() {
    let binary_path = get_binary_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).expect("Failed to create project directory");

    // Create config with showCommand: true (explicit)
    let config_content = r#"
stop:
  commands:
    - run: "echo 'Test command with showCommand true'"
      showCommand: true
preToolUse:
  preventRootAdditions: true
"#;

    let config_path = project_root.join(".conclaude.yaml");
    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Prepare JSON payload for Stop hook
    let payload = serde_json::json!({
        "session_id": "test-session-show-command-true",
        "transcript_path": "/tmp/test-transcript.jsonl",
        "hook_event_name": "Stop",
        "cwd": project_root.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Execute Stop hook
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed
    assert!(
        output.status.success(),
        "Stop hook should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify "Executing command X/Y: <command>" line IS printed
    assert!(
        stdout.contains("Executing command 1/1: echo 'Test command with showCommand true'"),
        "Output should contain 'Executing command X/Y: <command>' when showCommand is true. stdout:\n{}",
        stdout
    );
}

#[test]
fn test_stop_show_command_false_suppresses_command() {
    let binary_path = get_binary_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).expect("Failed to create project directory");

    // Create config with showCommand: false
    let config_content = r#"
stop:
  commands:
    - run: "echo 'Test command with showCommand false'"
      showCommand: false
preToolUse:
  preventRootAdditions: true
"#;

    let config_path = project_root.join(".conclaude.yaml");
    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Prepare JSON payload for Stop hook
    let payload = serde_json::json!({
        "session_id": "test-session-show-command-false",
        "transcript_path": "/tmp/test-transcript.jsonl",
        "hook_event_name": "Stop",
        "cwd": project_root.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Execute Stop hook
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed
    assert!(
        output.status.success(),
        "Stop hook should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the actual command string is NOT printed
    assert!(
        !stdout.contains("echo 'Test command with showCommand false'"),
        "Output should NOT contain the command string when showCommand is false. stdout:\n{}",
        stdout
    );

    // Verify only "Executing command X/Y" (without command) is printed
    assert!(
        stdout.contains("Executing command 1/1"),
        "Output should contain 'Executing command X/Y' without the command. stdout:\n{}",
        stdout
    );
}

#[test]
fn test_stop_show_command_default_prints_command() {
    let binary_path = get_binary_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).expect("Failed to create project directory");

    // Create config without showCommand (should default to true)
    let config_content = r#"
stop:
  commands:
    - run: "echo 'Test command with default showCommand'"
preToolUse:
  preventRootAdditions: true
"#;

    let config_path = project_root.join(".conclaude.yaml");
    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Prepare JSON payload for Stop hook
    let payload = serde_json::json!({
        "session_id": "test-session-show-command-default",
        "transcript_path": "/tmp/test-transcript.jsonl",
        "hook_event_name": "Stop",
        "cwd": project_root.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Execute Stop hook
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed
    assert!(
        output.status.success(),
        "Stop hook should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify "Executing command X/Y: <command>" line IS printed (default behavior)
    assert!(
        stdout.contains("Executing command 1/1: echo 'Test command with default showCommand'"),
        "Output should contain 'Executing command X/Y: <command>' when showCommand is not specified (default true). stdout:\n{}",
        stdout
    );
}

#[test]
fn test_subagent_stop_show_command_true_prints_command() {
    let binary_path = get_binary_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).expect("Failed to create project directory");

    // Create config with subagentStop commands and showCommand: true
    let config_content = r#"
stop:
  commands: []
subagentStop:
  commands:
    test-agent:
      - run: "echo 'Subagent command with showCommand true'"
        showCommand: true
preToolUse:
  preventRootAdditions: true
"#;

    let config_path = project_root.join(".conclaude.yaml");
    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Create transcript files (required for agent name extraction)
    let transcript_path = temp_dir.path().join("test-transcript.jsonl");
    fs::write(&transcript_path, "").expect("Failed to create transcript file");
    let agent_transcript_path = temp_dir.path().join("test-agent-transcript.jsonl");
    fs::write(&agent_transcript_path, "").expect("Failed to create agent transcript file");

    // Prepare JSON payload for SubagentStop hook
    let payload = serde_json::json!({
        "session_id": "test-session-subagent-show-true",
        "transcript_path": transcript_path.to_string_lossy(),
        "hook_event_name": "SubagentStop",
        "cwd": project_root.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_id": "test-agent",
        "agent_transcript_path": agent_transcript_path.to_string_lossy()
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Execute SubagentStop hook
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "SubagentStop"])
        .current_dir(&project_root)
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

    // Wait for command to complete
    let output = child
        .wait_with_output()
        .expect("Failed to wait for SubagentStop hook");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed
    assert!(
        output.status.success(),
        "SubagentStop hook should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify "Executing subagent stop command X/Y: <command>" line IS printed
    assert!(
        stdout.contains("Executing subagent stop command 1/1: echo 'Subagent command with showCommand true'"),
        "Output should contain 'Executing subagent stop command X/Y: <command>' when showCommand is true. stdout:\n{}",
        stdout
    );
}

#[test]
fn test_subagent_stop_show_command_false_suppresses_command() {
    let binary_path = get_binary_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).expect("Failed to create project directory");

    // Create config with subagentStop commands and showCommand: false
    let config_content = r#"
stop:
  commands: []
subagentStop:
  commands:
    test-agent:
      - run: "echo 'Subagent command with showCommand false'"
        showCommand: false
preToolUse:
  preventRootAdditions: true
"#;

    let config_path = project_root.join(".conclaude.yaml");
    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Create transcript files (required for agent name extraction)
    let transcript_path = temp_dir.path().join("test-transcript.jsonl");
    fs::write(&transcript_path, "").expect("Failed to create transcript file");
    let agent_transcript_path = temp_dir.path().join("test-agent-transcript.jsonl");
    fs::write(&agent_transcript_path, "").expect("Failed to create agent transcript file");

    // Prepare JSON payload for SubagentStop hook
    let payload = serde_json::json!({
        "session_id": "test-session-subagent-show-false",
        "transcript_path": transcript_path.to_string_lossy(),
        "hook_event_name": "SubagentStop",
        "cwd": project_root.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_id": "test-agent",
        "agent_transcript_path": agent_transcript_path.to_string_lossy()
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Execute SubagentStop hook
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "SubagentStop"])
        .current_dir(&project_root)
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

    // Wait for command to complete
    let output = child
        .wait_with_output()
        .expect("Failed to wait for SubagentStop hook");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed
    assert!(
        output.status.success(),
        "SubagentStop hook should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the actual command string is NOT printed
    assert!(
        !stdout.contains("echo 'Subagent command with showCommand false'"),
        "Output should NOT contain the command string when showCommand is false. stdout:\n{}",
        stdout
    );

    // Verify only "Executing subagent stop command X/Y" (without command) is printed
    assert!(
        stdout.contains("Executing subagent stop command 1/1"),
        "Output should contain 'Executing subagent stop command X/Y' without the command. stdout:\n{}",
        stdout
    );
}

#[test]
fn test_subagent_stop_show_command_default_prints_command() {
    let binary_path = get_binary_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).expect("Failed to create project directory");

    // Create config with subagentStop commands without showCommand (should default to true)
    let config_content = r#"
stop:
  commands: []
subagentStop:
  commands:
    test-agent:
      - run: "echo 'Subagent command with default showCommand'"
preToolUse:
  preventRootAdditions: true
"#;

    let config_path = project_root.join(".conclaude.yaml");
    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Create transcript files (required for agent name extraction)
    let transcript_path = temp_dir.path().join("test-transcript.jsonl");
    fs::write(&transcript_path, "").expect("Failed to create transcript file");
    let agent_transcript_path = temp_dir.path().join("test-agent-transcript.jsonl");
    fs::write(&agent_transcript_path, "").expect("Failed to create agent transcript file");

    // Prepare JSON payload for SubagentStop hook
    let payload = serde_json::json!({
        "session_id": "test-session-subagent-show-default",
        "transcript_path": transcript_path.to_string_lossy(),
        "hook_event_name": "SubagentStop",
        "cwd": project_root.to_string_lossy(),
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_id": "test-agent",
        "agent_transcript_path": agent_transcript_path.to_string_lossy()
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Execute SubagentStop hook
    let mut child = Command::new(&binary_path)
        .args(["Hooks", "SubagentStop"])
        .current_dir(&project_root)
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

    // Wait for command to complete
    let output = child
        .wait_with_output()
        .expect("Failed to wait for SubagentStop hook");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed
    assert!(
        output.status.success(),
        "SubagentStop hook should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify "Executing subagent stop command X/Y: <command>" line IS printed (default behavior)
    assert!(
        stdout.contains("Executing subagent stop command 1/1: echo 'Subagent command with default showCommand'"),
        "Output should contain 'Executing subagent stop command X/Y: <command>' when showCommand is not specified (default true). stdout:\n{}",
        stdout
    );
}
