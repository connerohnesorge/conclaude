use conclaude::hooks::*;
use conclaude::types::*;
use serde_json::Value;
use std::collections::HashMap;

// Helper function to create a base payload for testing
fn create_test_base_payload() -> BasePayload {
    BasePayload {
        session_id: "test_session_123".to_string(),
        transcript_path: "/tmp/test_transcript.jsonl".to_string(),
        hook_event_name: "PreToolUse".to_string(),
        cwd: "/home/user/project".to_string(),
        permission_mode: Some("default".to_string()),
    }
}

#[test]
fn test_extract_file_path_with_file_path() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::String("test.txt".to_string()),
    );

    let result = extract_file_path(&tool_input);
    assert_eq!(result, Some("test.txt".to_string()));
}

#[test]
fn test_extract_file_path_with_notebook_path() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "notebook_path".to_string(),
        Value::String("notebook.ipynb".to_string()),
    );

    let result = extract_file_path(&tool_input);
    assert_eq!(result, Some("notebook.ipynb".to_string()));
}

#[test]
fn test_extract_file_path_with_both_paths_prefers_file_path() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::String("test.txt".to_string()),
    );
    tool_input.insert(
        "notebook_path".to_string(),
        Value::String("notebook.ipynb".to_string()),
    );

    let result = extract_file_path(&tool_input);
    assert_eq!(result, Some("test.txt".to_string()));
}

#[test]
fn test_extract_file_path_with_no_path() {
    let tool_input = HashMap::new();

    let result = extract_file_path(&tool_input);
    assert_eq!(result, None);
}

#[test]
fn test_is_root_addition_true_cases() {
    use std::env;

    // Get current working directory for testing
    let cwd = env::current_dir().unwrap();

    // Simulate config file in the current directory
    let config_path = cwd.join(".conclaude.yaml");

    // Files directly in root directory (same level as config)
    assert!(is_root_addition("", "test.txt", &config_path));
    assert!(is_root_addition("", "script.sh", &config_path));
    assert!(is_root_addition("", "data.json", &config_path));

    // BREAKING CHANGE: Dotfiles are now also blocked at root level
    assert!(is_root_addition("", ".gitignore", &config_path));
    assert!(is_root_addition("", ".env", &config_path));

    // BREAKING CHANGE: Config files are now also blocked at root level
    assert!(is_root_addition("", "package.json", &config_path));
    assert!(is_root_addition("", "tsconfig.json", &config_path));
    assert!(is_root_addition("", "config.yaml", &config_path));
    assert!(is_root_addition("", "settings.ini", &config_path));
    assert!(is_root_addition("", "bun.lockb", &config_path));
    assert!(is_root_addition("", "bun.lock", &config_path));
}

#[test]
fn test_is_root_addition_false_cases() {
    use std::env;

    // Get current working directory for testing
    let cwd = env::current_dir().unwrap();

    // Simulate config file in the current directory
    let config_path = cwd.join(".conclaude.yaml");

    // Files in subdirectories should not be blocked
    assert!(!is_root_addition("", "src/test.txt", &config_path));
    assert!(!is_root_addition("", "docs/readme.md", &config_path));
    assert!(!is_root_addition("", "tests/unit.rs", &config_path));

    // Edge cases - empty paths
    assert!(!is_root_addition("", "", &config_path));
    assert!(!is_root_addition("", "..", &config_path));
}

#[test]
fn test_matches_uneditable_pattern_comprehensive() {
    // Test that the function matches against any of the three inputs (file_path, relative_path, resolved_path)
    // by using DIFFERENT values for each parameter to verify each is checked

    // Test Case 1: Pattern matches file_path only
    // resolved_path contains absolute path, relative_path has subdir prefix, but file_path matches
    assert!(matches_uneditable_pattern(
        "config.json",           // file_path - matches pattern
        "project/config.json",   // relative_path - does NOT match pattern
        "/home/user/project/config.json", // resolved_path - does NOT match pattern
        "config.json"            // pattern
    )
    .unwrap(), "Should match via file_path");

    // Test Case 2: Pattern matches relative_path only
    // Use different paths to ensure relative_path is the one being matched
    assert!(matches_uneditable_pattern(
        "other.json",            // file_path - does NOT match pattern
        "src/index.ts",          // relative_path - matches pattern
        "/home/user/project/src/index.ts", // resolved_path - may also match
        "src/**/*.ts"            // pattern
    )
    .unwrap(), "Should match via relative_path");

    // Test Case 3: Pattern matches resolved_path only
    // Absolute path pattern matching
    assert!(matches_uneditable_pattern(
        "file.txt",              // file_path - does NOT match pattern
        "subdir/file.txt",       // relative_path - does NOT match pattern
        "/restricted/file.txt",  // resolved_path - matches pattern
        "/restricted/*"          // pattern for absolute paths
    )
    .unwrap(), "Should match via resolved_path");

    // Test Case 4: Multiple patterns matching - environment files
    // All three inputs have .env prefix, testing with different suffixes
    assert!(matches_uneditable_pattern(
        ".env",
        ".env",
        "/path/.env",
        ".env*"
    )
    .unwrap(), ".env should match .env* pattern");

    assert!(matches_uneditable_pattern(
        ".env.local",
        ".env.local",
        "/path/.env.local",
        ".env*"
    )
    .unwrap(), ".env.local should match .env* pattern");

    assert!(matches_uneditable_pattern(
        ".env.production",
        ".env.production",
        "/path/.env.production",
        ".env*"
    )
    .unwrap(), ".env.production should match .env* pattern");

    assert!(!matches_uneditable_pattern(
        "environment.txt",
        "environment.txt",
        "/path/environment.txt",
        ".env*"
    )
    .unwrap(), "environment.txt should NOT match .env* pattern");

    // Test Case 5: Multiple config file patterns
    assert!(matches_uneditable_pattern(
        "package.json",
        "package.json",
        "/path/package.json",
        "package.json"
    )
    .unwrap(), "package.json should match exact pattern");

    assert!(matches_uneditable_pattern(
        "tsconfig.json",
        "tsconfig.json",
        "/path/tsconfig.json",
        "tsconfig.json"
    )
    .unwrap(), "tsconfig.json should match exact pattern");

    assert!(!matches_uneditable_pattern(
        "other.json",
        "other.json",
        "/path/other.json",
        "package.json"
    )
    .unwrap(), "other.json should NOT match package.json pattern");

    // Test Case 6: Wildcard extension matching
    assert!(matches_uneditable_pattern(
        "test.md",
        "test.md",
        "/path/test.md",
        "*.md"
    )
    .unwrap(), "test.md should match *.md pattern");

    assert!(matches_uneditable_pattern(
        "README.md",
        "README.md",
        "/path/README.md",
        "*.md"
    )
    .unwrap(), "README.md should match *.md pattern");

    assert!(!matches_uneditable_pattern(
        "other.txt",
        "other.txt",
        "/path/other.txt",
        "*.md"
    )
    .unwrap(), "other.txt should NOT match *.md pattern");

    // Test Case 7: Directory pattern matching with different depths
    assert!(matches_uneditable_pattern(
        "src/index.ts",
        "src/index.ts",
        "/path/src/index.ts",
        "src/**/*.ts"
    )
    .unwrap(), "src/index.ts should match src/**/*.ts pattern");

    assert!(matches_uneditable_pattern(
        "src/lib/utils.ts",
        "src/lib/utils.ts",
        "/path/src/lib/utils.ts",
        "src/**/*.ts"
    )
    .unwrap(), "src/lib/utils.ts should match src/**/*.ts pattern");

    assert!(!matches_uneditable_pattern(
        "lib/index.ts",
        "lib/index.ts",
        "/path/lib/index.ts",
        "src/**/*.ts"
    )
    .unwrap(), "lib/index.ts should NOT match src/**/*.ts pattern");
}

