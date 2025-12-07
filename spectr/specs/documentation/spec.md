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
