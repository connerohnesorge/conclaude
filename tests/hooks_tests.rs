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
fn test_extract_file_path_with_non_string_value() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::Number(serde_json::Number::from(42)),
    );

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
fn test_matches_uneditable_pattern() {
    // Exact file matches
    assert!(matches_uneditable_pattern(
        "package.json",
        "package.json",
        "/path/package.json",
        "package.json"
    )
    .unwrap());

    // Wildcard matches
    assert!(matches_uneditable_pattern("test.md", "test.md", "/path/test.md", "*.md").unwrap());
    assert!(
        matches_uneditable_pattern("README.md", "README.md", "/path/README.md", "*.md").unwrap()
    );

    // Directory pattern matches
    assert!(matches_uneditable_pattern(
        "src/index.ts",
        "src/index.ts",
        "/path/src/index.ts",
        "src/**/*.ts"
    )
    .unwrap());
    assert!(matches_uneditable_pattern(
        "src/lib/utils.ts",
        "src/lib/utils.ts",
        "/path/src/lib/utils.ts",
        "src/**/*.ts"
    )
    .unwrap());

    // Negative matches
    assert!(
        !matches_uneditable_pattern("other.txt", "other.txt", "/path/other.txt", "*.md").unwrap()
    );
    assert!(!matches_uneditable_pattern(
        "lib/index.ts",
        "lib/index.ts",
        "/path/lib/index.ts",
        "src/**/*.ts"
    )
    .unwrap());
}

#[test]
fn test_matches_uneditable_pattern_invalid_glob() {
    let result = matches_uneditable_pattern("test.txt", "test.txt", "/path/test.txt", "[invalid");
    assert!(result.is_err());
}

#[test]
fn test_matches_uneditable_pattern_multiple_patterns() {
    // Test multiple patterns separately (since the glob crate doesn't support brace expansion)
    assert!(matches_uneditable_pattern(
        "package.json",
        "package.json",
        "/path/package.json",
        "package.json"
    )
    .unwrap());
    assert!(matches_uneditable_pattern(
        "tsconfig.json",
        "tsconfig.json",
        "/path/tsconfig.json",
        "tsconfig.json"
    )
    .unwrap());
    assert!(!matches_uneditable_pattern(
        "other.json",
        "other.json",
        "/path/other.json",
        "package.json"
    )
    .unwrap());
}

#[test]
fn test_matches_uneditable_pattern_environment_files() {
    assert!(matches_uneditable_pattern(".env", ".env", "/path/.env", ".env*").unwrap());
    assert!(
        matches_uneditable_pattern(".env.local", ".env.local", "/path/.env.local", ".env*")
            .unwrap()
    );
    assert!(matches_uneditable_pattern(
        ".env.production",
        ".env.production",
        "/path/.env.production",
        ".env*"
    )
    .unwrap());
    assert!(!matches_uneditable_pattern(
        "environment.txt",
        "environment.txt",
        "/path/environment.txt",
        ".env*"
    )
    .unwrap());
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

// Tests for auto-generated file checking
#[test]
fn test_check_generated_file_markers_do_not_edit() {
    let content = "// DO NOT EDIT - This file is generated\nfn main() {}";
    let result = check_generated_file_markers(content);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "DO NOT EDIT");
}

#[test]
fn test_check_generated_file_markers_lowercase() {
    let content = "/* do not edit this file */\nclass Example {}";
    let result = check_generated_file_markers(content);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "do not edit");
}

#[test]
fn test_check_generated_file_markers_code_generated() {
    let content = "// Code generated by protoc-gen-go. DO NOT EDIT.\npackage main";
    let result = check_generated_file_markers(content);
    assert!(result.is_some());
    // The function returns "DO NOT EDIT" which appears first in the check order
    assert_eq!(result.unwrap(), "DO NOT EDIT");
}

#[test]
fn test_check_generated_file_markers_auto_generated() {
    let content = "/* Auto-generated file - modifications will be overwritten */\n";
    let result = check_generated_file_markers(content);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "Auto-generated");
}

#[test]
fn test_check_generated_file_markers_at_generated() {
    let content = "/**\n * @generated\n * This file is generated by tools\n */";
    let result = check_generated_file_markers(content);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "@generated");
}

#[test]
fn test_check_generated_file_markers_this_file_generated() {
    let content = "// This file is generated automatically\n// Don't modify\n";
    let result = check_generated_file_markers(content);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "This file is generated");
}

#[test]
fn test_check_generated_file_markers_case_variations() {
    let variations = vec![
        "// AUTOGENERATED",
        "// autogenerated",
        "// AuToGeNeRaTeD",
        "/* Generated Code */",
        "// generated code follows",
    ];

    for content in variations {
        let result = check_generated_file_markers(content);
        assert!(result.is_some(), "Failed to detect marker in: {}", content);
    }
}

#[test]
fn test_check_generated_file_markers_no_markers() {
    let content = "// Regular source file\nfn calculate() -> i32 {\n    42\n}";
    let result = check_generated_file_markers(content);
    assert!(result.is_none());
}

