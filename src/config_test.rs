use crate::config::{parse_and_validate_config, suggest_similar_fields, ConclaudeConfig};
use std::path::Path;

#[test]
fn test_suggest_similar_fields_common_typo() {
    // Test common typo: "showStdOut" should suggest "showStdout"
    let suggestions = suggest_similar_fields("showStdOut", "commands");
    assert!(
        !suggestions.is_empty(),
        "Should suggest fields for common typo"
    );
    assert_eq!(
        suggestions[0], "showStdout",
        "First suggestion should be 'showStdout'"
    );
}

#[test]
fn test_suggest_similar_fields_case_insensitive() {
    // Test case-insensitive matching: "INFINITE" should suggest "infinite"
    let suggestions = suggest_similar_fields("INFINITE", "stop");
    assert!(
        !suggestions.is_empty(),
        "Should suggest fields ignoring case"
    );
    assert!(
        suggestions.contains(&"infinite".to_string()),
        "Should suggest 'infinite' for 'INFINITE'"
    );
}

#[test]
fn test_suggest_similar_fields_distance_threshold() {
    // Test that only suggestions within distance 3 are returned
    // "infinit" (distance 1) should be suggested
    let suggestions = suggest_similar_fields("infinit", "stop");
    assert!(
        suggestions.contains(&"infinite".to_string()),
        "Should suggest 'infinite' for 'infinit' (distance 1)"
    );

    // "infinte" (distance 1, missing 'i') should be suggested
    let suggestions = suggest_similar_fields("infinte", "stop");
    assert!(
        suggestions.contains(&"infinite".to_string()),
        "Should suggest 'infinite' for 'infinte' (distance 1)"
    );

    // "wxyz" has distance > 3 from all stop fields, should not suggest anything
    let suggestions = suggest_similar_fields("wxyz", "stop");
    assert!(
        suggestions.is_empty(),
        "Should not suggest anything for 'wxyz' (distance > 3 from all fields)"
    );
}

#[test]
fn test_suggest_similar_fields_no_close_matches() {
    // Test that empty results are returned when no close matches exist
    let suggestions = suggest_similar_fields("completelywrongfield", "stop");
    assert!(
        suggestions.is_empty(),
        "Should return empty for field with no close matches"
    );

    let suggestions = suggest_similar_fields("abcdefgh", "rules");
    assert!(
        suggestions.is_empty(),
        "Should return empty when distance exceeds threshold"
    );
}

#[test]
fn test_suggest_similar_fields_sorted_by_distance() {
    // Test that suggestions are sorted by distance (closest first)
    // "messag" (distance 1 from "message") should come before anything with higher distance
    let suggestions = suggest_similar_fields("messag", "commands");
    if !suggestions.is_empty() {
        assert_eq!(
            suggestions[0], "message",
            "Closest match should be first in suggestions"
        );
    }
}

#[test]
fn test_suggest_similar_fields_max_three_suggestions() {
    // Test that at most 3 suggestions are returned
    let suggestions = suggest_similar_fields("sho", "commands");
    assert!(
        suggestions.len() <= 3,
        "Should return at most 3 suggestions, got {}",
        suggestions.len()
    );
}

#[test]
fn test_suggest_similar_fields_invalid_section() {
    // Test that empty results are returned for invalid section
    let suggestions = suggest_similar_fields("infinite", "invalid_section");
    assert!(
        suggestions.is_empty(),
        "Should return empty for invalid section"
    );
}

#[test]
fn test_suggest_similar_fields_notifications_section() {
    // Test suggestions for notifications section
    let suggestions = suggest_similar_fields("enable", "notifications");
    assert!(
        suggestions.contains(&"enabled".to_string()),
        "Should suggest 'enabled' for 'enable' in notifications section"
    );
}

#[test]
fn test_suggest_similar_fields_pretooluse_section() {
    // Test suggestions for preToolUse section with camelCase field
    let suggestions = suggest_similar_fields("preventRootAddition", "preToolUse");
    assert!(
        suggestions.contains(&"preventRootAdditions".to_string()),
        "Should suggest 'preventRootAdditions' for 'preventRootAddition'"
    );
}

#[test]
fn test_config_without_rules_section_works() {
    // Test that configuration without rules section works normally
    let valid_config = r#"
preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;

    let result = parse_and_validate_config(valid_config, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Should accept config without rules section: {:?}",
        result.err()
    );
}