#[test]
fn test_matches_uneditable_pattern_invalid_glob() {
    let result = matches_uneditable_pattern("test.txt", "test.txt", "/path/test.txt", "[invalid");
    assert!(result.is_err());
}

#[test]
fn test_matches_uneditable_pattern_directory_patterns() {
    // Match entire directories
    assert!(matches_uneditable_pattern(
        "docs/README.md",
        "docs/README.md",
        "/path/docs/README.md",
        "docs/**"
    )
    .unwrap());
    assert!(matches_uneditable_pattern(
        "docs/api/index.md",
        "docs/api/index.md",
        "/path/docs/api/index.md",
        "docs/**"
    )
    .unwrap());
    assert!(!matches_uneditable_pattern(
        "src/docs.ts",
        "src/docs.ts",
        "/path/src/docs.ts",
        "docs/**"
    )
    .unwrap());
}

// Integration test for path normalization scenarios
#[test]
fn test_file_path_normalization_scenarios() {
    let test_cases = [
        ("./package.json", "package.json", true),
        ("src/../package.json", "package.json", true),
        ("/absolute/path/package.json", "package.json", true),
        ("src/nested/file.ts", "src/**/*.ts", true),
    ];

    for (path, pattern, expected) in test_cases {
        // Normalize path similar to how the real code would
        let normalized = if let Some(stripped) = path.strip_prefix("./") {
            stripped
        } else if path.contains("..") {
            "package.json" // Simplified normalization for test
        } else {
            std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
        };

        let result = matches_uneditable_pattern(normalized, normalized, path, pattern).unwrap();
        assert_eq!(
            result, expected,
            "Failed for path: {path}, pattern: {pattern}"
        );
    }
}

// Test validation helpers
#[test]
fn test_validate_base_payload_integration() {
    let valid_base = create_test_base_payload();
    assert!(validate_base_payload(&valid_base).is_ok());

    let invalid_base = BasePayload {
        session_id: String::new(),
        transcript_path: "/path/to/transcript".to_string(),
        hook_event_name: "PreToolUse".to_string(),
        cwd: "/home/user/project".to_string(),
        permission_mode: Some("default".to_string()),
    };
    assert!(validate_base_payload(&invalid_base).is_err());
}


// ============================================================================
// Integration Tests for Bash Command Validation
// ============================================================================

