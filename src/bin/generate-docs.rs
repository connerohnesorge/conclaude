use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Command-line arguments for the documentation generator
#[derive(Debug)]
struct Args {
    output_dir: PathBuf,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut output_dir = PathBuf::from("docs/src/content/docs/reference/config");

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--output" => {
                    if let Some(path) = args.next() {
                        output_dir = PathBuf::from(path);
                    } else {
                        eprintln!("Error: --output requires a path argument");
                        std::process::exit(1);
                    }
                }
                "--help" | "-h" => {
                    println!("Usage: generate-docs [OPTIONS]");
                    println!();
                    println!("Options:");
                    println!("  --output <DIR>  Output directory for generated docs (default: docs/src/content/docs/reference/config)");
                    println!("  --help, -h      Show this help message");
                    std::process::exit(0);
                }
                other => {
                    eprintln!("Error: Unknown argument '{}'", other);
                    eprintln!("Use --help for usage information");
                    std::process::exit(1);
                }
            }
        }

        Self { output_dir }
    }
}

/// Schema definition extracted from JSON Schema
#[derive(Debug)]
struct Schema {
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    description: String,
    properties: HashMap<String, PropertyDefinition>,
    definitions: HashMap<String, DefinitionSchema>,
}

/// Property definition for a schema property
#[derive(Debug)]
struct PropertyDefinition {
    #[allow(dead_code)]
    name: String,
    description: String,
    property_type: String,
    default_value: Option<String>,
    reference: Option<String>,
}

/// Definition schema for a schema definition
#[derive(Debug)]
struct DefinitionSchema {
    name: String,
    description: String,
    properties: HashMap<String, PropertyDefinition>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Conclaude Documentation Generator");
    println!("==================================");
    println!();

    // Read the schema file
    println!("Reading schema from conclaude-schema.json...");
    let schema_path = "conclaude-schema.json";
    let schema_content = fs::read_to_string(schema_path)
        .context(format!("Failed to read schema file: {}", schema_path))?;

    // Parse as JSON
    println!("Parsing JSON schema...");
    let schema_json: Value =
        serde_json::from_str(&schema_content).context("Failed to parse schema JSON")?;

    // Extract schema information
    let schema = extract_schema(&schema_json)?;

    // Ensure output directory exists
    println!("Creating output directory: {}", args.output_dir.display());
    fs::create_dir_all(&args.output_dir).context(format!(
        "Failed to create output directory: {}",
        args.output_dir.display()
    ))?;

    // Generate overview page
    println!("Generating configuration overview page...");
    generate_overview_page(&schema, &args.output_dir)?;

    // Generate section pages (sorted for deterministic output)
    let mut section_names: Vec<_> = schema.properties.keys().collect();
    section_names.sort();

    for section_name in section_names {
        println!("Generating documentation for section: {}", section_name);
        generate_section_page(&schema, section_name, &args.output_dir, &schema_json)?;
    }

    println!();
    println!("Documentation generation complete!");
    println!("Generated files in: {}", args.output_dir.display());

    Ok(())
}

/// Extract schema information from JSON
fn extract_schema(schema_json: &Value) -> Result<Schema> {
    let title = schema_json
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Configuration Schema")
        .to_string();

    let description = schema_json
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let properties = extract_properties(
        schema_json
            .get("properties")
            .context("Schema missing 'properties'")?,
    )?;

    let definitions = extract_definitions(
        schema_json
            .get("definitions")
            .context("Schema missing 'definitions'")?,
    )?;

    Ok(Schema {
        title,
        description,
        properties,
        definitions,
    })
}

/// Extract properties from schema JSON
fn extract_properties(properties_json: &Value) -> Result<HashMap<String, PropertyDefinition>> {
    let mut properties = HashMap::new();

    if let Some(obj) = properties_json.as_object() {
        for (name, prop) in obj {
            let description = prop
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let property_type = extract_type(prop);
            let default_value = extract_default(prop);
            let reference = extract_reference(prop);

            properties.insert(
                name.clone(),
                PropertyDefinition {
                    name: name.clone(),
                    description,
                    property_type,
                    default_value,
                    reference,
                },
            );
        }
    }

    Ok(properties)
}