#[test]
fn test_permission_request_valid_config() {
    // Test valid permissionRequest configuration
    let config_yaml = r#"
permissionRequest:
  default: allow
  allow:
    - Read
    - Write
  deny:
    - Bash

stop:
  commands: []
  infinite: false

preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  toolUsageValidation: []

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;

    let result = parse_and_validate_config(config_yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Valid permissionRequest config should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert!(
        config.permission_request.is_some(),
        "permission_request should be populated"
    );
    let pr = config.permission_request.unwrap();
    assert_eq!(pr.default, "allow");
    assert_eq!(pr.allow.as_ref().unwrap().len(), 2);
    assert_eq!(pr.deny.as_ref().unwrap().len(), 1);
}

#[test]
fn test_permission_request_invalid_default() {
    // Test that invalid default value is rejected
    let config_yaml = r#"
permissionRequest:
  default: invalid_value

stop:
  commands: []
  infinite: false

preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  toolUsageValidation: []

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;

    let result = parse_and_validate_config(config_yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Invalid default value should fail validation"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("allow") && error.contains("deny"),
        "Error message should mention valid values"
    );
}

#[test]
fn test_permission_request_optional() {
    // Test that permissionRequest is optional
    let config_yaml = r#"
stop:
  commands: []
  infinite: false

preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  toolUsageValidation: []

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;

    let result = parse_and_validate_config(config_yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config without permissionRequest should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert!(
        config.permission_request.is_none(),
        "permission_request should be None when not specified"
    );
}

#[test]
fn test_uneditable_file_rule_simple_string_format() {
    // Test that simple string patterns deserialize correctly
    let yaml = r#"
preToolUse:
  uneditableFiles:
  - "*.lock"
  - ".env"
  preventAdditions: []
  preventRootAdditions: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.pre_tool_use.uneditable_files.len(), 2);

    // Verify patterns extracted correctly
    assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
    assert_eq!(config.pre_tool_use.uneditable_files[1].pattern(), ".env");

    // Verify no custom messages
    assert!(config.pre_tool_use.uneditable_files[0].message().is_none());
    assert!(config.pre_tool_use.uneditable_files[1].message().is_none());
}

#[test]
fn test_uneditable_file_rule_detailed_object_format() {
    // Test that detailed objects with pattern and message deserialize correctly
    let yaml = r#"
preToolUse:
  uneditableFiles:
  - pattern: "*.lock"
    message: "Lock files are auto-generated. Run 'npm install' to update."
  - pattern: ".env"
    message: "Environment files contain secrets."
  preventAdditions: []
  preventRootAdditions: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.pre_tool_use.uneditable_files.len(), 2);

    // Verify patterns extracted correctly
    assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
    assert_eq!(config.pre_tool_use.uneditable_files[1].pattern(), ".env");

    // Verify custom messages
    assert_eq!(
        config.pre_tool_use.uneditable_files[0].message(),
        Some("Lock files are auto-generated. Run 'npm install' to update.")
    );
    assert_eq!(
        config.pre_tool_use.uneditable_files[1].message(),
        Some("Environment files contain secrets.")
    );
}

#[test]
fn test_uneditable_file_rule_mixed_format() {
    // Test that arrays can mix both simple strings and detailed objects
    let yaml = r#"
preToolUse:
  uneditableFiles:
  - "*.lock"
  - pattern: ".env"
    message: "Secrets must not be committed."
  - "package.json"
  preventAdditions: []
  preventRootAdditions: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.pre_tool_use.uneditable_files.len(), 3);

    // First is simple format
    assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
    assert!(config.pre_tool_use.uneditable_files[0].message().is_none());

    // Second is detailed format with message
    assert_eq!(config.pre_tool_use.uneditable_files[1].pattern(), ".env");
    assert_eq!(
        config.pre_tool_use.uneditable_files[1].message(),
        Some("Secrets must not be committed.")
    );

    // Third is simple format
    assert_eq!(
        config.pre_tool_use.uneditable_files[2].pattern(),
        "package.json"
    );
    assert!(config.pre_tool_use.uneditable_files[2].message().is_none());
}

#[test]
fn test_uneditable_file_rule_detailed_without_message() {
    // Test that detailed format without message field works (message is optional)
    let yaml = r#"
preToolUse:
  uneditableFiles:
  - pattern: "*.lock"
    preventAdditions: []
    preventRootAdditions: true
    toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.pre_tool_use.uneditable_files.len(), 1);
    assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
    assert!(config.pre_tool_use.uneditable_files[0].message().is_none());
}