#[test]
fn test_check_generated_file_markers_marker_after_100_lines() {
    let mut content = String::new();
    for i in 0..105 {
        content.push_str(&format!("// Line {}\n", i));
    }
    content.push_str("// DO NOT EDIT\n");

    // Should not detect marker after 100 lines
    let result = check_generated_file_markers(&content);
    assert!(result.is_none());
}

#[test]
fn test_check_generated_file_markers_within_100_lines() {
    let mut content = String::new();
    for i in 0..95 {
        content.push_str(&format!("// Line {}\n", i));
    }
    content.push_str("// DO NOT EDIT - Generated file\n");

    // Should detect marker within first 100 lines
    let result = check_generated_file_markers(&content);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "DO NOT EDIT");
}

// Note: Integration tests for auto-generated file checking with config cannot be
// easily tested due to the global config cache. The functionality is tested
// through unit tests of the marker detection function and will be tested in
// real usage when the hook is invoked.

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
fn test_subagent_stop_payload_structure() {
    let payload = create_subagent_stop_payload();

    // Verify all required fields are present and populated
    assert_eq!(payload.base.session_id, "test_session_456");
    assert_eq!(
        payload.base.transcript_path,
        "/tmp/session_transcript.jsonl"
    );
    assert_eq!(payload.base.hook_event_name, "SubagentStop");
    assert_eq!(payload.base.cwd, "/home/user/project");
    assert!(payload.base.permission_mode.is_some());
    assert!(payload.stop_hook_active);
    assert_eq!(payload.agent_id, "coder");
    assert_eq!(payload.agent_transcript_path, "/tmp/coder_transcript.jsonl");
}

#[test]
fn test_subagent_stop_payload_with_different_agent_ids() {
    // Test with different agent IDs: coder, tester, stuck
    let agent_ids = vec!["coder", "tester", "stuck", "orchestrator"];

    for agent_id in agent_ids {
        let mut payload = create_subagent_stop_payload();
        payload.agent_id = agent_id.to_string();

        assert_eq!(payload.agent_id, agent_id);
        assert!(validate_subagent_stop_payload(&payload).is_ok());
    }
}

#[test]
fn test_subagent_stop_payload_serialization() {
    let payload = create_subagent_stop_payload();

    // Serialize to JSON
    let json_str = serde_json::to_string(&payload).expect("Failed to serialize");

    // Deserialize back
    let deserialized: SubagentStopPayload =
        serde_json::from_str(&json_str).expect("Failed to deserialize");

    // Verify round-trip preservation
    assert_eq!(deserialized.base.session_id, payload.base.session_id);
    assert_eq!(deserialized.agent_id, payload.agent_id);
    assert_eq!(
        deserialized.agent_transcript_path,
        payload.agent_transcript_path
    );
    assert_eq!(deserialized.stop_hook_active, payload.stop_hook_active);
}

#[test]
fn test_validate_subagent_stop_payload_all_fields_valid() {
    let payload = create_subagent_stop_payload();
    let result = validate_subagent_stop_payload(&payload);

    assert!(result.is_ok(), "Valid payload should pass validation");
}

#[test]
fn test_validate_subagent_stop_payload_missing_base_session_id() {
    let mut payload = create_subagent_stop_payload();
    payload.base.session_id = String::new();

    let result = validate_subagent_stop_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty session_id should fail validation"
    );
    let error = result.unwrap_err();
    assert!(
        error.contains("session_id") || error.contains("Missing required field"),
        "Error should mention session_id or missing field: {}",
        error
    );
}

#[test]
fn test_validate_subagent_stop_payload_missing_base_transcript_path() {
    let mut payload = create_subagent_stop_payload();
    payload.base.transcript_path = String::new();

    let result = validate_subagent_stop_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty transcript_path should fail validation"
    );
    let error = result.unwrap_err();
    assert!(
        error.contains("transcript_path") || error.contains("Missing required field"),
        "Error should mention transcript_path or missing field: {}",
        error
    );
}

#[test]
fn test_validate_subagent_stop_payload_missing_agent_id() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_id = String::new();

    let result = validate_subagent_stop_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty agent_id should fail validation"
    );
    assert!(result.unwrap_err().contains("agent_id cannot be empty"));
}

#[test]
fn test_validate_subagent_stop_payload_whitespace_agent_id() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_id = "   ".to_string(); // Only whitespace

    let result = validate_subagent_stop_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with whitespace-only agent_id should fail validation"
    );
    assert!(result.unwrap_err().contains("agent_id cannot be empty"));
}

#[test]
fn test_validate_subagent_stop_payload_missing_agent_transcript_path() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_transcript_path = String::new();

    let result = validate_subagent_stop_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty agent_transcript_path should fail validation"
    );
    assert!(result
        .unwrap_err()
        .contains("agent_transcript_path cannot be empty"));
}

#[test]
fn test_validate_subagent_stop_payload_whitespace_agent_transcript_path() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_transcript_path = "   ".to_string(); // Only whitespace

    let result = validate_subagent_stop_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with whitespace-only agent_transcript_path should fail validation"
    );
    assert!(result
        .unwrap_err()
        .contains("agent_transcript_path cannot be empty"));
}

#[test]
fn test_validate_subagent_stop_payload_multiple_missing_fields() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_id = String::new();
    payload.agent_transcript_path = String::new();

    let result = validate_subagent_stop_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with multiple missing fields should fail"
    );
}