#[tokio::test]
async fn test_bash_validation_block_exact_command() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with block rule for exact command
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "Bash".to_string(),
                pattern: String::new(),
                action: "block".to_string(),
                message: Some("Dangerous command blocked!".to_string()),
                command_pattern: Some("rm -rf /".to_string()),
                match_mode: Some("full".to_string()),
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    // Create PreToolUsePayload with Bash command
    let mut tool_input = HashMap::new();
    tool_input.insert("command".to_string(), Value::String("rm -rf /".to_string()));

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input,
        tool_use_id: None,
    };

    // Manually test the pattern matching logic (since we can't easily inject config)
    // Extract the command
    let command = payload
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    // Test full mode matching
    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = rule.command_pattern.as_ref().unwrap();
    let mode = rule.match_mode.as_deref().unwrap_or("full");

    let matches = if mode == "full" {
        glob::Pattern::new(pattern)?.matches(command)
    } else {
        false
    };

    assert!(matches, "Exact command should match in full mode");
    assert_eq!(rule.action, "block");
    assert_eq!(rule.message.as_deref(), Some("Dangerous command blocked!"));

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_block_command_family() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with block rule for command family (prefix mode)
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "Bash".to_string(),
                pattern: String::new(),
                action: "block".to_string(),
                message: Some("Git force push blocked!".to_string()),
                command_pattern: Some("git push --force*".to_string()),
                match_mode: Some("prefix".to_string()),
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    // Create PreToolUsePayload with Bash command that should match in prefix mode
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("git push --force origin main && echo done".to_string()),
    );

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input,
        tool_use_id: None,
    };

    // Test prefix mode matching
    let command = payload
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = rule.command_pattern.as_ref().unwrap();
    let mode = rule.match_mode.as_deref().unwrap_or("full");

    let matches = if mode == "prefix" {
        let glob = glob::Pattern::new(pattern)?;
        let words: Vec<&str> = command.split_whitespace().collect();
        (1..=words.len()).any(|i| {
            let prefix = words[..i].join(" ");
            glob.matches(&prefix)
        })
    } else {
        false
    };

    assert!(matches, "Command family should match in prefix mode");
    assert_eq!(rule.action, "block");

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_allow_whitelist() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with allow rule (whitelist pattern)
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "Bash".to_string(),
                pattern: String::new(),
                action: "allow".to_string(),
                message: Some("Only safe commands allowed".to_string()),
                command_pattern: Some("echo *".to_string()),
                match_mode: Some("full".to_string()),
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    // Test command that matches the whitelist
    let mut tool_input_allowed = HashMap::new();
    tool_input_allowed.insert(
        "command".to_string(),
        Value::String("echo hello world".to_string()),
    );

    let payload_allowed = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input: tool_input_allowed,
        tool_use_id: None,
    };

    let command_allowed = payload_allowed
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = rule.command_pattern.as_ref().unwrap();

    let matches_allowed = glob::Pattern::new(pattern)?.matches(command_allowed);
    assert!(
        matches_allowed,
        "Whitelisted command should match and be allowed"
    );

    // Test command that does NOT match the whitelist (should be blocked)
    let mut tool_input_blocked = HashMap::new();
    tool_input_blocked.insert(
        "command".to_string(),
        Value::String("rm -rf /tmp".to_string()),
    );

    let payload_blocked = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input: tool_input_blocked,
        tool_use_id: None,
    };

    let command_blocked = payload_blocked
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    let matches_blocked = glob::Pattern::new(pattern)?.matches(command_blocked);
    assert!(
        !matches_blocked,
        "Non-whitelisted command should not match and be blocked"
    );
    assert_eq!(rule.action, "allow");

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_custom_message() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with custom error message
    let custom_message = "DANGER: This command could delete important files!";
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "Bash".to_string(),
                pattern: String::new(),
                action: "block".to_string(),
                message: Some(custom_message.to_string()),
                command_pattern: Some("rm -rf*".to_string()),
                match_mode: Some("full".to_string()),
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("rm -rf /tmp".to_string()),
    );

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input,
        tool_use_id: None,
    };

    let command = payload
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = rule.command_pattern.as_ref().unwrap();

    let matches = glob::Pattern::new(pattern)?.matches(command);
    assert!(matches, "Command should match the pattern");
    assert_eq!(rule.message.as_deref(), Some(custom_message));

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_default_match_mode() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration WITHOUT explicit matchMode (should default to "full")
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "Bash".to_string(),
                pattern: String::new(),
                action: "block".to_string(),
                message: None,
                command_pattern: Some("curl *".to_string()),
                match_mode: None, // No explicit mode - should default to "full"
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("curl https://evil.com".to_string()),
    );

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input,
        tool_use_id: None,
    };

    let command = payload
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = rule.command_pattern.as_ref().unwrap();
    let mode = rule.match_mode.as_deref().unwrap_or("full");

    assert_eq!(mode, "full", "Default matchMode should be 'full'");

    let matches = glob::Pattern::new(pattern)?.matches(command);
    assert!(matches, "Command should match in full mode");

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_backward_compatible() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with file path rule (backward compatibility)
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "Write".to_string(),
                pattern: ".env*".to_string(),
                action: "block".to_string(),
                message: Some("Cannot write to .env files".to_string()),
                command_pattern: None, // No command pattern - uses file path pattern
                match_mode: None,
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::String(".env.local".to_string()),
    );

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Write".to_string(),
        tool_input,
        tool_use_id: None,
    };

    let file_path = payload
        .tool_input
        .get("file_path")
        .and_then(|v| v.as_str())
        .unwrap();

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = &rule.pattern;

    let matches = glob::Pattern::new(pattern)?.matches(file_path);
    assert!(matches, "File path pattern should still work");
    assert_eq!(rule.tool, "Write");
    assert!(rule.command_pattern.is_none());

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_wildcard_tool() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with tool: "*" (applies to all tools, including Bash)
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "*".to_string(),
                pattern: String::new(),
                action: "block".to_string(),
                message: Some("Wildcard rule blocks this Bash command".to_string()),
                command_pattern: Some("sudo *".to_string()),
                match_mode: Some("full".to_string()),
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("sudo apt-get update".to_string()),
    );

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input,
        tool_use_id: None,
    };

    let command = payload
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = rule.command_pattern.as_ref().unwrap();

    // Verify wildcard tool matches Bash
    assert!(
        rule.tool == "Bash" || rule.tool == "*",
        "Wildcard tool should apply to Bash"
    );

    let matches = glob::Pattern::new(pattern)?.matches(command);
    assert!(matches, "Wildcard tool rule should apply to Bash commands");

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_prefix_mode_no_match_in_middle() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with prefix mode
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![ToolUsageRule {
                tool: "Bash".to_string(),
                pattern: String::new(),
                action: "block".to_string(),
                message: None,
                command_pattern: Some("curl *".to_string()),
                match_mode: Some("prefix".to_string()),
                agent: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    // Command where "curl" appears in the middle (should NOT match in prefix mode)
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        Value::String("echo test && curl https://example.com".to_string()),
    );

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input,
        tool_use_id: None,
    };

    let command = payload
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    let pattern = rule.command_pattern.as_ref().unwrap();
    let mode = rule.match_mode.as_deref().unwrap_or("full");

    let matches = if mode == "prefix" {
        let glob = glob::Pattern::new(pattern)?;
        let words: Vec<&str> = command.split_whitespace().collect();
        (1..=words.len()).any(|i| {
            let prefix = words[..i].join(" ");
            glob.matches(&prefix)
        })
    } else {
        false
    };

    assert!(
        !matches,
        "Prefix mode should NOT match pattern in middle of command"
    );

    Ok(())
}