#[test]
fn test_uneditable_file_rule_backward_compatibility() {
    // Test that existing configs with simple string format still work
    let yaml = r#"
preToolUse:
  preventRootAdditions: true
  uneditableFiles:
  - "*.lock"
  - ".env"
  - "package-lock.json"
  preventAdditions: []
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;

    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Backward compatible config should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.pre_tool_use.uneditable_files.len(), 3);

    // Verify all patterns are extracted correctly
    let patterns: Vec<&str> = config
        .pre_tool_use
        .uneditable_files
        .iter()
        .map(|r| r.pattern())
        .collect();
    assert_eq!(patterns, vec!["*.lock", ".env", "package-lock.json"]);
}

#[test]
fn test_uneditable_file_rule_agent_field_parsing_from_yaml() {
    // Test that agent field can be parsed from YAML correctly
    let yaml = r#"
stop:
  commands: []
preToolUse:
  uneditableFiles:
    - pattern: "*.lock"
      message: "Lock files are auto-generated"
      agent: "coder"
    - pattern: "dist/**"
      agent: "test*"
    - pattern: ".env*"
      message: "Environment files contain secrets"
    - "*.md"
"#;

    let config = parse_and_validate_config(yaml, Path::new("test.yaml")).unwrap();
    let rules = &config.pre_tool_use.uneditable_files;

    assert_eq!(rules.len(), 4);

    // First rule: has agent, message, and pattern
    assert_eq!(rules[0].pattern(), "*.lock");
    assert_eq!(rules[0].agent(), Some("coder"));
    assert_eq!(rules[0].message(), Some("Lock files are auto-generated"));

    // Second rule: has agent and pattern, no message
    assert_eq!(rules[1].pattern(), "dist/**");
    assert_eq!(rules[1].agent(), Some("test*"));
    assert_eq!(rules[1].message(), None);

    // Third rule: has message and pattern, no agent
    assert_eq!(rules[2].pattern(), ".env*");
    assert_eq!(rules[2].agent(), None);
    assert_eq!(
        rules[2].message(),
        Some("Environment files contain secrets")
    );

    // Fourth rule: simple format (only pattern)
    assert_eq!(rules[3].pattern(), "*.md");
    assert_eq!(rules[3].agent(), None);
    assert_eq!(rules[3].message(), None);
}

#[test]
fn test_stop_command_timeout_parsing() {
    let yaml = r#"
stop:
  commands:
  - run: "sleep 10"
    timeout: 30
  - run: "echo hello"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with timeout should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 2);
    assert_eq!(config.stop.commands[0].timeout, Some(30));
    assert_eq!(config.stop.commands[1].timeout, None);
}

#[test]
fn test_stop_command_timeout_invalid_too_large() {
    let yaml = r#"
stop:
  commands:
  - run: "echo test"
    timeout: 3601
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Config with timeout > 3600 should fail validation"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("timeout") || error.contains("3600"),
        "Error should mention timeout issue: {}",
        error
    );
}

#[test]
fn test_stop_command_timeout_invalid_zero() {
    let yaml = r#"
stop:
  commands:
  - run: "echo test"
    timeout: 0
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Config with timeout = 0 should fail validation"
    );
}

#[test]
fn test_subagent_stop_command_timeout_parsing() {
    let yaml = r#"
subagentStop:
  commands:
    "*":
      - run: "npm run lint"
        timeout: 60
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with subagent timeout should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let cmds = config.subagent_stop.commands.get("*").unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].timeout, Some(60));
}

#[test]
fn test_timeout_backward_compatibility() {
    let yaml = r#"
stop:
  commands:
  - run: "echo hello"
    message: "Testing"
    showStdout: true
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config without timeout should still parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands[0].timeout, None);
}

#[test]
fn test_user_prompt_submit_config_basic() {
    let yaml = r#"
userPromptSubmit:
  contextRules:
  - pattern: "sidebar"
    prompt: "Read the sidebar docs"
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Valid userPromptSubmit config should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.user_prompt_submit.context_rules.len(), 1);
    assert_eq!(
        config.user_prompt_submit.context_rules[0].pattern,
        "sidebar"
    );
    assert_eq!(
        config.user_prompt_submit.context_rules[0].prompt,
        "Read the sidebar docs"
    );
}

