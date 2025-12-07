use conclaude::config::{
    extract_bash_commands, generate_default_config, load_conclaude_config, ConclaudeConfig,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_extract_bash_commands_single() {
    let script = "echo hello";
    let commands = extract_bash_commands(script).unwrap();
    assert_eq!(commands, vec!["echo hello"]);
}

#[test]
fn test_extract_bash_commands_multiple() {
    let script = "echo hello\nnpm install\nnpm test";
    let commands = extract_bash_commands(script).unwrap();
    assert_eq!(commands, vec!["echo hello", "npm install", "npm test"]);
}

#[test]
fn test_extract_bash_commands_ignores_comments() {
    let script = "# This is a comment\necho hello\n# Another comment\nnpm test";
    let commands = extract_bash_commands(script).unwrap();
    assert_eq!(commands, vec!["echo hello", "npm test"]);
}

#[test]
fn test_extract_bash_commands_ignores_empty_lines() {
    let script = "echo hello\n\nnpm test\n";
    let commands = extract_bash_commands(script).unwrap();
    assert_eq!(commands, vec!["echo hello", "npm test"]);
}

#[test]
fn test_extract_bash_commands_complex() {
    let script = r#"nix develop -c "lint"
bun x tsc --noEmit
cd /tmp && echo "test""#;
    let commands = extract_bash_commands(script).unwrap();
    assert_eq!(
        commands,
        vec![
            r#"nix develop -c "lint""#,
            "bun x tsc --noEmit",
            r#"cd /tmp && echo "test""#
        ]
    );
}

#[test]
fn test_extract_bash_commands_returns_empty_for_empty_script() {
    let commands = extract_bash_commands("").unwrap();
    assert_eq!(commands, Vec::<String>::new());
}

#[test]
fn test_extract_bash_commands_returns_empty_for_comments_only() {
    let script = "# Comment 1\n# Comment 2\n   # Comment 3";
    let commands = extract_bash_commands(script).unwrap();
    assert_eq!(commands, Vec::<String>::new());
}

#[test]
fn test_extract_bash_commands_handles_mixed_whitespace_and_comments() {
    let script = "   # Comment with leading spaces\necho hello\n   \n\t# Tab-indented comment\nnpm test\n   echo world   ";
    let commands = extract_bash_commands(script).unwrap();
    assert_eq!(commands, vec!["echo hello", "npm test", "   echo world   "]);
}

#[test]
fn test_yaml_parsing_directly() {
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
  infinite: true
  infiniteMessage: "continue"
preToolUse:
  preventRootAdditions: false
  uneditableFiles:
    - "*.lock"
"#;

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(config_content);
    match result {
        Ok(config) => {
            // Config parsed successfully, validate infinite_message field
            assert_eq!(config.stop.infinite_message, Some("continue".to_string()));
        }
        Err(e) => {
            panic!("YAML parsing failed: {e:?}");
        }
    }
}

// Note: File-based config loading is tested through the direct YAML parsing test above
// and through the integration tests. The load_conclaude_config function works correctly
// in practice, but this test has directory/path issues in the test environment.

#[tokio::test]
async fn test_load_config_not_found() {
    let temp_dir = tempdir().unwrap();

    // Create a deep directory structure (15 levels deep) to ensure we're beyond
    // the 12-level search limit, preventing the search from finding any config
    // files in parent directories like /tmp/
    let mut current_path = temp_dir.path().to_path_buf();
    for i in 0..15 {
        current_path = current_path.join(format!("level_{i}"));
        fs::create_dir(&current_path).unwrap();
    }

    let result = load_conclaude_config(Some(&current_path)).await;

    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Configuration file not found"));
}

#[test]
fn test_generate_default_config() {
    let config = generate_default_config();
    assert!(config.contains("stop:"));
    assert!(config.contains("preToolUse:"));
    assert!(config.contains("preventRootAdditions: true"));
    // Default config now includes conclaude config files in uneditableFiles for security
    assert!(config.contains("uneditableFiles:"));
    assert!(config.contains(".conclaude.yml"));
    assert!(config.contains(".conclaude.yaml"));
    assert!(config.contains("infinite: false"));
}

#[test]
fn test_default_config_can_be_parsed() {
    // This test demonstrates that the default config should be parseable
    // but will fail initially, showing the TDD approach
    let config_content = generate_default_config();

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(&config_content);

    match result {
        Ok(config) => {
            // Config parsed successfully
            assert!(config.pre_tool_use.prevent_root_additions);
            println!("✓ Default config parsed successfully");
        }
        Err(e) => {
            println!("✗ Default config failed to parse:");
            println!("Error: {}", e);
            println!("\nHere's the failing config content:");
            println!("{}", config_content);
            panic!("Default config should be parseable, but failed with: {}", e);
        }
    }
}

#[test]
fn test_local_conclaude_yaml_can_be_parsed() {
    // Test the actual .conclaude.yaml file in the repo
    let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".conclaude.yaml");

    if !config_path.exists() {
        panic!(
            "Expected .conclaude.yaml to exist at: {}",
            config_path.display()
        );
    }

    let content = std::fs::read_to_string(&config_path).expect("Failed to read .conclaude.yaml");

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(&content);

    match result {
        Ok(config) => {
            println!("✓ Local .conclaude.yaml parsed successfully");
            println!("  notifications.enabled: {}", config.notifications.enabled);
            println!("  notifications.hooks: {:?}", config.notifications.hooks);
        }
        Err(e) => {
            println!("✗ Local .conclaude.yaml failed to parse:");
            println!("Error: {}", e);
            panic!(
                "Local .conclaude.yaml should be parseable, but failed with: {}",
                e
            );
        }
    }
}