/// Extract definitions from schema JSON
fn extract_definitions(definitions_json: &Value) -> Result<HashMap<String, DefinitionSchema>> {
    let mut definitions = HashMap::new();

    if let Some(obj) = definitions_json.as_object() {
        for (name, def) in obj {
            let description = def
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let properties = if let Some(props) = def.get("properties") {
                extract_properties(props)?
            } else {
                HashMap::new()
            };

            definitions.insert(
                name.clone(),
                DefinitionSchema {
                    name: name.clone(),
                    description,
                    properties,
                },
            );
        }
    }

    Ok(definitions)
}

/// Extract type information from property JSON
fn extract_type(prop: &Value) -> String {
    if let Some(type_val) = prop.get("type") {
        if let Some(type_str) = type_val.as_str() {
            return type_str.to_string();
        } else if let Some(type_arr) = type_val.as_array() {
            let types: Vec<String> = type_arr
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            return types.join(" | ");
        }
    }

    if prop.get("$ref").is_some() {
        return "reference".to_string();
    }

    "unknown".to_string()
}

/// Extract nested type reference from a property
/// Returns the definition name if this property references another type
fn extract_nested_type_ref(prop: &Value) -> Option<String> {
    // Check for direct $ref
    if let Some(ref_str) = prop.get("$ref").and_then(|v| v.as_str()) {
        if let Some(def_name) = ref_str.strip_prefix("#/definitions/") {
            return Some(def_name.to_string());
        }
    }

    // Check for array items with $ref
    if let Some(items) = prop.get("items") {
        if let Some(ref_str) = items.get("$ref").and_then(|v| v.as_str()) {
            if let Some(def_name) = ref_str.strip_prefix("#/definitions/") {
                return Some(def_name.to_string());
            }
        }
    }

    // Check for additionalProperties with items.$ref (for HashMap<String, Vec<T>>)
    if let Some(additional) = prop.get("additionalProperties") {
        if let Some(items) = additional.get("items") {
            if let Some(ref_str) = items.get("$ref").and_then(|v| v.as_str()) {
                if let Some(def_name) = ref_str.strip_prefix("#/definitions/") {
                    return Some(def_name.to_string());
                }
            }
        }
    }

    None
}

/// Extract default value from property JSON
fn extract_default(prop: &Value) -> Option<String> {
    prop.get("default").map(|v| {
        if v.is_null() {
            "null".to_string()
        } else if v.is_string() {
            format!("\"{}\"", v.as_str().unwrap_or(""))
        } else if let Some(b) = v.as_bool() {
            b.to_string()
        } else if v.is_number() {
            v.to_string()
        } else if v.is_array() {
            "[]".to_string()
        } else if v.is_object() {
            "{}".to_string()
        } else {
            v.to_string()
        }
    })
}