#[test]
fn test_user_prompt_submit_config_with_all_fields() {
    let yaml = r#"
userPromptSubmit:
  contextRules:
  - pattern: "auth|login"
    prompt: "Review authentication docs"
    enabled: true
    caseInsensitive: true
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with all fields should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let rule = &config.user_prompt_submit.context_rules[0];
    assert_eq!(rule.pattern, "auth|login");
    assert_eq!(rule.prompt, "Review authentication docs");
    assert_eq!(rule.enabled, Some(true));
    assert_eq!(rule.case_insensitive, Some(true));
}

#[test]
fn test_user_prompt_submit_config_invalid_regex() {
    let yaml = r#"
userPromptSubmit:
  contextRules:
  - pattern: "[invalid"
    prompt: "Test"
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Invalid regex pattern should fail validation"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("Invalid regex pattern") || error.contains("[invalid"),
        "Error should mention invalid regex: {}",
        error
    );
}

#[test]
fn test_user_prompt_submit_config_multiple_rules() {
    let yaml = r#"
userPromptSubmit:
  contextRules:
  - pattern: "sidebar"
    prompt: "Read sidebar docs"
  - pattern: "auth"
    prompt: "Read auth docs"
  - pattern: "database"
    prompt: "Read db docs"
    enabled: false
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Multiple context rules should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.user_prompt_submit.context_rules.len(), 3);

    // Verify first rule
    assert_eq!(
        config.user_prompt_submit.context_rules[0].pattern,
        "sidebar"
    );
    assert_eq!(
        config.user_prompt_submit.context_rules[0].enabled,
        Some(true)
    );

    // Verify third rule with enabled: false
    assert_eq!(
        config.user_prompt_submit.context_rules[2].pattern,
        "database"
    );
    assert_eq!(
        config.user_prompt_submit.context_rules[2].enabled,
        Some(false)
    );
}

#[test]
fn test_user_prompt_submit_config_optional() {
    let yaml = r#"
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config without userPromptSubmit should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.user_prompt_submit.context_rules.len(), 0);
}

#[test]
fn test_context_injection_rule_parsing_all_fields() {
    let yaml = r#"
userPromptSubmit:
  contextRules:
  - pattern: "sidebar"
    prompt: "Read the sidebar docs"
    enabled: true
    caseInsensitive: false
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with all fields should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let rule = &config.user_prompt_submit.context_rules[0];
    assert_eq!(rule.pattern, "sidebar");
    assert_eq!(rule.prompt, "Read the sidebar docs");
    assert_eq!(rule.enabled, Some(true));
    assert_eq!(rule.case_insensitive, Some(false));
}

#[test]
fn test_context_injection_rule_parsing_minimal_fields() {
    let yaml = r#"
userPromptSubmit:
  contextRules:
  - pattern: "auth"
    prompt: "Check auth docs"
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with minimal fields should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let rule = &config.user_prompt_submit.context_rules[0];
    assert_eq!(rule.pattern, "auth");
    assert_eq!(rule.prompt, "Check auth docs");
    // Check default values
    assert_eq!(rule.enabled, Some(true), "Default enabled should be true");
    assert_eq!(
        rule.case_insensitive, None,
        "Default caseInsensitive should be None (false)"
    );
}

#[test]
fn test_context_injection_rule_default_values() {
    let yaml = r#"
userPromptSubmit:
  contextRules:
  - pattern: "test"
    prompt: "Test prompt"
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
    let rule = &config.user_prompt_submit.context_rules[0];

    // Verify default for enabled is Some(true)
    assert_eq!(rule.enabled, Some(true));

    // Verify default for caseInsensitive is None (which means false)
    assert_eq!(rule.case_insensitive, None);
}

// Tests for ToolUsageRule deserialization with agent field
#[test]
fn test_tool_usage_rule_deserialization_without_agent() {
    // Test that a ToolUsageRule without the agent field deserializes correctly (agent defaults to None)
    let yaml = r#"
preToolUse:
  toolUsageValidation:
    - tool: "Bash"
      pattern: ""
      action: "block"
      commandPattern: "rm -rf *"
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
stop:
  commands: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with ToolUsageRule without agent should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.pre_tool_use.tool_usage_validation.len(), 1);

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    assert_eq!(rule.tool, "Bash");
    assert_eq!(rule.pattern, "");
    assert_eq!(rule.action, "block");
    assert_eq!(rule.command_pattern, Some("rm -rf *".to_string()));
    assert_eq!(
        rule.agent, None,
        "agent should default to None when not specified"
    );
}

