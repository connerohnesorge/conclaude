# documentation Specification

## Purpose
TBD - created by archiving change add-automated-changelog. Update Purpose after archive.
## Requirements

### Requirement: Automated Changelog Generation

The system SHALL automatically generate a CHANGELOG.md file from git commit history when version tags are pushed to the repository.

#### Scenario: Tag push triggers changelog generation

- **WHEN** a version tag matching pattern `v*.*.*` is pushed to the repository
- **THEN** the changelog workflow executes and generates an updated CHANGELOG.md

#### Scenario: Conventional commits are parsed

- **WHEN** git-cliff processes commit history
- **THEN** commits following conventional commit format (feat:, fix:, chore:, etc.) are grouped by type

#### Scenario: Changelog is committed back to main

- **WHEN** CHANGELOG.md is generated successfully
- **THEN** the workflow commits the file to main branch using github-actions bot credentials

#### Scenario: No-op when no changes detected

- **WHEN** CHANGELOG.md content is identical to existing file
- **THEN** the workflow skips the commit step

### Requirement: Git-cliff Configuration

The system SHALL use git-cliff with conventional commits support to parse git history and format changelog entries.

#### Scenario: Configuration file exists

- **WHEN** the repository is initialized
- **THEN** a `cliff.toml` configuration file exists at the repository root

#### Scenario: Commit grouping by type

- **WHEN** git-cliff generates changelog
- **THEN** commits are grouped by type (Features, Bug Fixes, Chores, etc.)

#### Scenario: Commit message formatting

- **WHEN** a commit is included in the changelog
- **THEN** the entry shows the commit message first line and short hash (7 characters)

### Requirement: GitHub Actions Workflow

The system SHALL provide a GitHub Actions workflow that orchestrates changelog generation on tag events.

#### Scenario: Workflow file location

- **WHEN** the repository contains changelog automation
- **THEN** a workflow file exists at `.github/workflows/changelog.yml`

#### Scenario: Full git history available

- **WHEN** the workflow checks out the repository
- **THEN** it uses `fetch-depth: 0` to ensure all tags and history are available

#### Scenario: Git-cliff installation

- **WHEN** the workflow prepares to generate changelog
- **THEN** it installs git-cliff using the official GitHub action `orhun/git-cliff-action@v3`

#### Scenario: Bot commit credentials

- **WHEN** the workflow commits CHANGELOG.md
- **THEN** it uses github-actions bot name and email for git configuration

### Requirement: Release Workflow Integration

The system SHALL integrate with the existing cargo-dist release workflow without conflicts.

#### Scenario: Independent workflow execution

- **WHEN** both release.yml and changelog.yml are triggered by the same tag push
- **THEN** they execute independently without blocking each other

#### Scenario: Changelog available for release notes

- **WHEN** cargo-dist creates a GitHub release
- **THEN** the updated CHANGELOG.md is available in the repository

### Requirement: Starlight Changelogs Plugin Integration

The documentation site SHALL integrate the starlight-changelogs plugin to display project release history from GitHub releases.

#### Scenario: Plugin is installed and configured

- **WHEN** the documentation site is built
- **THEN** the starlight-changelogs plugin is loaded and active in the Astro configuration

#### Scenario: Changelogs collection is available

- **WHEN** the site content is processed
- **THEN** a changelogs collection is defined in the content configuration using changelogsLoader

### Requirement: GitHub Provider Configuration

The documentation site SHALL use the GitHub provider to fetch and display releases from the conclaude repository.

#### Scenario: GitHub releases are fetched

- **WHEN** the documentation site builds or the changelog page is accessed
- **THEN** the plugin fetches release data from the conclaude GitHub repository

#### Scenario: Repository owner and name are configured

- **WHEN** the GitHub provider is initialized
- **THEN** it uses the correct repository owner and repository name for the conclaude project

#### Scenario: Changelog base path is configured

- **WHEN** a user navigates to changelog pages
- **THEN** changelog routes are available under the configured base path (e.g., /changelog/)

### Requirement: Documentation Site Branding

The documentation site SHALL be properly branded with the conclaude project name and repository link.

#### Scenario: Site title reflects project name

- **WHEN** the documentation site is loaded
- **THEN** the site title displays 'conclaude' instead of generic placeholder text

#### Scenario: GitHub social link points to conclaude repository

- **WHEN** a user clicks the GitHub social icon
- **THEN** they are directed to the conclaude GitHub repository

### Requirement: Changelog Navigation

The documentation site SHALL provide navigation access to the changelog section.