/// Extract reference from property JSON
fn extract_reference(prop: &Value) -> Option<String> {
    // Check direct $ref
    if let Some(ref_val) = prop.get("$ref") {
        if let Some(ref_str) = ref_val.as_str() {
            return Some(ref_str.to_string());
        }
    }

    // Check allOf for references
    if let Some(all_of) = prop.get("allOf") {
        if let Some(arr) = all_of.as_array() {
            for item in arr {
                if let Some(ref_val) = item.get("$ref") {
                    if let Some(ref_str) = ref_val.as_str() {
                        return Some(ref_str.to_string());
                    }
                }
            }
        }
    }

    // Check anyOf for references
    if let Some(any_of) = prop.get("anyOf") {
        if let Some(arr) = any_of.as_array() {
            for item in arr {
                if let Some(ref_val) = item.get("$ref") {
                    if let Some(ref_str) = ref_val.as_str() {
                        return Some(ref_str.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Generate Starlight frontmatter for a page
fn generate_frontmatter(title: &str, description: &str) -> String {
    format!(
        r#"---
title: {}
description: {}
---

"#,
        title, description
    )
}

/// Generate the configuration overview page
fn generate_overview_page(schema: &Schema, output_dir: &Path) -> Result<()> {
    let mut content = String::new();

    // Add frontmatter
    content.push_str(&generate_frontmatter(
        "Configuration Reference",
        "Complete reference for Conclaude configuration options",
    ));

    // Add overview
    content.push_str("# Configuration Reference\n\n");

    // Add introduction paragraph
    content.push_str("Conclaude uses YAML configuration files to define lifecycle hooks, file protection rules, and workflow policies for Claude Code sessions. ");
    content.push_str("The configuration system is discovered via cosmiconfig and validated against a JSON Schema to ensure correctness.\n\n");

    // Add schema note
    content.push_str("**IDE Support**: Conclaude provides a JSON Schema for configuration validation and autocomplete in your editor. ");
    content.push_str("The schema is available at `conclaude-schema.json` and can be referenced in your YAML files for enhanced editing support.\n\n");

    // Add quick reference table
    content.push_str("## Quick Reference\n\n");
    content.push_str("| Section | Description | Key Options |\n");
    content.push_str("|---------|-------------|-------------|\n");

    // Sort sections for deterministic output
    let mut section_names: Vec<_> = schema.properties.keys().collect();
    section_names.sort();

    for section_name in &section_names {
        let property = &schema.properties[*section_name];
        let section_title = format_section_title(section_name);
        let kebab_name = to_kebab_case(section_name);

        // Extract first sentence for description
        // Try to get description from the referenced definition first
        let description = if let Some(ref_path) = &property.reference {
            if let Some(def_name) = ref_path.strip_prefix("#/definitions/") {
                if let Some(definition) = schema.definitions.get(def_name) {
                    &definition.description
                } else {
                    &property.description
                }
            } else {
                &property.description
            }
        } else {
            &property.description
        };

        let first_sentence = description
            .lines()
            .next()
            .unwrap_or("")
            .split('.')
            .next()
            .unwrap_or("")
            .trim();

        // Get key options from the definition
        let key_options = if let Some(ref_path) = &property.reference {
            if let Some(def_name) = ref_path.strip_prefix("#/definitions/") {
                if let Some(definition) = schema.definitions.get(def_name) {
                    let mut prop_names: Vec<_> =
                        definition.properties.keys().map(|s| s.as_str()).collect();
                    prop_names.sort();

                    // Take up to 3 key properties for the table
                    let key_props: Vec<_> = prop_names.iter().take(3).collect();
                    if key_props.is_empty() {
                        "-".to_string()
                    } else {
                        format!(
                            "`{}`",
                            key_props
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<_>>()
                                .join("`, `")
                        )
                    }
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        };

        content.push_str(&format!(
            "| [{}](/reference/config/{}/) | {} | {} |\n",
            section_title, kebab_name, first_sentence, key_options
        ));
    }

    content.push('\n');

    // Add configuration sections
    content.push_str("## Configuration Sections\n\n");
    content.push_str("Detailed documentation for each configuration section:\n\n");

    for section_name in &section_names {
        let property = &schema.properties[*section_name];
        let section_title = format_section_title(section_name);
        let kebab_name = to_kebab_case(section_name);
        content.push_str(&format!(
            "### [{}](/reference/config/{})\n\n",
            section_title, kebab_name
        )); // Note: no trailing slash for section headers

        // Add first line of description from definition if available
        let description = if let Some(ref_path) = &property.reference {
            if let Some(def_name) = ref_path.strip_prefix("#/definitions/") {
                if let Some(definition) = schema.definitions.get(def_name) {
                    &definition.description
                } else {
                    &property.description
                }
            } else {
                &property.description
            }
        } else {
            &property.description
        };

        let first_line = description.lines().next().unwrap_or("");
        if !first_line.is_empty() {
            content.push_str(&format!("{}\n\n", first_line));
        }
    }

    // Write to file
    let output_path = output_dir.join("configuration.md");
    fs::write(&output_path, content).context(format!(
        "Failed to write overview page: {}",
        output_path.display()
    ))?;

    Ok(())
}

/// Generate a section-specific documentation page
fn generate_section_page(
    schema: &Schema,
    section_name: &str,
    output_dir: &Path,
    schema_json: &Value,
) -> Result<()> {
    let property = schema
        .properties
        .get(section_name)
        .context(format!("Section not found: {}", section_name))?;

    let mut content = String::new();

    // Add frontmatter
    let section_title = format_section_title(section_name);
    let kebab_name = to_kebab_case(section_name);
    content.push_str(&generate_frontmatter(
        &section_title,
        &format!("Configuration options for {}", section_name),
    ));

    // Add section title
    content.push_str(&format!("# {}\n\n", section_title));

    // Get the definition for detailed information
    let definition = if let Some(ref_path) = &property.reference {
        if let Some(def_name) = ref_path.strip_prefix("#/definitions/") {
            schema.definitions.get(def_name)
        } else {
            None
        }
    } else {
        None
    };

    // Extract and add main description (without examples)
    let description = if let Some(def) = definition {
        &def.description
    } else {
        &property.description
    };

    let main_description = extract_main_description(description);
    if !main_description.is_empty() {
        content.push_str(&format!("{}\n\n", main_description));
    }

    // Track nested types we need to document
    let mut nested_types_to_document = Vec::new();

    // Add properties section if available
    if let Some(definition) = definition {
        if !definition.properties.is_empty() {
            content.push_str("## Configuration Properties\n\n");

            // Sort properties for deterministic output
            let mut prop_names: Vec<_> = definition.properties.keys().collect();
            prop_names.sort();

            // Get the raw property JSON for nested type detection
            let def_json = schema_json
                .get("definitions")
                .and_then(|defs| defs.get(definition.name.as_str()));

            for prop_name in prop_names {
                let prop = &definition.properties[prop_name];
                content.push_str(&format!("### `{}`\n\n", prop_name));

                // Extract main description for the property (before any examples)
                let prop_main_desc = extract_main_description(&prop.description);
                if !prop_main_desc.is_empty() {
                    content.push_str(&format!("{}\n\n", prop_main_desc));
                }

                // Add metadata table
                content.push_str("| Attribute | Value |\n");
                content.push_str("|-----------|-------|\n");
                content.push_str(&format!("| **Type** | `{}` |\n", prop.property_type));

                if let Some(default) = &prop.default_value {
                    content.push_str(&format!("| **Default** | `{}` |\n", default));
                }

                content.push('\n');

                // Check if this property references a nested type
                if let Some(def) = def_json {
                    if let Some(prop_json) = def
                        .get("properties")
                        .and_then(|p| p.get(prop_name.as_str()))
                    {
                        if let Some(nested_type) = extract_nested_type_ref(prop_json) {
                            if !nested_types_to_document.contains(&nested_type) {
                                nested_types_to_document.push(nested_type);
                            }
                        }
                    }
                }

                // Add property-specific examples if they exist
                let prop_examples = extract_yaml_examples(&prop.description);
                if !prop_examples.is_empty() {
                    content.push_str("**Examples:**\n\n");
                    for example in prop_examples {
                        content.push_str("```yaml\n");
                        content.push_str(&example);
                        content.push_str("\n```\n\n");
                    }
                }
            }
        }
    }

    // Add nested type documentation
    if !nested_types_to_document.is_empty() {
        content.push_str("## Nested Types\n\n");
        content.push_str("This section uses the following nested type definitions:\n\n");

        for nested_type in &nested_types_to_document {
            let nested_docs = generate_nested_type_docs(schema, nested_type, schema_json, 3);
            content.push_str(&nested_docs);
        }
    }

    // Add examples section from definition description
    let examples = extract_yaml_examples(description);
    if !examples.is_empty() {
        content.push_str("## Complete Examples\n\n");
        content.push_str(&format!(
            "Here are complete configuration examples for the `{}` section:\n\n",
            section_name
        ));

        for (i, example) in examples.iter().enumerate() {
            if examples.len() > 1 {
                content.push_str(&format!("### Example {}\n\n", i + 1));
            }
            content.push_str("```yaml\n");
            content.push_str(example);
            content.push_str("\n```\n\n");
        }
    }

    // Add navigation back to overview
    content.push_str("## See Also\n\n");
    content.push_str("- [Configuration Overview](/reference/config/configuration/) - Complete reference for all configuration options\n");

    // Write to file with kebab-case filename
    let output_path = output_dir.join(format!("{}.md", kebab_name));
    fs::write(&output_path, content).context(format!(
        "Failed to write section page: {}",
        output_path.display()
    ))?;

    Ok(())
}

/// Format section name into a readable title
fn format_section_title(section_name: &str) -> String {
    // Convert camelCase to Title Case
    let mut result = String::new();
    let mut prev_was_lower = false;

    for (i, ch) in section_name.chars().enumerate() {
        if i == 0 {
            result.push(ch.to_ascii_uppercase());
            prev_was_lower = ch.is_lowercase();
        } else if ch.is_uppercase() && prev_was_lower {
            result.push(' ');
            result.push(ch);
            prev_was_lower = false;
        } else {
            result.push(ch);
            prev_was_lower = ch.is_lowercase();
        }
    }

    result
}

/// Convert camelCase to kebab-case for filenames
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lower = false;

    for (i, ch) in s.chars().enumerate() {
        if i == 0 {
            result.push(ch.to_ascii_lowercase());
            prev_was_lower = ch.is_lowercase();
        } else if ch.is_uppercase() && prev_was_lower {
            result.push('-');
            result.push(ch.to_ascii_lowercase());
            prev_was_lower = false;
        } else {
            result.push(ch.to_ascii_lowercase());
            prev_was_lower = ch.is_lowercase();
        }
    }

    result
}

/// Extract YAML examples from schema description
fn extract_yaml_examples(description: &str) -> Vec<String> {
    let mut examples = Vec::new();

    // Handle case where YAML examples are on single lines (inline)
    // Pattern: ```yaml ... ```
    let mut search_start = 0;
    while let Some(yaml_start) = description[search_start..].find("```yaml") {
        let absolute_start = search_start + yaml_start;
        let after_marker = absolute_start + "```yaml".len();

        // Find the closing ```
        if let Some(yaml_end) = description[after_marker..].find("```") {
            let absolute_end = after_marker + yaml_end;
            let example_content = &description[after_marker..absolute_end];

            // Clean up the example content
            let cleaned = example_content.trim();
            if !cleaned.is_empty() {
                examples.push(cleaned.to_string());
            }

            // Continue searching after this example
            search_start = absolute_end + 3; // Skip past the closing ```
        } else {
            break;
        }
    }

    examples
}

/// Extract main description text (before examples section)
fn extract_main_description(description: &str) -> String {
    // Split on "# Examples" or similar headers
    let parts: Vec<&str> = description.split("\n# Examples").collect();
    parts[0].trim().to_string()
}

/// Generate documentation for a nested type definition
fn generate_nested_type_docs(
    schema: &Schema,
    type_name: &str,
    schema_json: &Value,
    indent_level: usize,
) -> String {
    let mut content = String::new();

    // Find the definition
    if let Some(definition) = schema.definitions.get(type_name) {
        // Add heading based on indent level
        let heading = "#".repeat(indent_level);
        content.push_str(&format!("{} `{}` Type\n\n", heading, type_name));

        // Add description
        let main_desc = extract_main_description(&definition.description);
        if !main_desc.is_empty() {
            content.push_str(&format!("{}\n\n", main_desc));
        }

        // Check if this is an enum type (anyOf)
        let def_json = schema_json
            .get("definitions")
            .and_then(|defs| defs.get(type_name));

        if let Some(def) = def_json {
            if let Some(any_of) = def.get("anyOf") {
                // This is an enum/union type - document variants
                content.push_str("**Variants:**\n\n");

                if let Some(variants) = any_of.as_array() {
                    for (i, variant) in variants.iter().enumerate() {
                        let variant_desc = variant
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        if let Some(variant_type) = variant.get("type").and_then(|v| v.as_str()) {
                            content.push_str(&format!(
                                "{}. **{}**: {}\n",
                                i + 1,
                                variant_type,
                                variant_desc
                            ));

                            // If it's an object variant, show its properties
                            if variant_type == "object" {
                                if let Some(props) = variant.get("properties") {
                                    if let Some(props_obj) = props.as_object() {
                                        content.push_str("\n   Properties:\n");

                                        let mut prop_names: Vec<_> = props_obj.keys().collect();
                                        prop_names.sort();

                                        for prop_name in prop_names {
                                            if let Some(prop_val) = props_obj.get(prop_name) {
                                                let prop_desc = prop_val
                                                    .get("description")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("");
                                                let prop_type = extract_type(prop_val);
                                                content.push_str(&format!(
                                                    "   - `{}` ({}): {}\n",
                                                    prop_name, prop_type, prop_desc
                                                ));
                                            }
                                        }
                                        content.push('\n');
                                    }
                                }
                            }
                        }
                    }
                }
                content.push('\n');
            } else if !definition.properties.is_empty() {
                // Regular object type - show properties table
                content.push_str("**Properties:**\n\n");
                content.push_str("| Property | Type | Default | Description |\n");
                content.push_str("|----------|------|---------|-------------|\n");

                // Sort properties for deterministic output
                let mut prop_names: Vec<_> = definition.properties.keys().collect();
                prop_names.sort();

                for prop_name in prop_names {
                    let prop = &definition.properties[prop_name];
                    let default_str = prop
                        .default_value
                        .as_ref()
                        .map(|d| format!("`{}`", d))
                        .unwrap_or_else(|| "-".to_string());

                    // Truncate description for table (take first sentence)
                    let short_desc = prop
                        .description
                        .lines()
                        .next()
                        .unwrap_or("")
                        .split('.')
                        .next()
                        .unwrap_or("")
                        .trim();

                    content.push_str(&format!(
                        "| `{}` | `{}` | {} | {} |\n",
                        prop_name, prop.property_type, default_str, short_desc
                    ));
                }
                content.push('\n');
            }
        }
    }

    content
}