#[test]
fn test_tool_usage_rule_deserialization_with_agent() {
    // Test that a ToolUsageRule with an agent field deserializes correctly
    let yaml = r#"
preToolUse:
  toolUsageValidation:
    - tool: "Bash"
      pattern: ""
      action: "block"
      commandPattern: "rm -rf *"
      agent: "coder"
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
stop:
  commands: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with ToolUsageRule with agent should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.pre_tool_use.tool_usage_validation.len(), 1);

    let rule = &config.pre_tool_use.tool_usage_validation[0];
    assert_eq!(rule.tool, "Bash");
    assert_eq!(rule.pattern, "");
    assert_eq!(rule.action, "block");
    assert_eq!(rule.command_pattern, Some("rm -rf *".to_string()));
    assert_eq!(
        rule.agent,
        Some("coder".to_string()),
        "agent should be 'coder'"
    );
}

#[test]
fn test_tool_usage_rule_deserialization_with_agent_pattern() {
    // Test that a ToolUsageRule with a glob agent pattern (e.g., "code*") deserializes correctly
    let yaml = r#"
preToolUse:
  toolUsageValidation:
    - tool: "Write"
      pattern: "**/*.rs"
      action: "block"
      agent: "test*"
      message: "Test agents cannot modify Rust files"
    - tool: "Bash"
      pattern: ""
      action: "allow"
      commandPattern: "cargo test"
      agent: "code*"
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
stop:
  commands: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with ToolUsageRule with agent glob patterns should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.pre_tool_use.tool_usage_validation.len(), 2);

    // First rule: glob pattern "test*"
    let rule1 = &config.pre_tool_use.tool_usage_validation[0];
    assert_eq!(rule1.tool, "Write");
    assert_eq!(rule1.pattern, "**/*.rs");
    assert_eq!(rule1.action, "block");
    assert_eq!(
        rule1.agent,
        Some("test*".to_string()),
        "agent should be 'test*' glob pattern"
    );
    assert_eq!(
        rule1.message,
        Some("Test agents cannot modify Rust files".to_string())
    );

    // Second rule: glob pattern "code*"
    let rule2 = &config.pre_tool_use.tool_usage_validation[1];
    assert_eq!(rule2.tool, "Bash");
    assert_eq!(rule2.pattern, "");
    assert_eq!(rule2.action, "allow");
    assert_eq!(rule2.command_pattern, Some("cargo test".to_string()));
    assert_eq!(
        rule2.agent,
        Some("code*".to_string()),
        "agent should be 'code*' glob pattern"
    );
}

// Tests for notifyPerCommand field parsing and validation
#[test]
fn test_notify_per_command_valid_true() {
    let yaml = r#"
stop:
  commands:
  - run: "npm test"
    notifyPerCommand: true
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with notifyPerCommand: true should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 1);
    assert_eq!(config.stop.commands[0].notify_per_command, Some(true));
}

#[test]
fn test_notify_per_command_valid_false() {
    let yaml = r#"
stop:
  commands:
  - run: "npm test"
    notifyPerCommand: false
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with notifyPerCommand: false should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 1);
    assert_eq!(config.stop.commands[0].notify_per_command, Some(false));
}

#[test]
fn test_notify_per_command_omitted_defaults_to_none() {
    let yaml = r#"
stop:
  commands:
  - run: "npm test"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config without notifyPerCommand should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 1);
    assert_eq!(
        config.stop.commands[0].notify_per_command, None,
        "notifyPerCommand should default to None when omitted"
    );
}

#[test]
fn test_notify_per_command_invalid_string_value() {
    let yaml = r#"
stop:
  commands:
  - run: "npm test"
    notifyPerCommand: "yes"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Config with notifyPerCommand as string should fail parsing"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("invalid type") || error.contains("notifyPerCommand"),
        "Error should mention type mismatch: {}",
        error
    );
}

#[test]
fn test_notify_per_command_subagent_stop_valid_true() {
    let yaml = r#"
subagentStop:
  commands:
    "*":
      - run: "npm run lint"
        notifyPerCommand: true
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with subagent notifyPerCommand: true should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let cmds = config.subagent_stop.commands.get("*").unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].notify_per_command, Some(true));
}

