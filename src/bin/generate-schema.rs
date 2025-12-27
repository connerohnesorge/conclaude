use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Generate the schema
    let schema = conclaude::schema::generate_config_schema()
        .map_err(|e| anyhow::anyhow!("Failed to generate schema: {}", e))?;

    // Write to workspace root
    let output_path = PathBuf::from("conclaude-schema.json");
    conclaude::schema::write_schema_to_file(&schema, &output_path)
        .map_err(|e| anyhow::anyhow!("Failed to write schema file: {}", e))?;

    // Success message matching the UX of the removed subcommand
    println!(
        "[OK] Schema generated successfully: {}",
        output_path.display()
    );
    println!("   The schema file is ready to be published as a release asset.");

    Ok(())
}
