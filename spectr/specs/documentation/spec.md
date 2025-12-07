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
