## ADDED Requirements

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
