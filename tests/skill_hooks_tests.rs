use std::fs;
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[test]
fn test_cli_init_with_commands_and_skills() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let claude_path = temp_path.join(".claude");
    let commands_path = claude_path.join("commands");
    let skills_path = claude_path.join("skills");

    // Create directories
    fs::create_dir_all(&commands_path).expect("Failed to create commands directory");
    fs::create_dir_all(&skills_path).expect("Failed to create skills directory");

    // Create a dummy command file
    let command_file = commands_path.join("test-command.md");
    fs::write(
        &command_file,
        "---\nname: test-command\n---\n# Test Command",
    )
    .expect("Failed to write command file");

    // Create a dummy skill file
    let skill_file = skills_path.join("test-skill.md");
    fs::write(&skill_file, "---\nname: test-skill\n---\n# Test Skill")
        .expect("Failed to write skill file");

    // Run conclaude init
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "init",
            "--config-path",
            &temp_path.join(".conclaude.yaml").to_string_lossy(),
            "--claude-path",
            &claude_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to run CLI init command");

    assert!(output.status.success(), "Init command should succeed");

    // Verify hooks were injected into command file
    let command_content = fs::read_to_string(&command_file).expect("Failed to read command file");
    assert!(command_content.contains("hooks:"));
    assert!(command_content.contains("conclaude Hooks PreToolUse --skill test-command"));

    // Verify hooks were injected into skill file
    let skill_content = fs::read_to_string(&skill_file).expect("Failed to read skill file");
    assert!(skill_content.contains("hooks:"));
    assert!(skill_content.contains("conclaude Hooks PreToolUse --skill test-skill"));
}

#[test]
fn test_skill_flag_sets_env_var() {
    use std::io::Write;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join(".conclaude.yaml");

    // Create a config that outputs CONCLAUDE_SKILL_NAME to a file
    let output_file = temp_path.join("skill_env.txt");
    let config = format!(
        r#"
stop:
  commands:
    - run: "echo $CONCLAUDE_SKILL_NAME > {}"
      skill: "tester"
preToolUse:
  preventRootAdditions: false
"#,
        output_file.to_string_lossy()
    );

    fs::write(&config_path, config).expect("Failed to write config file");

    // Create a Stop hook payload
    let payload = serde_json::json!({
        "session_id": "test-session",
        "transcript_path": "/tmp/test-transcript.jsonl",
        "hook_event_name": "Stop",
        "cwd": temp_path.to_string_lossy().to_string(),
        "permission_mode": "default",
        "stop_hook_active": true
    });

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    // Get binary path from cargo

    let binary_path = env!("CARGO_BIN_EXE_conclaude");

    // Execute the Stop hook with --skill tester

    let mut child = Command::new(binary_path)
        .args(["Hooks", "Stop", "--skill", "tester"])
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
    if !result.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&result.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&result.stderr));
    }
    assert!(result.status.success());

    // Verify env var was set and used
    let env_output = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert_eq!(env_output.trim(), "tester");
}