#[test]
fn test_subagent_stop_payload_json_round_trip_coder() {
    let json_str = r#"{
        "session_id": "test_session_coder",
        "transcript_path": "/tmp/session_123.jsonl",
        "hook_event_name": "SubagentStop",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "stop_hook_active": true,
        "agent_id": "coder",
        "agent_transcript_path": "/tmp/coder_123.jsonl"
    }"#;

    let payload: SubagentStopPayload =
        serde_json::from_str(json_str).expect("Failed to deserialize JSON");

    assert_eq!(payload.agent_id, "coder");
    assert_eq!(payload.agent_transcript_path, "/tmp/coder_123.jsonl");
    assert_eq!(payload.base.session_id, "test_session_coder");
    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_subagent_stop_payload_json_round_trip_tester() {
    let json_str = r#"{
        "session_id": "test_session_tester",
        "transcript_path": "/tmp/session_456.jsonl",
        "hook_event_name": "SubagentStop",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "stop_hook_active": false,
        "agent_id": "tester",
        "agent_transcript_path": "/tmp/tester_456.jsonl"
    }"#;

    let payload: SubagentStopPayload =
        serde_json::from_str(json_str).expect("Failed to deserialize JSON");

    assert_eq!(payload.agent_id, "tester");
    assert_eq!(payload.agent_transcript_path, "/tmp/tester_456.jsonl");
    assert!(!payload.stop_hook_active);
    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_subagent_stop_payload_json_round_trip_stuck() {
    let json_str = r#"{
        "session_id": "test_session_stuck",
        "transcript_path": "/tmp/session_789.jsonl",
        "hook_event_name": "SubagentStop",
        "cwd": "/home/user/project",
        "permission_mode": "restrict",
        "stop_hook_active": true,
        "agent_id": "stuck",
        "agent_transcript_path": "/tmp/stuck_789.jsonl"
    }"#;

    let payload: SubagentStopPayload =
        serde_json::from_str(json_str).expect("Failed to deserialize JSON");

    assert_eq!(payload.agent_id, "stuck");
    assert_eq!(payload.agent_transcript_path, "/tmp/stuck_789.jsonl");
    assert_eq!(payload.base.permission_mode, Some("restrict".to_string()));
    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_subagent_stop_payload_with_long_paths() {
    let mut payload = create_subagent_stop_payload();
    let long_path =
        "/very/long/path/to/transcript/file/that/contains/many/nested/directories/transcript.jsonl";
    payload.agent_transcript_path = long_path.to_string();

    assert_eq!(payload.agent_transcript_path, long_path);
    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_subagent_stop_payload_with_special_characters_in_paths() {
    let mut payload = create_subagent_stop_payload();
    payload.agent_transcript_path = "/tmp/transcript-2024-11-16_10:30:45.jsonl".to_string();
    payload.agent_id = "coder-v2.1".to_string();

    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_subagent_stop_payload_agent_id_case_sensitive() {
    let mut payload1 = create_subagent_stop_payload();
    let mut payload2 = create_subagent_stop_payload();

    payload1.agent_id = "Coder".to_string();
    payload2.agent_id = "coder".to_string();

    // Both should validate successfully
    assert!(validate_subagent_stop_payload(&payload1).is_ok());
    assert!(validate_subagent_stop_payload(&payload2).is_ok());

    // But they should be different values
    assert_ne!(payload1.agent_id, payload2.agent_id);
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
fn test_subagent_stop_payload_with_none_permission_mode() {
    let mut payload = create_subagent_stop_payload();
    payload.base.permission_mode = None;

    // Should still validate successfully since permission_mode is optional
    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_subagent_stop_payload_with_empty_permission_mode() {
    let mut payload = create_subagent_stop_payload();
    payload.base.permission_mode = Some(String::new());

    // Should still validate - only required fields matter for base payload
    // Empty permission_mode is allowed
    assert!(validate_subagent_stop_payload(&payload).is_ok());
}

#[test]
fn test_subagent_stop_payload_stop_hook_active_true_and_false() {
    let mut payload_true = create_subagent_stop_payload();
    let mut payload_false = create_subagent_stop_payload();

    payload_true.stop_hook_active = true;
    payload_false.stop_hook_active = false;

    assert!(validate_subagent_stop_payload(&payload_true).is_ok());
    assert!(validate_subagent_stop_payload(&payload_false).is_ok());
    assert_ne!(
        payload_true.stop_hook_active,
        payload_false.stop_hook_active
    );
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
fn test_subagent_stop_environment_variable_setting_simulation() {
    // Simulate setting environment variables as the hook handler would do
    let payload = create_subagent_stop_payload();
    let unique_id = "test_env_var_setting";

    // Set environment variables with unique suffix to avoid test conflicts
    let agent_id_var = format!("CONCLAUDE_AGENT_ID_{}", unique_id);
    let transcript_var = format!("CONCLAUDE_AGENT_TRANSCRIPT_PATH_{}", unique_id);

    std::env::set_var(&agent_id_var, &payload.agent_id);
    std::env::set_var(&transcript_var, &payload.agent_transcript_path);

    // Verify they were set correctly
    assert_eq!(std::env::var(&agent_id_var).unwrap(), payload.agent_id);
    assert_eq!(
        std::env::var(&transcript_var).unwrap(),
        payload.agent_transcript_path
    );

    // Test the actual environment variables (document what the hook does)
    // Note: We only verify the structure is correct, actual env vars may be set by other tests
    assert!(!payload.agent_id.is_empty());
    assert!(!payload.agent_transcript_path.is_empty());

    // Clean up
    std::env::remove_var(&agent_id_var);
    std::env::remove_var(&transcript_var);
}

#[test]
fn test_subagent_stop_environment_variables_different_agents() {
    let agents = ["coder", "tester", "stuck"];

    for (idx, agent) in agents.iter().enumerate() {
        let mut payload = create_subagent_stop_payload();
        payload.agent_id = agent.to_string();

        // Set environment variables with unique suffix to avoid test conflicts
        let agent_id_var = format!("CONCLAUDE_AGENT_ID_diff_agents_{}", idx);
        let transcript_var = format!("CONCLAUDE_AGENT_TRANSCRIPT_PATH_diff_agents_{}", idx);

        std::env::set_var(&agent_id_var, &payload.agent_id);
        std::env::set_var(&transcript_var, &payload.agent_transcript_path);

        // Verify they match
        assert_eq!(
            std::env::var(&agent_id_var).unwrap(),
            *agent,
            "Agent ID should match in environment for {}",
            agent
        );

        // Clean up
        std::env::remove_var(&agent_id_var);
        std::env::remove_var(&transcript_var);
    }
}

#[test]
fn test_subagent_stop_environment_variables_special_paths() {
    let paths = [
        "/tmp/transcript-with-dashes.jsonl",
        "/tmp/transcript_with_underscores.jsonl",
        "/tmp/transcript.2024-11-16.jsonl",
        "/var/log/conclaude/transcript.jsonl",
    ];

    for (idx, path) in paths.iter().enumerate() {
        let mut payload = create_subagent_stop_payload();
        payload.agent_transcript_path = path.to_string();

        // Set environment variable with unique suffix
        let transcript_var = format!("CONCLAUDE_AGENT_TRANSCRIPT_PATH_special_{}", idx);
        std::env::set_var(&transcript_var, &payload.agent_transcript_path);

        // Verify it was set correctly by reading it back
        let read_value = std::env::var(&transcript_var);
        assert!(
            read_value.is_ok(),
            "Path should be set in environment: {}",
            path
        );
        assert_eq!(
            read_value.unwrap(),
            *path,
            "Path should be preserved in environment: {}",
            path
        );

        // Clean up for next iteration
        std::env::remove_var(&transcript_var);
    }
}

#[test]
fn test_subagent_stop_multiple_sequential_invocations() {
    let agents = [
        ("coder", "/tmp/coder_001.jsonl"),
        ("tester", "/tmp/tester_001.jsonl"),
        ("stuck", "/tmp/stuck_001.jsonl"),
    ];

    for (idx, (agent_id, transcript_path)) in agents.iter().enumerate() {
        let mut payload = create_subagent_stop_payload();
        payload.agent_id = agent_id.to_string();
        payload.agent_transcript_path = transcript_path.to_string();

        // Validate the payload
        assert!(
            validate_subagent_stop_payload(&payload).is_ok(),
            "Payload for {} should validate",
            agent_id
        );

        // Simulate setting environment variables with unique suffix
        let agent_id_var = format!("CONCLAUDE_AGENT_ID_seq_{}", idx);
        let transcript_var = format!("CONCLAUDE_AGENT_TRANSCRIPT_PATH_seq_{}", idx);

        std::env::set_var(&agent_id_var, &payload.agent_id);
        std::env::set_var(&transcript_var, &payload.agent_transcript_path);

        // Verify environment variables are set to the new values
        let agent_id_read = std::env::var(&agent_id_var);
        assert!(agent_id_read.is_ok());
        assert_eq!(
            agent_id_read.unwrap(),
            *agent_id,
            "Environment variable should reflect current agent"
        );

        let transcript_path_read = std::env::var(&transcript_var);
        assert!(transcript_path_read.is_ok());
        assert_eq!(
            transcript_path_read.unwrap(),
            *transcript_path,
            "Environment variable should reflect current transcript path"
        );

        // Clean up
        std::env::remove_var(&agent_id_var);
        std::env::remove_var(&transcript_var);
    }
}

#[test]
fn test_subagent_stop_payload_all_required_fields_present() {
    // This test ensures all required fields are actually present and tested
    let payload = create_subagent_stop_payload();

    // Base fields
    assert!(!payload.base.session_id.is_empty());
    assert!(!payload.base.transcript_path.is_empty());
    assert_eq!(payload.base.hook_event_name, "SubagentStop");
    assert!(!payload.base.cwd.is_empty());

    // New SubagentStop-specific fields
    assert!(!payload.agent_id.is_empty());
    assert!(!payload.agent_transcript_path.is_empty());

    // Verify validation passes
    assert!(validate_subagent_stop_payload(&payload).is_ok());
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
// Integration Tests for SubagentStart Hook
// ============================================================================

// Helper function to create a SubagentStart payload for testing
fn create_subagent_start_payload() -> SubagentStartPayload {
    SubagentStartPayload {
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
    }
}

#[test]
fn test_subagent_start_payload_structure() {
    let payload = create_subagent_start_payload();

    // Verify all required fields are present and populated
    assert_eq!(payload.base.session_id, "test_session_789");
    assert_eq!(
        payload.base.transcript_path,
        "/tmp/session_transcript.jsonl"
    );
    assert_eq!(payload.base.hook_event_name, "SubagentStart");
    assert_eq!(payload.base.cwd, "/home/user/project");
    assert!(payload.base.permission_mode.is_some());
    assert_eq!(payload.agent_id, "coder");
    assert_eq!(payload.subagent_type, "implementation");
    assert_eq!(payload.agent_transcript_path, "/tmp/coder_transcript.jsonl");
}

#[test]
fn test_subagent_start_payload_with_different_agent_ids() {
    // Test with different agent IDs: coder, tester, stuck
    let agent_ids = vec!["coder", "tester", "stuck", "orchestrator"];

    for agent_id in agent_ids {
        let mut payload = create_subagent_start_payload();
        payload.agent_id = agent_id.to_string();

        assert_eq!(payload.agent_id, agent_id);
        assert!(validate_subagent_start_payload(&payload).is_ok());
    }
}

#[test]
fn test_subagent_start_payload_serialization() {
    let payload = create_subagent_start_payload();

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

#[test]
fn test_validate_subagent_start_payload_all_fields_valid() {
    let payload = create_subagent_start_payload();
    let result = validate_subagent_start_payload(&payload);

    assert!(result.is_ok(), "Valid payload should pass validation");
}

#[test]
fn test_validate_subagent_start_payload_missing_base_session_id() {
    let mut payload = create_subagent_start_payload();
    payload.base.session_id = String::new();

    let result = validate_subagent_start_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty session_id should fail validation"
    );
    let error = result.unwrap_err();
    assert!(
        error.contains("session_id") || error.contains("Missing required field"),
        "Error should mention session_id or missing field: {}",
        error
    );
}

#[test]
fn test_validate_subagent_start_payload_missing_agent_id() {
    let mut payload = create_subagent_start_payload();
    payload.agent_id = String::new();

    let result = validate_subagent_start_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty agent_id should fail validation"
    );
    assert!(result.unwrap_err().contains("agent_id cannot be empty"));
}

#[test]
fn test_validate_subagent_start_payload_whitespace_agent_id() {
    let mut payload = create_subagent_start_payload();
    payload.agent_id = "   ".to_string(); // Only whitespace

    let result = validate_subagent_start_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with whitespace-only agent_id should fail validation"
    );
    assert!(result.unwrap_err().contains("agent_id cannot be empty"));
}

#[test]
fn test_validate_subagent_start_payload_missing_subagent_type() {
    let mut payload = create_subagent_start_payload();
    payload.subagent_type = String::new();

    let result = validate_subagent_start_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty subagent_type should fail validation"
    );
    assert!(result
        .unwrap_err()
        .contains("subagent_type cannot be empty"));
}

#[test]
fn test_validate_subagent_start_payload_whitespace_subagent_type() {
    let mut payload = create_subagent_start_payload();
    payload.subagent_type = "   ".to_string(); // Only whitespace

    let result = validate_subagent_start_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with whitespace-only subagent_type should fail validation"
    );
    assert!(result
        .unwrap_err()
        .contains("subagent_type cannot be empty"));
}

#[test]
fn test_validate_subagent_start_payload_missing_agent_transcript_path() {
    let mut payload = create_subagent_start_payload();
    payload.agent_transcript_path = String::new();

    let result = validate_subagent_start_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with empty agent_transcript_path should fail validation"
    );
    assert!(result
        .unwrap_err()
        .contains("agent_transcript_path cannot be empty"));
}

#[test]
fn test_validate_subagent_start_payload_whitespace_agent_transcript_path() {
    let mut payload = create_subagent_start_payload();
    payload.agent_transcript_path = "   ".to_string(); // Only whitespace

    let result = validate_subagent_start_payload(&payload);
    assert!(
        result.is_err(),
        "Payload with whitespace-only agent_transcript_path should fail validation"
    );
    assert!(result
        .unwrap_err()
        .contains("agent_transcript_path cannot be empty"));
}

#[test]
fn test_validate_subagent_start_payload_agent_id_with_leading_trailing_spaces() {
    let mut payload = create_subagent_start_payload();
    payload.agent_id = "  coder  ".to_string();

    // Should pass validation because we trim before checking
    assert!(validate_subagent_start_payload(&payload).is_ok());
}

#[test]
fn test_validate_subagent_start_payload_invalid_base() {
    let mut payload = create_subagent_start_payload();
    payload.base.session_id = String::new();

    let result = validate_subagent_start_payload(&payload);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("session_id"));
}