#### Scenario: Changelog appears in sidebar

- **WHEN** a user views the documentation site
- **THEN** a changelog navigation entry is visible in the sidebar

#### Scenario: Changelog link routes correctly

- **WHEN** a user clicks the changelog navigation entry
- **THEN** they are taken to the changelog index or overview page

### Requirement: LLM Context File Generation

The documentation site SHALL automatically generate machine-readable context files (`llms.txt`, `llms-full.txt`, `llms-small.txt`) that enable AI systems to learn from documentation content.

#### Scenario: Plugin installed and configured

- **WHEN** the `starlight-llms-txt` plugin is added to the Starlight configuration
- **THEN** the plugin is registered in the Starlight plugins array in `astro.config.mjs`

#### Scenario: Build generates llms.txt files

- **WHEN** the documentation site is built
- **THEN** three files are generated: `llms.txt`, `llms-full.txt`, and `llms-small.txt` at the site root

#### Scenario: Generated files contain documentation content

- **WHEN** the llms.txt files are generated
- **THEN** they contain formatted documentation content from all pages in the site

#### Scenario: Files accessible via HTTP

- **WHEN** the built documentation is served
- **THEN** `/llms.txt`, `/llms-full.txt`, and `/llms-small.txt` are accessible at their respective URLs

### Requirement: Site URL Configuration

The documentation site SHALL have a configured site URL required for proper plugin operation.

#### Scenario: Site URL present in config

- **WHEN** the `starlight-llms-txt` plugin is configured
- **THEN** the `site` property is set in the Astro configuration

### Requirement: Conclaude-Branded Homepage

The documentation site homepage SHALL display conclaude branding and content instead of generic Starlight placeholder text.

#### Scenario: User visits documentation homepage
- **WHEN** a user navigates to the documentation site root
- **THEN** they see a hero section with conclaude branding
- **AND** the tagline communicates conclaude's value proposition ("Guardrails for Claude Code sessions")
- **AND** quick installation commands are displayed
- **AND** feature cards highlight key capabilities (Hook System, YAML Configuration, File Protection, Session Logging)
- **AND** call-to-action buttons link to Getting Started guide and Configuration Reference

### Requirement: Getting Started Guide

The documentation site SHALL include a Getting Started guide that helps new users configure conclaude for the first time.

#### Scenario: User follows getting started guide
- **WHEN** a user reads the Getting Started guide
- **THEN** they understand how to install conclaude
- **AND** they can create their first `.conclaude.yaml` configuration
- **AND** they understand how to run `conclaude init`
- **AND** they know how to verify their configuration with `conclaude validate`

### Requirement: CLI Reference Documentation

The documentation site SHALL include a CLI reference page documenting all available commands and their options.

#### Scenario: User looks up CLI command
- **WHEN** a user accesses the CLI reference
- **THEN** they find documentation for all commands: init, validate, and hook commands
- **AND** each command includes description, options, and examples
- **AND** exit codes are documented
- **AND** environment variables are listed

### Requirement: Installation Guide

The documentation site SHALL include a comprehensive installation guide covering all supported installation methods.

#### Scenario: User chooses installation method
- **WHEN** a user reads the installation guide
- **THEN** they find instructions for shell script installation
- **AND** they find instructions for PowerShell installation on Windows
- **AND** they find instructions for npm package installation
- **AND** they find instructions for manual binary download
- **AND** they find instructions for Nix flake installation
- **AND** they find instructions for building from source

### Requirement: Hooks Overview Guide

The documentation site SHALL include a guide explaining the hook system and how to configure each hook type.

#### Scenario: User learns about hooks
- **WHEN** a user reads the hooks overview guide
- **THEN** they understand the hook lifecycle (PreToolUse, PostToolUse, Stop)
- **AND** they understand session hooks (SessionStart, UserPromptSubmit)
- **AND** they understand subagent hooks (SubagentStart, SubagentStop)
- **AND** they find example configurations for each hook type
- **AND** they understand when and why to use each hook

### Requirement: Client-Side Search with Pagefind

The documentation site SHALL provide fast, client-side search functionality powered by the Pagefind search engine through the star-warp plugin.

#### Scenario: Plugin installed and configured

- **WHEN** the documentation site is built
- **THEN** the @inox-tools/star-warp plugin is installed as a dependency and integrated into the Astro configuration

#### Scenario: Search index generated at build time

- **WHEN** the site build process completes
- **THEN** Pagefind generates a search index of all documentation content

#### Scenario: Search interface available