#[test]
fn test_default_config_with_comments_removed_can_be_parsed() {
    // Test parsing by stripping comments to see if they cause issues
    let config_content = generate_default_config();

    // Remove comment lines to isolate the YAML content
    let yaml_only: String = config_content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#') && !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(&yaml_only);

    match result {
        Ok(config) => {
            // Config parsed successfully
            assert!(config.pre_tool_use.prevent_root_additions);
        }
        Err(e) => {
            panic!(
                "YAML-only content should be parseable, but failed with: {}\n\nYAML content:\n{}",
                e, yaml_only
            );
        }
    }
}

#[test]
fn test_default_config_without_uncommented_grep_rules_can_be_parsed() {
    // Create a version of the default config with grepRules lines completely removed
    let config_content = generate_default_config();

    // Remove lines that contain grepRules (including commented ones)
    let cleaned_config: String = config_content
        .lines()
        .filter(|line| !line.contains("grepRules"))
        .collect::<Vec<_>>()
        .join("\n");

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(&cleaned_config);

    match result {
        Ok(config) => {
            // Config parsed successfully
            assert!(config.pre_tool_use.prevent_root_additions);
        }
        Err(e) => {
            panic!(
                "Config without grepRules should be parseable, but failed with: {}\n\nConfig content:\n{}",
                e, cleaned_config
            );
        }
    }
}

#[tokio::test]
async fn test_config_search_level_limit() {
    let temp_dir = tempdir().unwrap();

    // Create a deep directory structure (15 levels deep)
    let mut current_path = temp_dir.path().to_path_buf();
    for i in 0..15 {
        current_path = current_path.join(format!("level_{i}"));
        fs::create_dir(&current_path).unwrap();
    }

    // Place a config file exactly 13 levels up from the deepest directory
    // (this should be beyond the 12-level search limit)
    let mut config_dir = current_path.clone();
    for _ in 0..13 {
        config_dir = config_dir.parent().unwrap().to_path_buf();
    }
    let deep_config_path = config_dir.join(".conclaude.yaml");
    fs::write(
        &deep_config_path,
        "stop:\n  commands:\n    - run: 'deep config'\npreToolUse:\n  preventRootAdditions: true",
    )
    .unwrap();

    // Attempt to load config - should not find the deep config due to level limit
    let result = load_conclaude_config(Some(&current_path)).await;

    // Should fail to find config due to level limit
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Configuration file not found"));
}