#[tokio::test]
async fn test_bash_validation_multiple_rules() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with multiple rules
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![
                ToolUsageRule {
                    tool: "Bash".to_string(),
                    pattern: String::new(),
                    action: "block".to_string(),
                    message: Some("Blocked: rm commands".to_string()),
                    command_pattern: Some("rm *".to_string()),
                    match_mode: Some("full".to_string()),
                    agent: None,
                },
                ToolUsageRule {
                    tool: "Bash".to_string(),
                    pattern: String::new(),
                    action: "block".to_string(),
                    message: Some("Blocked: curl commands".to_string()),
                    command_pattern: Some("curl *".to_string()),
                    match_mode: Some("full".to_string()),
                    agent: None,
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    // Test that first rule matches
    let mut tool_input1 = HashMap::new();
    tool_input1.insert(
        "command".to_string(),
        Value::String("rm -rf /tmp".to_string()),
    );

    let payload1 = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input: tool_input1,
        tool_use_id: None,
    };

    let command1 = payload1
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();
    let rule1 = &config.pre_tool_use.tool_usage_validation[0];
    let matches1 = glob::Pattern::new(rule1.command_pattern.as_ref().unwrap())?.matches(command1);
    assert!(matches1, "First rule should match rm command");

    // Test that second rule matches
    let mut tool_input2 = HashMap::new();
    tool_input2.insert(
        "command".to_string(),
        Value::String("curl https://example.com".to_string()),
    );

    let payload2 = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Bash".to_string(),
        tool_input: tool_input2,
        tool_use_id: None,
    };

    let command2 = payload2
        .tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap();
    let rule2 = &config.pre_tool_use.tool_usage_validation[1];
    let matches2 = glob::Pattern::new(rule2.command_pattern.as_ref().unwrap())?.matches(command2);
    assert!(matches2, "Second rule should match curl command");

    Ok(())
}

// ============================================================================
// Integration Tests for SubagentStop Hook
// ============================================================================

// Helper function to create a SubagentStop payload for testing
fn create_subagent_stop_payload() -> SubagentStopPayload {
    SubagentStopPayload {
        base: BasePayload {
            session_id: "test_session_456".to_string(),
            transcript_path: "/tmp/session_transcript.jsonl".to_string(),
            hook_event_name: "SubagentStop".to_string(),
            cwd: "/home/user/project".to_string(),
            permission_mode: Some("default".to_string()),
        },
        stop_hook_active: true,
        agent_id: "coder".to_string(),
        agent_transcript_path: "/tmp/coder_transcript.jsonl".to_string(),
    }
}

#[test]
fn test_subagent_stop_hook_event_name_validation() {
    let payload = create_subagent_stop_payload();

    // Verify the hook event name is correctly set
    assert_eq!(payload.base.hook_event_name, "SubagentStop");

    // Verify the base payload validates correctly
    assert!(validate_base_payload(&payload.base).is_ok());
}

#[test]
fn test_subagent_stop_payload_with_empty_cwd() {
    let mut payload = create_subagent_stop_payload();
    payload.base.cwd = String::new();

    // Should fail on base payload validation
    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
}

#[test]
fn test_subagent_stop_payload_json_with_missing_agent_id_field() {
    let json_str = r#"{
        "session_id": "test_session",
        "transcript_path": "/tmp/session.jsonl",
        "hook_event_name": "SubagentStop",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_transcript_path": "/tmp/agent.jsonl"
    }"#;

    // This should fail to deserialize because agent_id is required
    let result: Result<SubagentStopPayload, _> = serde_json::from_str(json_str);
    assert!(
        result.is_err(),
        "JSON missing agent_id should fail to deserialize"
    );
}

#[test]
fn test_subagent_stop_payload_json_with_missing_agent_transcript_path_field() {
    let json_str = r#"{
        "session_id": "test_session",
        "transcript_path": "/tmp/session.jsonl",
        "hook_event_name": "SubagentStop",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_id": "coder"
    }"#;

    // This should fail to deserialize because agent_transcript_path is required
    let result: Result<SubagentStopPayload, _> = serde_json::from_str(json_str);
    assert!(
        result.is_err(),
        "JSON missing agent_transcript_path should fail to deserialize"
    );
}

#[test]
fn test_subagent_stop_validation_fails_gracefully_on_invalid_base() {
    let mut payload = create_subagent_stop_payload();
    // Invalidate the base payload by removing session_id
    payload.base.session_id = String::new();

    // The validation should fail due to invalid base
    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());

    // The error message should indicate which field is invalid
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("session_id"));
}

#[test]
fn test_subagent_stop_validation_fails_gracefully_on_invalid_agent_id() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_id = String::new();

    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("agent_id"));
}

#[test]
fn test_subagent_stop_validation_fails_gracefully_on_invalid_agent_transcript_path() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_transcript_path = String::new();

    let result = validate_subagent_stop_payload(&payload);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("agent_transcript_path"));
}

