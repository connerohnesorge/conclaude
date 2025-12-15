// Export modules for testing
pub mod config;
pub mod gitignore;
pub mod hooks;

#[cfg(test)]
mod hooks_tests;

/// Schema generation and validation utilities.
///
/// This module is primarily used by build and release automation tooling, including:
/// - The external `generate-schema` binary (`src/bin/generate-schema.rs`) which creates
///   the JSON Schema file during builds
/// - GitHub Actions workflows that upload schema files to releases
/// - External scripts that need to programmatically generate or validate configuration schemas
///
/// The generated schema files are automatically uploaded to GitHub releases and referenced
/// in YAML language server headers for configuration file autocompletion and validation.
///
/// # Examples
///
/// ```rust
/// use conclaude::schema::{generate_config_schema, write_schema_to_file};
/// use std::path::PathBuf;
///
/// // Generate the schema
/// let schema = generate_config_schema().unwrap();
///
/// // Write to file
/// let output_path = PathBuf::from("conclaude-schema.json");
/// write_schema_to_file(&schema, &output_path).unwrap();
/// ```
pub mod schema;
pub mod types;