#[test]
fn test_notify_per_command_subagent_stop_valid_false() {
    let yaml = r#"
subagentStop:
  commands:
    "*":
      - run: "npm run lint"
        notifyPerCommand: false
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with subagent notifyPerCommand: false should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let cmds = config.subagent_stop.commands.get("*").unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].notify_per_command, Some(false));
}

#[test]
fn test_notify_per_command_subagent_stop_omitted() {
    let yaml = r#"
subagentStop:
  commands:
    "*":
      - run: "npm run lint"
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config without subagent notifyPerCommand should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let cmds = config.subagent_stop.commands.get("*").unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(
        cmds[0].notify_per_command, None,
        "notifyPerCommand should default to None when omitted"
    );
}

#[test]
fn test_notify_per_command_mixed_configuration() {
    let yaml = r#"
stop:
  commands:
  - run: "npm test"
    notifyPerCommand: true
  - run: "npm run build"
    notifyPerCommand: false
  - run: "git status"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with mixed notifyPerCommand values should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.stop.commands.len(), 3);
    assert_eq!(
        config.stop.commands[0].notify_per_command,
        Some(true),
        "First command should have notifyPerCommand: true"
    );
    assert_eq!(
        config.stop.commands[1].notify_per_command,
        Some(false),
        "Second command should have notifyPerCommand: false"
    );
    assert_eq!(
        config.stop.commands[2].notify_per_command, None,
        "Third command should have notifyPerCommand: None (omitted)"
    );
}

// ========== RgConfig Validation Tests ==========

#[test]
fn test_rg_config_constraint_mutual_exclusivity() {
    // Test that having multiple constraints (max, min, equal) fails validation
    let yaml = r#"
stop:
  commands:
  - rg:
      pattern: "TODO"
      files: "**/*.rs"
      max: 10
      min: 1
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Config with multiple constraints should fail validation"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("Only one of 'max', 'min', or 'equal'"),
        "Error should mention mutual exclusivity: {}",
        error
    );
}

#[test]
fn test_rg_config_default_constraint_is_max_zero() {
    // Test that when no constraint is specified, default is max: 0
    let yaml = r#"
stop:
  commands:
  - rg:
      pattern: "TODO"
      files: "**/*.rs"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with no constraint should parse: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let rg = config.stop.commands[0].rg.as_ref().unwrap();
    assert_eq!(rg.max, None);
    assert_eq!(rg.min, None);
    assert_eq!(rg.equal, None);

    // Verify constraint evaluates to Max(0)
    use crate::rg_search::Constraint;
    let constraint = Constraint::from_config(rg);
    match constraint {
        Constraint::Max(n) => assert_eq!(n, 0, "Default should be Max(0)"),
        _ => panic!("Default constraint should be Max(0)"),
    }
}

#[test]
fn test_rg_config_invalid_regex_fails_validation() {
    // Test that invalid regex pattern fails validation
    let yaml = r#"
stop:
  commands:
  - rg:
      pattern: "[invalid"
      files: "**/*.rs"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Config with invalid regex should fail validation"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("Invalid regex pattern"),
        "Error should mention invalid regex: {}",
        error
    );
}

#[test]
fn test_rg_config_fixed_strings_skips_regex_validation() {
    // Test that with fixedStrings: true, invalid regex-like patterns are allowed
    let yaml = r#"
stop:
  commands:
  - rg:
      pattern: "[this would be invalid regex"
      files: "**/*.rs"
      fixedStrings: true
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with fixedStrings should allow invalid regex: {:?}",
        result.err()
    );
}

#[test]
fn test_stop_command_run_and_rg_mutual_exclusivity() {
    // Test that having both run and rg fails validation
    let yaml = r#"
stop:
  commands:
  - run: "echo test"
    rg:
      pattern: "TODO"
      files: "**/*.rs"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Config with both run and rg should fail validation"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("mutually exclusive"),
        "Error should mention mutual exclusivity: {}",
        error
    );
}

#[test]
fn test_stop_command_neither_run_nor_rg_fails() {
    // Test that having neither run nor rg fails validation
    let yaml = r#"
stop:
  commands:
  - message: "No command specified"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Config with neither run nor rg should fail validation"
    );
    let error = result.err().unwrap().to_string();
    assert!(
        error.contains("must have either 'run' or 'rg'"),
        "Error should mention missing command: {}",
        error
    );
}