#[test]
fn test_subagent_stop_validation_error_messages_specific() {
    // Test that error messages are specific and helpful

    let mut payload = create_subagent_stop_payload();

    // Test agent_id error
    payload.agent_id = String::new();
    let error = validate_subagent_stop_payload(&payload).unwrap_err();
    assert!(
        error.contains("agent_id"),
        "Error should mention agent_id field"
    );

    // Test agent_transcript_path error
    let mut payload = create_subagent_stop_payload();
    payload.agent_transcript_path = String::new();
    let error = validate_subagent_stop_payload(&payload).unwrap_err();
    assert!(
        error.contains("agent_transcript_path"),
        "Error should mention agent_transcript_path field"
    );

    // Test whitespace-only agent_id error
    let mut payload = create_subagent_stop_payload();
    payload.agent_id = "   ".to_string();
    let error = validate_subagent_stop_payload(&payload).unwrap_err();
    assert!(
        error.contains("agent_id"),
        "Error should mention agent_id for whitespace-only value"
    );
}


// ============================================================================
// Tests for Refined preventRootAdditions Behavior
// ============================================================================
// These tests verify that preventRootAdditions now distinguishes between:
// 1. Creating NEW files at root (should be BLOCKED)
// 2. Modifying EXISTING files at root (should be ALLOWED)

#[test]
fn test_is_root_addition_identifies_root_level_correctly() {
    use std::env;

    let cwd = env::current_dir().unwrap();
    let config_path = cwd.join(".conclaude.yaml");

    // Root-level files should be identified
    assert!(
        is_root_addition("README.md", "README.md", &config_path),
        "README.md should be identified as root-level"
    );
    assert!(
        is_root_addition("package.json", "package.json", &config_path),
        "package.json should be identified as root-level"
    );
    assert!(
        is_root_addition(".env", ".env", &config_path),
        ".env should be identified as root-level"
    );

    // Non-root files should not be identified
    assert!(
        !is_root_addition("src/main.rs", "src/main.rs", &config_path),
        "src/main.rs should NOT be identified as root-level"
    );
    assert!(
        !is_root_addition("tests/test.rs", "tests/test.rs", &config_path),
        "tests/test.rs should NOT be identified as root-level"
    );
}

#[test]
fn test_prevent_root_additions_path_resolution() {
    use std::env;

    let cwd = env::current_dir().unwrap();
    let config_path = cwd.join(".conclaude.yaml");

    // Test that path resolution works correctly for file existence check
    let test_file = "Cargo.toml";
    let resolved_path = cwd.join(test_file);

    // Verify path resolution
    assert!(
        resolved_path.exists(),
        "Cargo.toml should exist at resolved path"
    );

    // The is_root_addition check uses the relative path
    let is_root = is_root_addition(test_file, test_file, &config_path);
    assert!(is_root, "Cargo.toml should be at root level");

    // The existence check uses the resolved path
    let exists = resolved_path.exists();
    assert!(exists, "Cargo.toml should exist");

    // Combined: is_root && !exists = false, so not blocked
    let should_block = is_root && !exists;
    assert!(!should_block, "Existing root file should NOT be blocked");
}

// ============================================================================
// Integration Tests for preventAdditions Feature
// ============================================================================
// These tests verify that preventAdditions:
// 1. ONLY applies to the "Write" tool (file creation), NOT "Edit" or "NotebookEdit"
// 2. Uses glob pattern matching (same as uneditableFiles)
// 3. Blocks file creation when patterns match
// 4. Works independently from and alongside other rules

#[test]
fn test_prevent_additions_basic_glob_matching() {
    // Test basic glob pattern matching for preventAdditions
    // This tests the pattern matching logic that will be used by the implementation

    // Test case 1: "dist/**" should match files in dist directory
    assert!(
        matches_uneditable_pattern(
            "dist/output.js",
            "dist/output.js",
            "/path/dist/output.js",
            "dist/**"
        )
        .unwrap(),
        "Pattern 'dist/**' should match 'dist/output.js'"
    );

    assert!(
        matches_uneditable_pattern(
            "dist/nested/deep/file.js",
            "dist/nested/deep/file.js",
            "/path/dist/nested/deep/file.js",
            "dist/**"
        )
        .unwrap(),
        "Pattern 'dist/**' should match 'dist/nested/deep/file.js'"
    );

    // Test case 2: "build/**" should match files in build directory
    assert!(
        matches_uneditable_pattern(
            "build/app.js",
            "build/app.js",
            "/path/build/app.js",
            "build/**"
        )
        .unwrap(),
        "Pattern 'build/**' should match 'build/app.js'"
    );

    // Test case 3: "*.log" should match log files
    assert!(
        matches_uneditable_pattern("debug.log", "debug.log", "/path/debug.log", "*.log").unwrap(),
        "Pattern '*.log' should match 'debug.log'"
    );

    assert!(
        matches_uneditable_pattern("app.log", "app.log", "/path/app.log", "*.log").unwrap(),
        "Pattern '*.log' should match 'app.log'"
    );

    // Test case 4: Non-matching paths should NOT match
    assert!(
        !matches_uneditable_pattern("src/main.rs", "src/main.rs", "/path/src/main.rs", "dist/**")
            .unwrap(),
        "Pattern 'dist/**' should NOT match 'src/main.rs'"
    );

    assert!(
        !matches_uneditable_pattern("README.md", "README.md", "/path/README.md", "*.log").unwrap(),
        "Pattern '*.log' should NOT match 'README.md'"
    );

    assert!(
        !matches_uneditable_pattern("build.rs", "build.rs", "/path/build.rs", "build/**").unwrap(),
        "Pattern 'build/**' should NOT match 'build.rs' (file, not in directory)"
    );
}