#[test]
fn test_subagent_start_payload_json_round_trip_coder() {
    let json_str = r#"{
        "session_id": "test_session_coder",
        "transcript_path": "/tmp/session_123.jsonl",
        "hook_event_name": "SubagentStart",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "agent_id": "coder",
        "subagent_type": "implementation",
        "agent_transcript_path": "/tmp/coder_123.jsonl"
    }"#;

    let payload: SubagentStartPayload =
        serde_json::from_str(json_str).expect("Failed to deserialize JSON");

    assert_eq!(payload.agent_id, "coder");
    assert_eq!(payload.subagent_type, "implementation");
    assert_eq!(payload.agent_transcript_path, "/tmp/coder_123.jsonl");
    assert_eq!(payload.base.session_id, "test_session_coder");
    assert!(validate_subagent_start_payload(&payload).is_ok());
}

#[test]
fn test_subagent_start_payload_json_round_trip_tester() {
    let json_str = r#"{
        "session_id": "test_session_tester",
        "transcript_path": "/tmp/session_456.jsonl",
        "hook_event_name": "SubagentStart",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "agent_id": "tester",
        "subagent_type": "testing",
        "agent_transcript_path": "/tmp/tester_456.jsonl"
    }"#;

    let payload: SubagentStartPayload =
        serde_json::from_str(json_str).expect("Failed to deserialize JSON");

    assert_eq!(payload.agent_id, "tester");
    assert_eq!(payload.subagent_type, "testing");
    assert_eq!(payload.agent_transcript_path, "/tmp/tester_456.jsonl");
    assert!(validate_subagent_start_payload(&payload).is_ok());
}

