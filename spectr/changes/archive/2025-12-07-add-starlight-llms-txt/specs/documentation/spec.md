# Documentation Spec Delta

## ADDED Requirements

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