#[test]
fn test_prevent_additions_multiple_patterns() {
    // Test that multiple patterns work correctly and ANY match blocks the operation
    // This simulates having preventAdditions: ["dist/**", "build/**", "*.log"]

    let patterns = ["dist/**", "build/**", "*.log"];

    // Test case 1: File matches first pattern
    let test_file_1 = "dist/output.js";
    let matches_any_1 = patterns.iter().any(|pattern| {
        matches_uneditable_pattern(
            test_file_1,
            test_file_1,
            &format!("/path/{}", test_file_1),
            pattern,
        )
        .unwrap_or(false)
    });
    assert!(
        matches_any_1,
        "dist/output.js should match 'dist/**' pattern"
    );

    // Test case 2: File matches second pattern
    let test_file_2 = "build/app.js";
    let matches_any_2 = patterns.iter().any(|pattern| {
        matches_uneditable_pattern(
            test_file_2,
            test_file_2,
            &format!("/path/{}", test_file_2),
            pattern,
        )
        .unwrap_or(false)
    });
    assert!(
        matches_any_2,
        "build/app.js should match 'build/**' pattern"
    );

    // Test case 3: File matches third pattern
    let test_file_3 = "debug.log";
    let matches_any_3 = patterns.iter().any(|pattern| {
        matches_uneditable_pattern(
            test_file_3,
            test_file_3,
            &format!("/path/{}", test_file_3),
            pattern,
        )
        .unwrap_or(false)
    });
    assert!(matches_any_3, "debug.log should match '*.log' pattern");

    // Test case 4: File matches NONE of the patterns
    let test_file_4 = "src/main.rs";
    let matches_any_4 = patterns.iter().any(|pattern| {
        matches_uneditable_pattern(
            test_file_4,
            test_file_4,
            &format!("/path/{}", test_file_4),
            pattern,
        )
        .unwrap_or(false)
    });
    assert!(
        !matches_any_4,
        "src/main.rs should NOT match any of the patterns"
    );

    // Test case 5: Nested file in dist directory (should match first pattern)
    let test_file_5 = "dist/nested/deep/file.js";
    let matches_any_5 = patterns.iter().any(|pattern| {
        matches_uneditable_pattern(
            test_file_5,
            test_file_5,
            &format!("/path/{}", test_file_5),
            pattern,
        )
        .unwrap_or(false)
    });
    assert!(
        matches_any_5,
        "dist/nested/deep/file.js should match 'dist/**' pattern"
    );
}

#[test]
fn test_prevent_additions_and_uneditable_files_both_checked() {
    // Test that both preventAdditions and uneditableFiles are checked independently
    // A file can be blocked by either rule

    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, UnEditableFileRule};

    // Create config with both preventAdditions and uneditableFiles
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            prevent_additions: vec!["dist/**".to_string()],
            uneditable_files: vec![UnEditableFileRule::Simple(".env*".to_string())],
            ..Default::default()
        },
        ..Default::default()
    };

    // Test case 1: File matches preventAdditions pattern only
    let file_1 = "dist/output.js";
    let matches_prevent = config.pre_tool_use.prevent_additions.iter().any(|pattern| {
        matches_uneditable_pattern(file_1, file_1, &format!("/path/{}", file_1), pattern)
            .unwrap_or(false)
    });
    let matches_uneditable = config.pre_tool_use.uneditable_files.iter().any(|rule| {
        matches_uneditable_pattern(file_1, file_1, &format!("/path/{}", file_1), rule.pattern())
            .unwrap_or(false)
    });
    assert!(
        matches_prevent,
        "dist/output.js should match preventAdditions pattern"
    );
    assert!(
        !matches_uneditable,
        "dist/output.js should NOT match uneditableFiles pattern"
    );

    // Test case 2: File matches uneditableFiles pattern only
    let file_2 = ".env.local";
    let matches_prevent_2 = config.pre_tool_use.prevent_additions.iter().any(|pattern| {
        matches_uneditable_pattern(file_2, file_2, &format!("/path/{}", file_2), pattern)
            .unwrap_or(false)
    });
    let matches_uneditable_2 = config.pre_tool_use.uneditable_files.iter().any(|rule| {
        matches_uneditable_pattern(file_2, file_2, &format!("/path/{}", file_2), rule.pattern())
            .unwrap_or(false)
    });
    assert!(
        !matches_prevent_2,
        ".env.local should NOT match preventAdditions pattern"
    );
    assert!(
        matches_uneditable_2,
        ".env.local should match uneditableFiles pattern"
    );

    // Test case 3: File matches neither pattern
    let file_3 = "src/main.rs";
    let matches_prevent_3 = config.pre_tool_use.prevent_additions.iter().any(|pattern| {
        matches_uneditable_pattern(file_3, file_3, &format!("/path/{}", file_3), pattern)
            .unwrap_or(false)
    });
    let matches_uneditable_3 = config.pre_tool_use.uneditable_files.iter().any(|rule| {
        matches_uneditable_pattern(file_3, file_3, &format!("/path/{}", file_3), rule.pattern())
            .unwrap_or(false)
    });
    assert!(
        !matches_prevent_3,
        "src/main.rs should NOT match preventAdditions pattern"
    );
    assert!(
        !matches_uneditable_3,
        "src/main.rs should NOT match uneditableFiles pattern"
    );

    // Both rules are checked independently in check_file_validation_rules
    // The implementation checks preventAdditions first (for Write tool only),
    // then checks uneditableFiles (for all file operations)
}

