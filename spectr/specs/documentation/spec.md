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