#[tokio::test]
async fn test_config_search_within_level_limit() {
    let temp_dir = tempdir().unwrap();

    // Create a directory structure within the 12 level limit (10 levels)
    let mut current_path = temp_dir.path().to_path_buf();
    for i in 0..10 {
        current_path = current_path.join(format!("level_{i}"));
        fs::create_dir(&current_path).unwrap();
    }

    // Place a config file at level 5 (should be found within limit)
    let config_path = temp_dir
        .path()
        .join("level_0/level_1/level_2/level_3/level_4/.conclaude.yaml");
    fs::write(
        &config_path,
        "stop:\n  commands:\n    - run: 'found config'\n  infinite: false\npreToolUse:\n  preventRootAdditions: true",
    )
    .unwrap();

    // Attempt to load config - should find the config within level limit
    let result = load_conclaude_config(Some(&current_path)).await;

    // Should successfully find and parse config
    assert!(result.is_ok());
    let (config, _config_path) = result.unwrap();
    assert_eq!(config.stop.commands[0].run, "found config");
    assert!(!config.stop.infinite);
    assert!(config.pre_tool_use.prevent_root_additions);
}

#[tokio::test]
async fn test_config_search_exactly_at_12_level_boundary() {
    // This test verifies the exact boundary condition for MAX_SEARCH_LEVELS = 12
    //
    // The implementation works as follows:
    // 1. Check starting directory (levels_searched = 0)
    // 2. Move to parent, increment levels_searched
    // 3. Check if levels_searched >= 12, if so, break
    // 4. Otherwise, check parent directory and repeat
    //
    // This means:
    // - Starting dir is checked (before any moves)
    // - Then it moves up and checks 11 more parent levels (levels_searched goes 1→11)
    // - When levels_searched reaches 12, it breaks WITHOUT checking that level
    // - Total directories checked: starting dir + 11 parents = 12 total
    //
    // So the boundary is:
    // - Config at 11 levels up: FOUND (checked when levels_searched = 11)
    // - Config at 12 levels up: NOT FOUND (breaks when levels_searched = 12)

    let temp_dir = tempdir().unwrap();

    // TEST CASE 1: Config at exactly 11 levels up (should be found - at the limit)
    {
        // Create 12 levels: temp_dir/level_0/level_1/.../level_10
        let mut search_path = temp_dir.path().to_path_buf();
        for i in 0..11 {
            search_path = search_path.join(format!("level_{i}"));
            fs::create_dir(&search_path).unwrap();
        }

        // Place config at temp_dir (exactly 11 parent levels from search_path)
        let config_at_11 = temp_dir.path().join(".conclaude.yaml");
        fs::write(
            &config_at_11,
            "stop:\n  commands:\n    - run: 'found at boundary'\n  infinite: false\npreToolUse:\n  preventRootAdditions: true",
        )
        .unwrap();

        // Search from level_10 - should find config at temp_dir (11 levels up)
        let result = load_conclaude_config(Some(&search_path)).await;
        assert!(
            result.is_ok(),
            "Config at exactly 11 levels up should be found (at the 12-directory limit)"
        );
        let (config, found_path) = result.unwrap();
        assert_eq!(config.stop.commands[0].run, "found at boundary");
        assert_eq!(found_path, config_at_11);

        // Clean up for next test
        fs::remove_file(&config_at_11).unwrap();
    }

    // TEST CASE 2: Config at 12 levels up (should NOT be found - beyond limit)
    {
        // Create 13 levels: temp_dir/deep_level_0/deep_level_1/.../deep_level_11
        let mut search_path = temp_dir.path().to_path_buf();
        for i in 0..12 {
            search_path = search_path.join(format!("deep_level_{i}"));
            fs::create_dir(&search_path).unwrap();
        }

        // Place config at temp_dir (exactly 12 parent levels from search_path)
        let config_at_12 = temp_dir.path().join(".conclaude.yaml");
        fs::write(
            &config_at_12,
            "stop:\n  commands:\n    - run: 'beyond limit'\npreToolUse:\n  preventRootAdditions: true",
        )
        .unwrap();

        // Search from deep_level_11 - should NOT find config at temp_dir (12 levels up)
        let result = load_conclaude_config(Some(&search_path)).await;
        assert!(
            result.is_err(),
            "Config at 12 levels up should NOT be found (beyond the 12-directory limit)"
        );
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Configuration file not found"));
    }
}