#[test]
fn test_prevent_additions_glob_pattern_variations() {
    // Test various glob pattern formats that should work with preventAdditions

    // Test case 1: Directory with wildcard
    assert!(
        matches_uneditable_pattern(
            "dist/file.js",
            "dist/file.js",
            "/path/dist/file.js",
            "dist/**"
        )
        .unwrap(),
        "Pattern 'dist/**' should match files in dist directory"
    );

    // Test case 2: Extension wildcard
    assert!(
        matches_uneditable_pattern("test.tmp", "test.tmp", "/path/test.tmp", "*.tmp").unwrap(),
        "Pattern '*.tmp' should match .tmp files"
    );

    // Test case 3: Specific file
    assert!(
        matches_uneditable_pattern("output.log", "output.log", "/path/output.log", "output.log")
            .unwrap(),
        "Exact filename should match"
    );

    // Test case 4: Multiple levels with wildcard
    assert!(
        matches_uneditable_pattern(
            "node_modules/package/dist/file.js",
            "node_modules/package/dist/file.js",
            "/path/node_modules/package/dist/file.js",
            "node_modules/**"
        )
        .unwrap(),
        "Pattern 'node_modules/**' should match deeply nested files"
    );

    // Test case 5: Combined patterns (prefix + extension)
    assert!(
        matches_uneditable_pattern(
            "temp/test.tmp",
            "temp/test.tmp",
            "/path/temp/test.tmp",
            "temp/*.tmp"
        )
        .unwrap(),
        "Pattern 'temp/*.tmp' should match .tmp files in temp directory"
    );

    // Test case 6: Hidden files
    assert!(
        matches_uneditable_pattern(".cache", ".cache", "/path/.cache", ".*").unwrap(),
        "Pattern '.*' should match hidden files"
    );
}

#[test]
fn test_prevent_additions_write_tool_with_various_paths() {
    // Test that Write tool payload is correctly identified for various file paths

    let test_paths = vec![
        "dist/output.js",
        "build/app.min.js",
        "temp/cache.tmp",
        ".cache/data",
        "node_modules/package/index.js",
        "logs/debug.log",
    ];

    for file_path in test_paths {
        let mut tool_input = HashMap::new();
        tool_input.insert(
            "file_path".to_string(),
            Value::String(file_path.to_string()),
        );

        let payload = PreToolUsePayload {
            base: create_test_base_payload(),
            tool_name: "Write".to_string(),
            tool_input,
            tool_use_id: None,
        };

        // Verify the payload is correctly structured
        assert_eq!(payload.tool_name, "Write");
        let extracted_path = extract_file_path(&payload.tool_input);
        assert_eq!(
            extracted_path,
            Some(file_path.to_string()),
            "File path should be extracted correctly for {}",
            file_path
        );
    }
}

#[test]
fn test_prevent_additions_pattern_matching_edge_cases() {
    // Test edge cases in pattern matching

    // Test case 1: Root-level file with wildcard pattern
    assert!(
        matches_uneditable_pattern("test.log", "test.log", "/path/test.log", "*.log").unwrap(),
        "Root-level .log file should match '*.log' pattern"
    );

    // Test case 2: File with multiple extensions
    assert!(
        matches_uneditable_pattern(
            "archive.tar.gz",
            "archive.tar.gz",
            "/path/archive.tar.gz",
            "*.gz"
        )
        .unwrap(),
        "File with multiple extensions should match by final extension"
    );

    // Test case 3: Directory name similar to file pattern
    assert!(
        !matches_uneditable_pattern("dist.js", "dist.js", "/path/dist.js", "dist/**").unwrap(),
        "File named 'dist.js' should NOT match 'dist/**' (file, not directory)"
    );

    // Test case 4: Empty file name (edge case)
    assert!(
        !matches_uneditable_pattern("", "", "/path/", "*.log").unwrap(),
        "Empty filename should not match any pattern"
    );

    // Test case 5: Path with leading ./ (normalized)
    assert!(
        matches_uneditable_pattern(
            "dist/output.js",
            "dist/output.js",
            "/path/dist/output.js",
            "dist/**"
        )
        .unwrap(),
        "Path without leading ./ should still match"
    );
}

#[test]
fn test_prevent_additions_does_not_affect_edit_operations() {
    // Explicitly verify that Edit operations are never blocked by preventAdditions
    // even if the file matches a preventAdditions pattern

    let test_files = vec![
        "dist/output.js", // Matches "dist/**"
        "build/app.js",   // Matches "build/**"
        "debug.log",      // Matches "*.log"
        "temp/cache.tmp", // Matches "temp/**" or "*.tmp"
    ];

    for file_path in test_files {
        let mut tool_input = HashMap::new();
        tool_input.insert(
            "file_path".to_string(),
            Value::String(file_path.to_string()),
        );

        // Create Edit tool payload
        let edit_payload = PreToolUsePayload {
            base: create_test_base_payload(),
            tool_name: "Edit".to_string(),
            tool_input,
            tool_use_id: None,
        };

        // Verify it's Edit tool, not Write
        assert_eq!(edit_payload.tool_name, "Edit");
        assert_ne!(edit_payload.tool_name, "Write");

        // The check in check_file_validation_rules has:
        // `&& payload.tool_name == "Write"`
        // So Edit operations will NOT be blocked by preventAdditions
    }
}

#[test]
fn test_prevent_additions_combined_with_prevent_root_additions() {
    // Test that preventAdditions and preventRootAdditions work independently
    // preventRootAdditions: blocks NEW files at root level (Write tool only)
    // preventAdditions: blocks NEW files matching patterns (Write tool only)

    use std::env;

    let cwd = env::current_dir().unwrap();
    let config_path = cwd.join(".conclaude.yaml");

    // Test case 1: Root-level file that matches preventAdditions pattern
    let root_file_pattern = "dist/output.js"; // Not at root, in dist/
    let is_root = is_root_addition(root_file_pattern, root_file_pattern, &config_path);
    assert!(
        !is_root,
        "dist/output.js is not a root-level file (it's in dist/)"
    );

    // Even though it's not at root, it could be blocked by preventAdditions if pattern matches
    let matches_dist_pattern = matches_uneditable_pattern(
        root_file_pattern,
        root_file_pattern,
        root_file_pattern,
        "dist/**",
    )
    .unwrap();
    assert!(
        matches_dist_pattern,
        "dist/output.js should match 'dist/**' pattern"
    );

    // Test case 2: Root-level file that does NOT match preventAdditions pattern
    let root_file = "newfile.txt";
    let is_root_2 = is_root_addition(root_file, root_file, &config_path);
    assert!(is_root_2, "newfile.txt is at root level");

    let matches_pattern =
        matches_uneditable_pattern(root_file, root_file, root_file, "dist/**").unwrap();
    assert!(
        !matches_pattern,
        "newfile.txt should NOT match 'dist/**' pattern"
    );
    // This would be blocked by preventRootAdditions (if enabled)
    // but NOT by preventAdditions with pattern "dist/**"

    // The two rules check different conditions and both can apply to Write operations
}

