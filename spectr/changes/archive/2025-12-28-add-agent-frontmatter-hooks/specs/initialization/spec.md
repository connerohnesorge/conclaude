## ADDED Requirements

### Requirement: Agent Frontmatter Hook Injection

The `conclaude init` command SHALL discover agent files and inject hooks frontmatter.

#### Scenario: Agent files discovered and updated
- **WHEN** a user runs `conclaude init` in a project
- **AND** `.claude/agents/` directory contains markdown files
- **THEN** each agent file SHALL be updated with a `hooks` section in frontmatter
- **AND** the hooks section SHALL include all hook types
- **AND** each hook SHALL call `conclaude Hooks <type> --agent <name>`

#### Scenario: Agent name extracted from frontmatter
- **WHEN** an agent file has a `name` field in frontmatter
- **THEN** the hooks SHALL use that name value
- **AND** the command format SHALL be `conclaude Hooks <type> --agent <name>`

#### Scenario: Agent name derived from filename
- **WHEN** an agent file lacks a `name` field in frontmatter
- **THEN** the name SHALL be derived from the filename (without .md extension)
- **AND** a warning SHALL be logged suggesting adding a name field

#### Scenario: Existing hooks preserved
- **WHEN** an agent file already has a `hooks` section
- **THEN** conclaude-generated hooks SHALL be merged
- **AND** user-defined hooks SHALL NOT be overwritten

## MODIFIED Requirements

### Requirement: Generated Hooks Include Timeout Configuration

The `conclaude init` command SHALL generate Claude Code hook configurations that include a timeout field to prevent indefinite hook execution.

#### Scenario: Default timeout for generated hooks

- **WHEN** a user runs `conclaude init` to initialize a project
- **THEN** the generated `.claude/settings.json` SHALL include hook configurations with a `timeout` field
- **AND** the timeout value SHALL be 600 seconds (10 minutes)
- **AND** SubagentStart and SubagentStop hooks SHALL NOT be included in settings.json
- **AND** agent-specific hooks SHALL be defined in agent frontmatter instead

#### Scenario: Settings.json hook structure with timeout

- **WHEN** the `.claude/settings.json` file is generated or updated
- **THEN** each hook configuration SHALL follow the Claude Code hooks schema
- **AND** SubagentStart and SubagentStop SHALL be omitted from the hooks object
