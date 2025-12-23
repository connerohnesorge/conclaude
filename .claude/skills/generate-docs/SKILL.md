---
name: generate-docs
description: |
  Generate configuration reference documentation from conclaude-schema.json using src/bin/generate-docs.rs.
  USE WHEN the schema file changes, configuration options are added/modified, or documentation needs to be regenerated for the website.
---

# Generate Documentation

This skill helps you generate the configuration reference documentation for conclaude. The documentation is generated from `conclaude-schema.json` and output as Markdown files ready for the Starlight-based documentation site.

## USE WHEN

**Use this skill when:**

- The `conclaude-schema.json` file has been updated
- New configuration sections or properties have been added
- Configuration descriptions or examples have been modified in the schema
- Documentation is out of sync with the configuration schema
- Preparing documentation updates for a release
- Setting up or rebuilding the documentation site
- Schema descriptions, types, or default values have changed
- Adding new configuration examples to the schema

**Do NOT use this skill when:**

- Just viewing or reading the existing documentation
- Making changes unrelated to configuration
- Working on non-documentation features
- The schema hasn't changed since last doc generation

## How It Works

The documentation generator (`src/bin/generate-docs.rs`) processes the JSON Schema file to create structured documentation:

1. **Reads** `conclaude-schema.json` from the workspace root
2. **Parses** the schema to extract:
   - Configuration sections and their properties
   - Type information and default values
   - Descriptions and documentation strings
   - YAML examples embedded in descriptions
   - Nested type definitions
3. **Generates** Markdown files with Starlight frontmatter:
   - Overview page with quick reference table
   - Individual section pages with detailed properties
   - Nested type documentation
   - Complete configuration examples
4. **Outputs** to `docs/src/content/docs/reference/config/` by default

The generated documentation integrates seamlessly with the Starlight documentation site.

## Instructions

### Step 1: Ensure Schema is Up-to-Date

First, make sure your schema file reflects the current configuration:

```bash
# If you've made config changes, regenerate schema first
cargo run --bin generate-schema
```

### Step 2: Generate Documentation

Run the documentation generator:

```bash
cargo run --bin generate-docs
```

This will:
- Read the schema from `conclaude-schema.json`
- Generate an overview page at `docs/src/content/docs/reference/config/configuration.md`
- Create individual section pages (e.g., `hooks.md`, `files.md`, etc.)
- Extract and format YAML examples from schema descriptions
- Document nested types used by configuration properties

### Step 3: Custom Output Directory (Optional)

To generate docs to a different location:

```bash
cargo run --bin generate-docs -- --output path/to/output/dir
```

### Step 4: Verify Generated Documentation

Check the generated files:

```bash
ls -l docs/src/content/docs/reference/config/
cat docs/src/content/docs/reference/config/configuration.md
```

Look for:
- Overview page with all configuration sections listed
- Individual section pages with property tables
- Properly formatted YAML examples
- Nested type definitions where applicable
- Starlight frontmatter with title and description

### Step 5: Review in the Documentation Site

If you have the docs site running locally:

```bash
cd docs
npm run dev
```

Navigate to `/reference/config/configuration` to review the generated documentation in context.

### Step 6: Commit the Documentation

Include the generated docs in your commit:

```bash
git add docs/src/content/docs/reference/config/
git commit -m "Update configuration reference documentation"
```

## Expected Output

When successful, you'll see:

```
Conclaude Documentation Generator
==================================

Reading schema from conclaude-schema.json...
Parsing JSON schema...
Creating output directory: docs/src/content/docs/reference/config
Generating configuration overview page...
Generating documentation for section: files
Generating documentation for section: hooks
Generating documentation for section: preventGeneratedFileEdits
Generating documentation for section: sessionStart

Documentation generation complete!
Generated files in: docs/src/content/docs/reference/config
```

Generated files include:
- `configuration.md` - Overview page with quick reference
- `hooks.md` - Hooks configuration documentation
- `files.md` - File protection rules documentation
- `session-start.md` - Session startup configuration
- Additional section pages as needed

## Common Issues

### Schema File Not Found

If the generator can't find the schema:
- Ensure `conclaude-schema.json` exists in the workspace root
- Run from the workspace root directory
- Regenerate the schema: `cargo run --bin generate-schema`

### Build Errors

If the generator fails to build:
- Ensure Rust toolchain is installed: `cargo --version`
- Check that all dependencies are available: `cargo build`
- Verify `src/bin/generate-docs.rs` compiles

### Output Directory Creation Failed

If it fails to create the output directory:
- Check file system permissions
- Ensure parent directories exist
- Try specifying an absolute path with `--output`

### Missing Examples or Descriptions

If generated docs lack examples or seem incomplete:
- Check that schema descriptions include YAML examples in ` ```yaml ` code blocks
- Verify schema descriptions are properly formatted
- Ensure nested types are defined in the `definitions` section

### Incorrect Markdown Formatting

If the output has formatting issues:
- Review the schema descriptions for special characters
- Check that YAML examples are properly escaped
- Verify Starlight frontmatter is correctly formatted

## Output File Structure

The generator creates the following structure:

```
docs/src/content/docs/reference/config/
├── configuration.md          # Overview with quick reference table
├── hooks.md                  # Hooks configuration section
├── files.md                  # File protection rules section
├── session-start.md          # Session startup configuration
└── [other-sections].md       # Additional configuration sections
```

Each section page includes:
- Starlight frontmatter (title, description)
- Section overview
- Configuration properties table with types and defaults
- Nested type definitions (if applicable)
- Complete YAML examples
- Navigation links back to overview

## Related Files

- `src/bin/generate-docs.rs` - The documentation generator binary
- `conclaude-schema.json` - Input JSON Schema file
- `docs/src/content/docs/reference/config/` - Output directory for generated docs
- `src/bin/generate-schema.rs` - Schema generator (run before this)

## Schema Description Format

For best documentation output, schema descriptions should follow this format:

```rust
/// Main description of the configuration section.
/// This part appears in the overview and section introduction.
///
/// # Examples
///
/// ```yaml
/// sectionName:
///   property: value
///   anotherProperty: "example"
/// ```
///
/// Additional context or notes can go here.
```

The generator will:
- Extract the main description (before `# Examples`)
- Parse and format YAML code blocks as separate examples
- Include type information from the schema
- Document nested types automatically

## Tips for Maintaining Documentation

1. **Always regenerate after schema changes**: Documentation should stay in sync with the schema
2. **Keep schema descriptions clear**: They become user-facing documentation
3. **Include practical examples**: YAML examples in schema descriptions are extracted and displayed
4. **Document nested types**: Complex configuration structures are automatically documented
5. **Use consistent terminology**: Match the language used in configuration files
6. **Review generated output**: Always verify the generated docs render correctly in Starlight

## Command Reference

```bash
# Basic usage (default output directory)
cargo run --bin generate-docs

# Custom output directory
cargo run --bin generate-docs -- --output path/to/docs

# Show help
cargo run --bin generate-docs -- --help

# Complete workflow: schema + docs
cargo run --bin generate-schema && cargo run --bin generate-docs
```

## Notes

- Documentation is automatically generated from the schema, so manual edits to generated files will be overwritten
- Always regenerate documentation after updating the schema
- The generator creates deterministic output (sorted sections/properties)
- Generated files include Starlight frontmatter for proper site integration
- YAML examples are extracted from schema description fields
- Nested types are automatically detected and documented
- The documentation site must be rebuilt to see changes: `cd docs && npm run build`