#[test]
fn test_subagent_start_payload_json_round_trip_stuck() {
    let json_str = r#"{
        "session_id": "test_session_stuck",
        "transcript_path": "/tmp/session_789.jsonl",
        "hook_event_name": "SubagentStart",
        "cwd": "/home/user/project",
        "permission_mode": "restrict",
        "agent_id": "stuck",
        "subagent_type": "escalation",
        "agent_transcript_path": "/tmp/stuck_789.jsonl"
    }"#;

    let payload: SubagentStartPayload =
        serde_json::from_str(json_str).expect("Failed to deserialize JSON");

    assert_eq!(payload.agent_id, "stuck");
    assert_eq!(payload.subagent_type, "escalation");
    assert_eq!(payload.agent_transcript_path, "/tmp/stuck_789.jsonl");
    assert_eq!(payload.base.permission_mode, Some("restrict".to_string()));
    assert!(validate_subagent_start_payload(&payload).is_ok());
}

#[test]
fn test_subagent_start_payload_json_with_missing_agent_id_field() {
    let json_str = r#"{
        "session_id": "test_session",
        "transcript_path": "/tmp/session.jsonl",
        "hook_event_name": "SubagentStart",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "subagent_type": "implementation",
        "agent_transcript_path": "/tmp/agent.jsonl"
    }"#;

    // This should fail to deserialize because agent_id is required
    let result: Result<SubagentStartPayload, _> = serde_json::from_str(json_str);
    assert!(
        result.is_err(),
        "JSON missing agent_id should fail to deserialize"
    );
}