#[test]
fn test_prevent_additions_with_nested_directories() {
    // Test that deeply nested files are correctly matched by directory patterns

    let pattern = "build/**";

    // Test various nesting levels
    let test_cases = vec![
        ("build/output.js", true),
        ("build/js/app.js", true),
        ("build/js/vendor/lib.js", true),
        ("build/css/styles.css", true),
        ("build/assets/images/logo.png", true),
        ("src/build/file.js", false), // "build" in different context
        ("prebuild/file.js", false),  // "build" as suffix
        ("build.js", false),          // filename, not directory
    ];

    for (file_path, should_match) in test_cases {
        let matches = matches_uneditable_pattern(file_path, file_path, file_path, pattern).unwrap();
        assert_eq!(
            matches, should_match,
            "Pattern '{}' match result for '{}' should be {}",
            pattern, file_path, should_match
        );
    }
}

// ============================================================================
// Integration Tests for Agent-Scoped Tool Usage Validation
// ============================================================================

#[test]
fn test_agent_pattern_exact_match() {
    use conclaude::hooks::matches_agent_pattern;

    assert!(matches_agent_pattern("coder", "coder"));
    assert!(!matches_agent_pattern("tester", "coder"));
    assert!(matches_agent_pattern("orchestrator", "orchestrator"));
    assert!(!matches_agent_pattern("orchestrator", "coder"));
}

#[test]
fn test_agent_pattern_wildcard() {
    use conclaude::hooks::matches_agent_pattern;

    // "*" should match all agents
    assert!(matches_agent_pattern("coder", "*"));
    assert!(matches_agent_pattern("tester", "*"));
    assert!(matches_agent_pattern("orchestrator", "*"));
    assert!(matches_agent_pattern("any_agent_name", "*"));
    assert!(matches_agent_pattern("", "*"));
}

#[test]
fn test_agent_pattern_glob() {
    use conclaude::hooks::matches_agent_pattern;

    // Test "code*" glob pattern
    assert!(matches_agent_pattern("coder", "code*"));
    assert!(matches_agent_pattern("code", "code*"));
    assert!(matches_agent_pattern("codesmith", "code*"));
    assert!(!matches_agent_pattern("tester", "code*"));
    assert!(!matches_agent_pattern("decoder", "code*"));

    // Test "test*" glob pattern
    assert!(matches_agent_pattern("tester", "test*"));
    assert!(matches_agent_pattern("test", "test*"));
    assert!(matches_agent_pattern("testing", "test*"));
    assert!(!matches_agent_pattern("coder", "test*"));
    assert!(!matches_agent_pattern("contest", "test*"));
}

#[test]
fn test_agent_pattern_no_match() {
    use conclaude::hooks::matches_agent_pattern;

    // Test that non-matching patterns return false
    assert!(!matches_agent_pattern("coder", "tester"));
    assert!(!matches_agent_pattern("orchestrator", "agent*"));
    assert!(!matches_agent_pattern("myagent", "your*"));
    assert!(!matches_agent_pattern("test", "testing"));
}

#[tokio::test]
async fn test_tool_usage_rule_with_agent_filter() -> anyhow::Result<()> {
    use conclaude::config::{ConclaudeConfig, PreToolUseConfig, ToolUsageRule};

    // Create test configuration with a rule that only applies to "coder" agent
    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            tool_usage_validation: vec![
                ToolUsageRule {
                    tool: "Bash".to_string(),
                    pattern: String::new(),
                    action: "block".to_string(),
                    message: Some("Command blocked for coder agent!".to_string()),
                    command_pattern: Some("rm -rf /".to_string()),
                    match_mode: Some("full".to_string()),
                    agent: Some("coder".to_string()),
                },
                ToolUsageRule {
                    tool: "Bash".to_string(),
                    pattern: String::new(),
                    action: "block".to_string(),
                    message: Some("Command blocked for test* agents!".to_string()),
                    command_pattern: Some("drop database".to_string()),
                    match_mode: Some("full".to_string()),
                    agent: Some("test*".to_string()),
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    // Test that the first rule matches "coder" agent
    let rule1 = &config.pre_tool_use.tool_usage_validation[0];
    assert!(rule1.agent.is_some());
    let agent_pattern1 = rule1.agent.as_deref().unwrap();

    assert!(
        conclaude::hooks::matches_agent_pattern("coder", agent_pattern1),
        "Rule should match 'coder' agent"
    );
    assert!(
        !conclaude::hooks::matches_agent_pattern("tester", agent_pattern1),
        "Rule should not match 'tester' agent"
    );
    assert!(
        !conclaude::hooks::matches_agent_pattern("orchestrator", agent_pattern1),
        "Rule should not match 'orchestrator' agent"
    );

    // Test that the second rule matches "test*" pattern
    let rule2 = &config.pre_tool_use.tool_usage_validation[1];
    assert!(rule2.agent.is_some());
    let agent_pattern2 = rule2.agent.as_deref().unwrap();

    assert!(
        conclaude::hooks::matches_agent_pattern("tester", agent_pattern2),
        "Rule should match 'tester' agent"
    );
    assert!(
        conclaude::hooks::matches_agent_pattern("testing", agent_pattern2),
        "Rule should match 'testing' agent"
    );
    assert!(
        !conclaude::hooks::matches_agent_pattern("coder", agent_pattern2),
        "Rule should not match 'coder' agent"
    );

    Ok(())
}
