## Context

Conclaude configuration is defined in `src/config.rs` using Rust structs with:
- `serde` annotations for (de)serialization and field renaming
- `schemars::JsonSchema` derive for JSON Schema generation
- Doc comments describing fields
- `FieldList` custom derive for error suggestion support

The project already generates `conclaude-schema.json` which contains type definitions, descriptions, defaults, and validation constraints. The Starlight documentation site is in `docs/` using Astro.

## Goals / Non-Goals

**Goals:**
- Generate accurate, up-to-date Markdown reference documentation from code
- Support Starlight frontmatter format
- Include type information, defaults, constraints, and descriptions
- Single source of truth eliminates documentation drift

**Non-Goals:**
- Interactive documentation features
- API documentation (this is configuration only)
- Replacing all manual documentation

## Decisions

### Decision: Use JSON Schema as primary source

Parse `conclaude-schema.json` instead of directly parsing Rust AST.

**Why:**
- Schema already contains all necessary metadata (types, defaults, constraints)
- Schemars handles the complexity of Rust type extraction
- Schema is the canonical contract for configuration validation
- Simpler implementation than AST parsing

**Alternatives considered:**
- Direct Rust AST parsing: More complex, duplicates schemars work
- Extend `FieldList` macro: Would require significant macro changes

### Decision: Standalone generator binary

Create `src/bin/generate-docs.rs` rather than a build script.

**Why:**
- Explicit invocation via `cargo run --bin generate-docs`
- Can be run in CI or manually
- No build-time dependencies on doc generation
- Keeps build times fast

**Alternatives considered:**
- build.rs: Would slow every build, adds complexity
- Xtask pattern: Overkill for single task

### Decision: Single comprehensive reference page

Generate one `configuration.md` file with all sections, using heading anchors.

**Why:**
- Easier to search (Ctrl+F)
- Starlight table of contents works well with nested headings
- Avoids navigation complexity

**Alternatives considered:**
- Separate page per section: More navigation overhead, harder to get overview

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Generated docs drift if not regenerated | CI check compares generated output |
| Schema lacks doc comments | Can augment with inline comments in generator |
| Breaking Starlight build | Validate output format in CI |

## Migration Plan

1. Create generator binary
2. Run generator to create initial `configuration.md`
3. Verify Starlight builds and renders correctly
4. Add CI step to check docs are up-to-date
5. Remove any manually-maintained duplicate configuration docs

## Resolved Decisions

### YAML Examples: Hybrid approach
Include inline YAML snippets for common use cases in each section, plus link to `default-config.yaml` for full reference. This provides quick copy-paste examples while avoiding full duplication.

### Documentation Source: Context-aware selection
Use whichever source has richer context per field. Currently `default-config.yaml` has the most detailed comments. As part of this change, consolidate YAML comment content into Rust doc comments on the structs, making schema descriptions the single source of truth going forward.

### Page Format: Hybrid structure
- `configuration.md` - Overview page with quick reference table and links
- `reference/config/stop.md` - Detailed stop hook configuration
- `reference/config/pre-tool-use.md` - Detailed preToolUse configuration
- `reference/config/notifications.md` - Detailed notifications configuration
- `reference/config/permission-request.md` - Detailed permissionRequest configuration
- `reference/config/subagent-stop.md` - Detailed subagentStop configuration

### Comment Consolidation
Migrate detailed comments from `default-config.yaml` into Rust doc comments on struct fields in `src/config.rs`. This ensures schemars generates rich descriptions in the JSON Schema, enabling the documentation generator to use schema as the single source.