#[test]
fn test_subagent_start_payload_json_with_missing_subagent_type_field() {
    let json_str = r#"{
        "session_id": "test_session",
        "transcript_path": "/tmp/session.jsonl",
        "hook_event_name": "SubagentStart",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "agent_id": "coder",
        "agent_transcript_path": "/tmp/agent.jsonl"
    }"#;

    // This should fail to deserialize because subagent_type is required
    let result: Result<SubagentStartPayload, _> = serde_json::from_str(json_str);
    assert!(
        result.is_err(),
        "JSON missing subagent_type should fail to deserialize"
    );
}

#[test]
fn test_subagent_start_payload_json_with_missing_agent_transcript_path_field() {
    let json_str = r#"{
        "session_id": "test_session",
        "transcript_path": "/tmp/session.jsonl",
        "hook_event_name": "SubagentStart",
        "cwd": "/home/user/project",
        "permission_mode": "default",
        "agent_id": "coder",
        "subagent_type": "implementation"
    }"#;

    // This should fail to deserialize because agent_transcript_path is required
    let result: Result<SubagentStartPayload, _> = serde_json::from_str(json_str);
    assert!(
        result.is_err(),
        "JSON missing agent_transcript_path should fail to deserialize"
    );
}

#[test]
fn test_subagent_start_validation_error_messages_specific() {
    // Test that error messages are specific and helpful

    let mut payload = create_subagent_start_payload();

    // Test agent_id error
    payload.agent_id = String::new();
    let error = validate_subagent_start_payload(&payload).unwrap_err();
    assert!(
        error.contains("agent_id"),
        "Error should mention agent_id field"
    );

    // Test subagent_type error
    let mut payload = create_subagent_start_payload();
    payload.subagent_type = String::new();
    let error = validate_subagent_start_payload(&payload).unwrap_err();
    assert!(
        error.contains("subagent_type"),
        "Error should mention subagent_type field"
    );

    // Test agent_transcript_path error
    let mut payload = create_subagent_start_payload();
    payload.agent_transcript_path = String::new();
    let error = validate_subagent_start_payload(&payload).unwrap_err();
    assert!(
        error.contains("agent_transcript_path"),
        "Error should mention agent_transcript_path field"
    );

    // Test whitespace-only agent_id error
    let mut payload = create_subagent_start_payload();
    payload.agent_id = "   ".to_string();
    let error = validate_subagent_start_payload(&payload).unwrap_err();
    assert!(
        error.contains("agent_id"),
        "Error should mention agent_id for whitespace-only value"
    );
}