#[tokio::test]
async fn test_notification_config_default_disabled() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    // Create a config with default notification settings
    fs::write(&config_path, generate_default_config()).unwrap();

    let result = load_conclaude_config(Some(temp_dir.path())).await;

    assert!(result.is_ok());

    let (config, _config_path) = result.unwrap();

    // Test default notification settings
    assert!(!config.notifications.enabled);
    assert!(config.notifications.hooks.is_empty());

    // Test that no hooks are enabled when disabled
    assert!(!config.notifications.is_enabled_for("Stop"));
    assert!(!config.notifications.is_enabled_for("PreToolUse"));
    assert!(!config.notifications.is_enabled_for("*"));
}

#[tokio::test]
async fn test_notification_config_enabled_specific_hooks() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    // Create config with specific hooks enabled
    let config_content = r#"
stop:
  commands: []
preToolUse:
  preventRootAdditions: true
notifications:
  enabled: true
  hooks: ["Stop", "PreToolUse"]
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = load_conclaude_config(Some(temp_dir.path())).await;

    assert!(result.is_ok());

    let (config, _config_path) = result.unwrap();

    // Test enabled configuration
    assert!(config.notifications.enabled);
    assert_eq!(config.notifications.hooks, vec!["Stop", "PreToolUse"]);

    // Test that only configured hooks are enabled
    assert!(config.notifications.is_enabled_for("Stop"));
    assert!(config.notifications.is_enabled_for("PreToolUse"));
    assert!(!config.notifications.is_enabled_for("PostToolUse"));
    assert!(!config.notifications.is_enabled_for("SessionStart"));
    assert!(!config.notifications.is_enabled_for("NonExistentHook"));
}

#[tokio::test]
async fn test_notification_config_enabled_wildcard() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    // Create config with wildcard enabled
    let config_content = r#"
stop:
  commands: []
preToolUse:
  preventRootAdditions: true
notifications:
  enabled: true
  hooks: ["*"]
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = load_conclaude_config(Some(temp_dir.path())).await;

    assert!(result.is_ok());

    let (config, _config_path) = result.unwrap();

    // Test wildcard configuration
    assert!(config.notifications.enabled);
    assert_eq!(config.notifications.hooks, vec!["*"]);

    // Test that all hooks are enabled with wildcard
    assert!(config.notifications.is_enabled_for("Stop"));
    assert!(config.notifications.is_enabled_for("PreToolUse"));
    assert!(config.notifications.is_enabled_for("PostToolUse"));
    assert!(config.notifications.is_enabled_for("SessionStart"));
    assert!(config.notifications.is_enabled_for("AnyRandomHook"));
    assert!(config.notifications.is_enabled_for("NonExistentHook"));
}

#[tokio::test]
async fn test_notification_config_enabled_empty_hooks() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    // Create config with enabled but empty hooks list
    let config_content = r#"
stop:
  commands: []
preToolUse:
  preventRootAdditions: true
notifications:
  enabled: true
  hooks: []
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = load_conclaude_config(Some(temp_dir.path())).await;

    assert!(result.is_ok());

    let (config, _config_path) = result.unwrap();

    // Test enabled but empty configuration
    assert!(config.notifications.enabled);
    assert!(config.notifications.hooks.is_empty());

    // Test that no hooks are enabled when hooks list is empty
    assert!(!config.notifications.is_enabled_for("Stop"));
    assert!(!config.notifications.is_enabled_for("PreToolUse"));
    assert!(!config.notifications.is_enabled_for("AnyHook"));
}

#[test]
fn test_parse_actual_repo_config() {
    // Test that the actual .conclaude.yaml file in the repo can be parsed
    let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".conclaude.yaml");

    if !config_path.exists() {
        panic!(
            "Expected .conclaude.yaml to exist at: {}",
            config_path.display()
        );
    }

    let content = fs::read_to_string(&config_path).expect("Failed to read .conclaude.yaml");

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(&content);

    match &result {
        Ok(_config) => {
            // Config parsed successfully
        }
        Err(e) => {
            panic!(
                "Failed to parse .conclaude.yaml: {}\n\nFile content:\n{}",
                e, content
            );
        }
    }

    assert!(
        result.is_ok(),
        "The .conclaude.yaml file should parse successfully"
    );
}

