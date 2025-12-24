//! Schema generation and validation for Conclaude configuration files.
//!
//! This module provides utilities for generating JSON Schema definitions from the
//! `ConclaudeConfig` structure, writing schemas to files, and validating YAML
//! configuration files against the schema.
//!
//! # Primary Use Cases
//!
//! 1. **Build/Release Automation**: The `generate-schema` binary uses these functions
//!    to create schema files during the build process
//! 2. **GitHub Releases**: Generated schemas are automatically uploaded to GitHub
//!    releases for public consumption
//! 3. **IDE Integration**: YAML language server headers reference the published schema
//!    for autocompletion and validation in editors
//!
//! # Schema Distribution
//!
//! The generated schema is published at:
//! `https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-schema.json`
//!
//! This URL is embedded in configuration files via the `yaml-language-server` comment
//! header, enabling IDE support for `.conclaude.yml` files.
//!
//! # Examples
//!
//! ```rust
//! use conclaude::schema::{generate_config_schema, write_schema_to_file};
//! use std::path::PathBuf;
//!
//! // Generate a schema
//! let schema = generate_config_schema().unwrap();
//!
//! // Write to file
//! let path = PathBuf::from("schema.json");
//! write_schema_to_file(&schema, &path).unwrap();
//! ```

use anyhow::{Context, Result};
use schemars::schema_for;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::config::ConclaudeConfig;

/// Generates a JSON Schema for the `ConclaudeConfig` structure
///
/// # Errors
///
/// Returns an error if JSON schema generation or value serialization fails.
#[allow(dead_code)]
pub fn generate_config_schema() -> Result<Value> {
    let schema = schema_for!(ConclaudeConfig);

    // Enhance the schema with additional metadata
    let mut schema_value = serde_json::to_value(schema)?;

    // Add metadata to the root schema
    if let Value::Object(ref mut schema_obj) = schema_value {
        schema_obj.insert(
            "$schema".to_string(),
            json!("http://json-schema.org/draft-07/schema#"),
        );
        schema_obj.insert("title".to_string(), json!("Conclaude Configuration"));
        schema_obj.insert(
            "description".to_string(),
            json!("Configuration schema for Conclaude - Claude Code hook handler"),
        );
        schema_obj.insert("$id".to_string(), json!("https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-schema.json"));
    }

    Ok(schema_value)
}

/// Writes the generated schema to a file
///
/// # Errors
///
/// Returns an error if JSON serialization fails, directory creation fails, or file writing fails.
#[allow(dead_code)]
pub fn write_schema_to_file(schema: &Value, output_path: &PathBuf) -> Result<()> {
    let schema_json =
        serde_json::to_string_pretty(schema).context("Failed to serialize schema to JSON")?;

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    fs::write(output_path, schema_json)
        .with_context(|| format!("Failed to write schema to: {}", output_path.display()))?;

    Ok(())
}

/// Validates that a given YAML content matches the schema
///
/// # Errors
///
/// Returns an error if the YAML content is invalid or does not match the expected structure.
#[cfg(test)]
pub fn validate_config_against_schema(config_content: &str) -> Result<()> {
    // Parse the YAML to ensure it's valid
    let _: ConclaudeConfig = serde_yaml::from_str(config_content).map_err(|e| {
        let base_error = e.to_string();
        let mut parts = vec![
            "Configuration validation failed".to_string(),
            String::new(),
            format!("Error: {}", base_error),
        ];

        // Add specific guidance based on error type
        if base_error.contains("unknown field") {
            parts.push(String::new());
            parts.push("The configuration contains an unknown field.".to_string());
            parts.push("Check the field name for typos or incorrect casing.".to_string());
        } else if base_error.contains("invalid type") {
            parts.push(String::new());
            parts.push("A field has the wrong type (e.g., string instead of boolean).".to_string());
        } else if base_error.contains("expected") || base_error.contains("while parsing") {
            parts.push(String::new());
            parts.push("YAML syntax error detected. Check indentation and formatting.".to_string());
        }

        anyhow::anyhow!(parts.join("\n"))
    })?;

    Ok(())
}

/// Gets the default schema URL for YAML language server headers
#[must_use]
pub fn get_schema_url() -> String {
    "https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-schema.json"
        .to_string()
}

/// Generates a YAML language server header comment with schema URL
#[must_use]
pub fn generate_yaml_language_server_header(custom_schema_url: Option<&str>) -> String {
    let default_url = get_schema_url();
    let schema_url = custom_schema_url.unwrap_or(&default_url);
    format!("# yaml-language-server: $schema={schema_url}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_against_schema() {
        let valid_config = r#"
stop:
  commands:
    - run: "echo test"
  infinite: false
preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
"#;

        validate_config_against_schema(valid_config).unwrap();

        let invalid_config = r#"
invalid_field: "should fail"
"#;

        assert!(validate_config_against_schema(invalid_config).is_err());
    }
}