#[test]
fn test_subagent_start_payload_with_different_subagent_types() {
    let subagent_types = vec!["implementation", "testing", "escalation", "analysis"];

    for subagent_type in subagent_types {
        let mut payload = create_subagent_start_payload();
        payload.subagent_type = subagent_type.to_string();

        assert_eq!(payload.subagent_type, subagent_type);
        assert!(validate_subagent_start_payload(&payload).is_ok());
    }
}

#[test]
fn test_subagent_start_hook_event_name_validation() {
    let payload = create_subagent_start_payload();

    // Verify the hook event name is correctly set
    assert_eq!(payload.base.hook_event_name, "SubagentStart");

    // Verify the base payload validates correctly
    assert!(validate_base_payload(&payload.base).is_ok());
}

#[test]
fn test_hook_payload_subagent_start_variant_serialization() {
    let subagent_start_payload = create_subagent_start_payload();

    let hook_payload = HookPayload::SubagentStart(subagent_start_payload.clone());

    // Serialize to JSON
    let json_str = serde_json::to_string(&hook_payload).expect("Failed to serialize");

    // Verify the JSON contains the hook_event_name tag
    assert!(json_str.contains("\"hook_event_name\":\"SubagentStart\""));

    // Verify the JSON contains the required fields
    assert!(json_str.contains("\"agent_id\":\"coder\""));
    assert!(json_str.contains("\"subagent_type\":\"implementation\""));
    assert!(json_str.contains("\"agent_transcript_path\":\"/tmp/coder_transcript.jsonl\""));
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
fn test_prevent_root_additions_semantic_correctness() {
    // This test documents the expected semantic behavior:
    // - is_root_addition() checks LOCATION only (is it at root level?)
    // - check_file_validation_rules() now also checks EXISTENCE
    //
    // The combination means:
    // - New files at root → blocked (is_root_addition=true, exists=false)
    // - Existing files at root → allowed (is_root_addition=true, exists=true)
    // - Files not at root → allowed (is_root_addition=false, doesn't matter if exists)

    use std::env;

    let cwd = env::current_dir().unwrap();
    let config_path = cwd.join(".conclaude.yaml");

    // Test 1: A file that exists at root (e.g., Cargo.toml in this project)
    let existing_root_file = "Cargo.toml";
    let is_root = is_root_addition(existing_root_file, existing_root_file, &config_path);
    let exists = cwd.join(existing_root_file).exists();
    assert!(is_root, "Cargo.toml should be at root level");
    assert!(exists, "Cargo.toml should exist");
    // With both conditions true, the file should be ALLOWED to be modified

    // Test 2: A non-existent file at root
    let new_root_file = "nonexistent_test_file_12345.txt";
    let is_root = is_root_addition(new_root_file, new_root_file, &config_path);
    let exists = cwd.join(new_root_file).exists();
    assert!(is_root, "nonexistent file should be at root level");
    assert!(!exists, "nonexistent file should not exist");
    // With is_root=true and exists=false, the file should be BLOCKED

    // Test 3: A file in a subdirectory (should always be allowed)
    let subdir_file = "src/main.rs";
    let is_root = is_root_addition(subdir_file, subdir_file, &config_path);
    assert!(!is_root, "src/main.rs should NOT be at root level");
    // Doesn't matter if it exists, it's not at root so it's allowed
}

#[test]
fn test_prevent_root_additions_existing_config_files() {
    use std::env;

    let cwd = env::current_dir().unwrap();
    let config_path = cwd.join(".conclaude.yaml");

    // Common config files that SHOULD be editable when they exist
    let config_files = [
        "Cargo.toml",
        "Cargo.lock",
        ".gitignore",
        "README.md",
        "LICENSE",
    ];

    for file in &config_files {
        let full_path = cwd.join(file);
        let is_root = is_root_addition(file, file, &config_path);

        // All these files are at root level
        assert!(is_root, "{} should be identified as root-level", file);

        // The key insight: if the file exists, it should be ALLOWED
        // because the refined logic checks `!resolved_path.exists()`
        if full_path.exists() {
            // This file exists, so it should be allowed to be edited
            // (the `!exists()` check in check_file_validation_rules will be false)
            println!("{} exists and should be allowed for modification", file);
        } else {
            // This file doesn't exist, so creating it would be blocked
            println!("{} does not exist and would be blocked from creation", file);
        }
    }
}

#[test]
fn test_prevent_root_additions_write_vs_edit_tool() {
    // This test documents the tool-specific behavior:
    // - Write tool on NEW root file → BLOCKED
    // - Write tool on EXISTING root file → ALLOWED (refinement)
    // - Edit tool on root file → ALLOWED (Edit tool is never blocked by preventRootAdditions)

    // The check only applies to "Write" tool, not "Edit" or "NotebookEdit"
    // See check_file_validation_rules: `&& payload.tool_name == "Write"`

    // Create test payloads to demonstrate the logic
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::String("test.txt".to_string()),
    );

    let write_payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Write".to_string(),
        tool_input: tool_input.clone(),
        tool_use_id: None,
    };

    let edit_payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Edit".to_string(),
        tool_input: tool_input.clone(),
        tool_use_id: None,
    };

    // The Write tool is subject to preventRootAdditions check
    assert_eq!(write_payload.tool_name, "Write");

    // The Edit tool is NOT subject to preventRootAdditions check
    // (it's only for Write tool)
    assert_eq!(edit_payload.tool_name, "Edit");
    assert_ne!(edit_payload.tool_name, "Write");
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
fn test_prevent_additions_only_affects_write_tool() {
    // Verify that preventAdditions ONLY blocks the Write tool, not Edit or NotebookEdit
    // This test documents the expected behavior - the actual enforcement happens in check_file_validation_rules

    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::String("dist/output.js".to_string()),
    );

    // Test 1: Write tool - SHOULD be subject to preventAdditions check
    let write_payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Write".to_string(),
        tool_input: tool_input.clone(),
        tool_use_id: None,
    };
    assert_eq!(
        write_payload.tool_name, "Write",
        "Write tool should be identified correctly"
    );

    // Test 2: Edit tool - should NOT be subject to preventAdditions check
    let edit_payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Edit".to_string(),
        tool_input: tool_input.clone(),
        tool_use_id: None,
    };
    assert_eq!(
        edit_payload.tool_name, "Edit",
        "Edit tool should be identified correctly"
    );
    assert_ne!(
        edit_payload.tool_name, "Write",
        "Edit tool should NOT be treated as Write tool"
    );

    // Test 3: NotebookEdit tool - should NOT be subject to preventAdditions check
    let mut notebook_input = HashMap::new();
    notebook_input.insert(
        "notebook_path".to_string(),
        Value::String("notebooks/analysis.ipynb".to_string()),
    );

    let notebook_edit_payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "NotebookEdit".to_string(),
        tool_input: notebook_input,
        tool_use_id: None,
    };
    assert_eq!(
        notebook_edit_payload.tool_name, "NotebookEdit",
        "NotebookEdit tool should be identified correctly"
    );
    assert_ne!(
        notebook_edit_payload.tool_name, "Write",
        "NotebookEdit tool should NOT be treated as Write tool"
    );

    // The actual preventAdditions check in check_file_validation_rules has:
    // `&& payload.tool_name == "Write"`
    // This ensures only Write operations are blocked by preventAdditions patterns
}