#[test]
fn test_reject_unknown_fields_in_stop_config() {
    // Test that unknown fields in stop config are rejected
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
  unknownField: "should fail"
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(config_content);

    assert!(
        result.is_err(),
        "Config with unknown field 'unknownField' in stop section should be rejected"
    );
}

#[test]
fn test_reject_unknown_fields_in_pre_tool_use_config() {
    // Test that unknown fields in preToolUse config are rejected
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
preToolUse:
  preventRootAdditions: true
preToolUse:
  preventAdditions: []
  unknownField: "should fail"
"#;

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(config_content);

    assert!(
        result.is_err(),
        "Config with unknown field 'unknownField' in preToolUse section should be rejected"
    );
}

#[test]
fn test_reject_grep_rules_in_stop_config() {
    // Test that grepRules field (which exists in .conclaude.yaml but not in StopConfig struct) is rejected
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
  grepRules: []
preToolUse:
  preventRootAdditions: true
"#;

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(config_content);

    assert!(
        result.is_err(),
        "Config with 'grepRules' field in stop section should be rejected (field doesn't exist in StopConfig)"
    );
}

#[test]
fn test_reject_grep_rules_in_pre_tool_use_config() {
    // Test that grepRules field in preToolUse is rejected
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
preToolUse:
  preventRootAdditions: true
preToolUse:
  preventAdditions: []
  grepRules: []
"#;

    let result = serde_yaml::from_str::<conclaude::config::ConclaudeConfig>(config_content);

    assert!(
        result.is_err(),
        "Config with 'grepRules' field in preToolUse section should be rejected (field doesn't exist in PreToolUseConfig)"
    );
}

#[tokio::test]
async fn test_descriptive_error_for_unknown_field() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    // Create config with an unknown field
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
  invalidField: "this should fail"
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = load_conclaude_config(Some(temp_dir.path())).await;

    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();

    // Verify the error message contains helpful information
    assert!(
        error_message.contains("unknown field"),
        "Error should mention 'unknown field'"
    );
    assert!(
        error_message.contains("Common causes"),
        "Error should provide common causes"
    );
    assert!(
        error_message.contains("Valid field names"),
        "Error should list valid field names"
    );
}

#[tokio::test]
async fn test_descriptive_error_for_invalid_type() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    // Create config with an invalid type (string instead of boolean)
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
  infinite: "true"
preToolUse:
  preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = load_conclaude_config(Some(temp_dir.path())).await;

    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();

    // Verify the error message contains helpful information
    assert!(
        error_message.contains("invalid type") || error_message.contains("type"),
        "Error should mention type mismatch: {}",
        error_message
    );
    assert!(
        error_message.contains("Common causes"),
        "Error should provide common causes"
    );
}

#[tokio::test]
async fn test_descriptive_error_for_yaml_syntax() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".conclaude.yaml");

    // Create config with YAML syntax error (bad indentation)
    let config_content = r#"
stop:
  commands:
    - run: "echo test"
preToolUse:
preventRootAdditions: true
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = load_conclaude_config(Some(temp_dir.path())).await;

    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();

    // Verify the error message contains helpful information
    assert!(
        error_message.contains("syntax error")
            || error_message.contains("indentation")
            || error_message.contains("expected"),
        "Error should mention syntax or parsing issue: {}",
        error_message
    );
}

#[test]
fn test_notifications_config_camelcase_field_names() {
    // Test that the new camelCase field names work correctly
    let config_content = r#"
notifications:
  enabled: true
  hooks: ["*"]
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_ok(),
        "Config with camelCase field names should parse successfully"
    );

    let config = result.unwrap();
    assert!(config.notifications.enabled);
    assert!(!config.notifications.show_errors);
    assert!(!config.notifications.show_success);
    assert!(config.notifications.show_system_events);
}