#[test]
fn test_stop_command_rg_only_is_valid() {
    // Test that having only rg field is valid
    let yaml = r#"
stop:
  commands:
  - rg:
      pattern: "TODO"
      files: "**/*.rs"
      max: 5
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_ok(),
        "Config with only rg should be valid: {:?}",
        result.err()
    );
}

#[test]
fn test_subagent_stop_command_mutual_exclusivity() {
    // Test that subagent stop commands have same mutual exclusivity rules
    let yaml = r#"
subagentStop:
  commands:
    "*":
      - run: "echo test"
        rg:
          pattern: "TODO"
          files: "**/*.rs"
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(
        result.is_err(),
        "Subagent config with both run and rg should fail validation"
    );
}

// ========== RgConfig Parsing Tests ==========

#[test]
fn test_rg_config_parses_all_fields() {
    // Test parsing a complete RgConfig from YAML
    let yaml = r#"
stop:
  commands:
  - rg:
      pattern: "TODO"
      files: "**/*.rs"
      max: 10
      ignoreCase: true
      smartCase: false
      word: true
      fixedStrings: false
      multiLine: true
      wholeLine: false
      dotMatchesNewLine: true
      unicode: true
      maxDepth: 5
      hidden: false
      followLinks: true
      maxFilesize: 1000000
      gitIgnore: true
      rgIgnore: true
      parents: true
      sameFileSystem: false
      threads: 4
      types:
        - rust
        - js
      context: 2
      countMode: lines
      invertMatch: false
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(result.is_ok(), "Full RgConfig should parse: {:?}", result.err());

    let config = result.unwrap();
    let rg = config.stop.commands[0].rg.as_ref().unwrap();

    // Verify all fields
    assert_eq!(rg.pattern, "TODO");
    assert_eq!(rg.files, "**/*.rs");
    assert_eq!(rg.max, Some(10));
    assert_eq!(rg.ignore_case, true);
    assert_eq!(rg.smart_case, false);
    assert_eq!(rg.word, true);
    assert_eq!(rg.fixed_strings, false);
    assert_eq!(rg.multi_line, true);
    assert_eq!(rg.whole_line, false);
    assert_eq!(rg.dot_matches_new_line, true);
    assert_eq!(rg.unicode, true);
    assert_eq!(rg.max_depth, Some(5));
    assert_eq!(rg.hidden, false);
    assert_eq!(rg.follow_links, true);
    assert_eq!(rg.max_filesize, Some(1000000));
    assert_eq!(rg.git_ignore, true);
    assert_eq!(rg.rg_ignore, true);
    assert_eq!(rg.parents, true);
    assert_eq!(rg.same_file_system, false);
    assert_eq!(rg.threads, Some(4));
    assert_eq!(rg.types, vec!["rust", "js"]);
    assert_eq!(rg.context, 2);
    assert_eq!(rg.invert_match, false);
}

#[test]
fn test_rg_config_defaults() {
    // Test that default values are correct
    let yaml = r#"
stop:
  commands:
  - rg:
      pattern: "TODO"
      files: "**/*.rs"
preToolUse:
  preventAdditions: []
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
  "#;
    let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
    assert!(result.is_ok(), "Minimal RgConfig should parse: {:?}", result.err());

    let config = result.unwrap();
    let rg = config.stop.commands[0].rg.as_ref().unwrap();

    // Verify defaults
    assert_eq!(rg.ignore_case, false);
    assert_eq!(rg.smart_case, false);
    assert_eq!(rg.word, false);
    assert_eq!(rg.fixed_strings, false);
    assert_eq!(rg.multi_line, false);
    assert_eq!(rg.whole_line, false);
    assert_eq!(rg.dot_matches_new_line, false);
    assert_eq!(rg.unicode, true); // default_true
    assert_eq!(rg.max_depth, None);
    assert_eq!(rg.hidden, false);
    assert_eq!(rg.follow_links, false);
    assert_eq!(rg.max_filesize, None);
    assert_eq!(rg.git_ignore, true); // default_true
    assert_eq!(rg.rg_ignore, true); // default_true
    assert_eq!(rg.parents, true); // default_true
    assert_eq!(rg.same_file_system, false);
    assert_eq!(rg.threads, None);
    assert_eq!(rg.types.len(), 0);
    assert_eq!(rg.context, 0);
    assert_eq!(rg.invert_match, false);
}