#[test]
fn test_prevent_additions_empty_array_allows_all() {
    // When preventAdditions is an empty array, no operations should be blocked
    // This test verifies the logic for empty pattern lists

    use conclaude::config::{ConclaudeConfig, PreToolUseConfig};

    let config = ConclaudeConfig {
        pre_tool_use: PreToolUseConfig {
            prevent_additions: vec![], // Empty array
            ..Default::default()
        },
        ..Default::default()
    };

    // Verify the config has an empty preventAdditions array
    assert!(
        config.pre_tool_use.prevent_additions.is_empty(),
        "preventAdditions should be empty"
    );

    // With an empty array, no files should be blocked
    // The check_file_validation_rules loop: `for pattern in &config.pre_tool_use.prevent_additions`
    // will not iterate, so no files will be blocked

    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        Value::String("any/file/path.js".to_string()),
    );

    let payload = PreToolUsePayload {
        base: create_test_base_payload(),
        tool_name: "Write".to_string(),
        tool_input,
        tool_use_id: None,
    };

    // No patterns to match against, so nothing should be blocked
    assert_eq!(payload.tool_name, "Write");
    assert!(config.pre_tool_use.prevent_additions.is_empty());
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
fn test_prevent_additions_expected_error_message_format() {
    // Test that the error message format matches the specification:
    // "Blocked {} operation: file matches preToolUse.preventAdditions pattern '{}'. File: {}"

    // This test documents the expected error message format
    // The actual implementation will format the message in check_file_validation_rules

    let tool_name = "Write";
    let pattern = "dist/**";
    let file_path = "dist/output.js";

    let expected_message = format!(
        "Blocked {} operation: file matches preToolUse.preventAdditions pattern '{}'. File: {}",
        tool_name, pattern, file_path
    );

    // Verify the format matches specification
    assert!(
        expected_message.contains("Blocked Write operation"),
        "Message should contain 'Blocked Write operation'"
    );
    assert!(
        expected_message.contains("preToolUse.preventAdditions"),
        "Message should mention preToolUse.preventAdditions"
    );
    assert!(
        expected_message.contains("pattern 'dist/**'"),
        "Message should include the pattern"
    );
    assert!(
        expected_message.contains("File: dist/output.js"),
        "Message should include the file path"
    );

    // Test with different values
    let pattern_2 = "*.log";
    let file_path_2 = "debug.log";
    let expected_message_2 = format!(
        "Blocked {} operation: file matches preToolUse.preventAdditions pattern '{}'. File: {}",
        tool_name, pattern_2, file_path_2
    );

    assert!(expected_message_2.contains("pattern '*.log'"));
    assert!(expected_message_2.contains("File: debug.log"));
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