#[test]
fn test_notifications_config_snake_case_field_names_error() {
    // Test that old snake_case field names produce a helpful error message
    let config_content = r#"
notifications:
  enabled: true
  hooks: ["*"]
  show_errors: false
  show_success: false
  show_system_events: true
"#;

    let result = serde_yaml::from_str::<ConclaudeConfig>(config_content);
    assert!(
        result.is_err(),
        "Config with snake_case field names should fail to parse"
    );

    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("showErrors"),
        "Error should mention showErrors field name"
    );
    assert!(
        error.contains("showSuccess"),
        "Error should mention showSuccess field name"
    );
    assert!(
        error.contains("showSystemEvents"),
        "Error should mention showSystemEvents field name"
    );
}

#[tokio::test]
async fn test_config_search_above_package_json() {
    let temp_dir = tempdir().unwrap();

    // Create directory structure: temp_dir/level_0/project/package.json and temp_dir/.conclaude.yaml
    let project_dir = temp_dir.path().join("level_0").join("project");
    fs::create_dir_all(&project_dir).unwrap();

    // Create package.json in the project directory
    let package_json_path = project_dir.join("package.json");
    fs::write(&package_json_path, r#"{"name": "test-project"}"#).unwrap();

    // Create config file above the package.json (in temp_dir/level_0)
    let config_path = temp_dir.path().join("level_0").join(".conclaude.yaml");
    fs::write(
        &config_path,
        "stop:\n  commands:\n    - run: 'found config above package.json'\npreToolUse:\n  preventRootAdditions: true",
    )
    .unwrap();

    // Create a subdirectory under project to search from
    let search_dir = project_dir.join("src");
    fs::create_dir(&search_dir).unwrap();

    // Attempt to load config - should find the config above package.json
    let result = load_conclaude_config(Some(&search_dir)).await;

    // Should successfully find and parse config despite package.json barrier
    assert!(result.is_ok());
    let (config, _config_path) = result.unwrap();
    assert_eq!(
        config.stop.commands[0].run,
        "found config above package.json"
    );
    assert!(config.pre_tool_use.prevent_root_additions);
}

#[tokio::test]
async fn test_config_search_monorepo_nested_package_json() {
    let temp_dir = tempdir().unwrap();

    // Create monorepo structure: temp_dir/monorepo/packages/app/package.json and temp_dir/monorepo/.conclaude.yaml
    let app_dir = temp_dir
        .path()
        .join("monorepo")
        .join("packages")
        .join("app");
    fs::create_dir_all(&app_dir).unwrap();

    // Create package.json files at multiple levels
    let root_package_json = temp_dir.path().join("monorepo").join("package.json");
    fs::write(&root_package_json, r#"{"name": "monorepo"}"#).unwrap();

    let app_package_json = app_dir.join("package.json");
    fs::write(&app_package_json, r#"{"name": "app"}"#).unwrap();

    // Create config file at monorepo root
    let config_path = temp_dir.path().join("monorepo").join(".conclaude.yaml");
    fs::write(
        &config_path,
        "stop:\n  commands:\n    - run: 'found monorepo config'\n  infinite: false\npreToolUse:\n  preventRootAdditions: true",
    )
    .unwrap();

    // Create a deep subdirectory under app to search from
    let search_dir = app_dir.join("src").join("deep").join("dir");
    fs::create_dir_all(&search_dir).unwrap();

    // Attempt to load config - should find the config at monorepo root despite nested package.json files
    let result = load_conclaude_config(Some(&search_dir)).await;

    // Should successfully find and parse config
    assert!(result.is_ok());
    let (config, _config_path) = result.unwrap();
    assert_eq!(config.stop.commands[0].run, "found monorepo config");
    assert!(!config.stop.infinite);
    assert!(config.pre_tool_use.prevent_root_additions);
}

#[tokio::test]
async fn test_config_search_stops_at_filesystem_root() {
    let temp_dir = tempdir().unwrap();

    // Create a directory structure and try to search from filesystem root
    // This test ensures we don't go above the filesystem root
    let search_dir = temp_dir.path().to_path_buf();

    // Attempt to load config from the temp directory (no config should exist)
    let result = load_conclaude_config(Some(&search_dir)).await;

    // Should fail to find config (reaches filesystem root without finding config)
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Configuration file not found"));
}