- **WHEN** users access the documentation site
- **THEN** a search interface is available allowing users to query documentation content

#### Scenario: Search results returned instantly

- **WHEN** users enter a search query
- **THEN** results are returned from the client-side index without server requests

#### Scenario: Default configuration used

- **WHEN** the plugin is integrated
- **THEN** it uses minimal/default configuration settings without custom options

### Requirement: Configuration Documentation Generation

The system SHALL provide a documentation generator that extracts configuration metadata from the JSON Schema and outputs Markdown reference documentation compatible with Starlight.

#### Scenario: Generator produces valid Starlight Markdown

- **WHEN** the generator is invoked via `cargo run --bin generate-docs`
- **THEN** it SHALL output Markdown files to `docs/src/content/docs/reference/`
- **AND** each file SHALL include valid Starlight frontmatter with `title` and `description` fields

#### Scenario: Hybrid page structure is generated

- **WHEN** the generator runs
- **THEN** it SHALL produce an overview page at `reference/configuration.md`
- **AND** it SHALL produce detail pages for each configuration section:
  - `reference/config/stop.md`
  - `reference/config/subagent-stop.md`
  - `reference/config/pre-tool-use.md`
  - `reference/config/notifications.md`
  - `reference/config/permission-request.md`

#### Scenario: Overview page contains quick reference

- **WHEN** the overview page is generated
- **THEN** it SHALL include a summary table of all configuration sections
- **AND** it SHALL link to the detailed per-section pages

#### Scenario: Field metadata is extracted from schema

- **WHEN** a configuration field has metadata in the JSON Schema
- **THEN** the generated documentation SHALL include:
  - Field name (using the YAML/JSON key, not Rust field name)
  - Type information (string, boolean, integer, array, object)
  - Default value if specified
  - Description from schema if available
  - Validation constraints (min/max ranges) if applicable

#### Scenario: YAML examples are included

- **WHEN** detail pages are generated
- **THEN** each section SHALL include inline YAML code blocks showing common configurations
- **AND** each page SHALL link to `default-config.yaml` for the complete reference

#### Scenario: Nested types are documented

- **WHEN** a configuration field references a nested type (e.g., `StopCommand`, `ToolUsageRule`)
- **THEN** the generator SHALL include a subsection documenting that nested type's fields

### Requirement: Schema Description Consolidation

The system SHALL use Rust doc comments as the single source of truth for configuration field descriptions.

#### Scenario: Doc comments populate schema descriptions

- **WHEN** a struct field in `src/config.rs` has a `///` doc comment
- **THEN** the `schemars` derive SHALL include that comment as the field's `description` in `conclaude-schema.json`

#### Scenario: All configuration fields have descriptions

- **WHEN** the schema is generated
- **THEN** every user-facing configuration field SHALL have a non-empty description
- **AND** descriptions SHALL explain the field's purpose and valid values

#### Scenario: YAML comments are simplified

- **WHEN** descriptions are consolidated to Rust doc comments
- **THEN** `default-config.yaml` SHALL contain only minimal example-focused comments
- **AND** detailed explanations SHALL be in the generated documentation

### Requirement: Documentation Synchronization

The system SHALL support verification that generated documentation matches the current schema.

#### Scenario: CI validates documentation is current

- **WHEN** the CI pipeline runs
- **THEN** it SHALL regenerate documentation and compare with committed version
- **AND** the build SHALL fail if generated output differs from committed documentation

#### Scenario: Generator is idempotent

- **WHEN** the generator is run multiple times with identical schema input
- **THEN** the output SHALL be identical each time (no non-deterministic ordering or timestamps)
### Requirement: Internal Link Validation

The documentation site SHALL validate all internal links during production builds using the starlight-links-validator plugin.

#### Scenario: Plugin is configured

- **WHEN** the Starlight configuration is loaded
- **THEN** the `starlightLinksValidator` plugin is included in the plugins array

#### Scenario: Internal links are validated on build

- **WHEN** a production build is executed (`npm run build`)
- **THEN** the plugin validates all internal links in Markdown and MDX files

#### Scenario: Broken internal link fails build

- **WHEN** a Markdown file contains a broken internal link (404)
- **THEN** the production build fails with an error indicating the broken link

#### Scenario: External links are ignored

- **WHEN** a Markdown file contains external links (http/https URLs)
- **THEN** the plugin ignores these links and does not validate them

#### Scenario: Hash links are validated

- **WHEN** a Markdown file links to a heading anchor (e.g., `#installation`)
- **THEN** the plugin validates that the target heading exists in the referenced page
