## REMOVED Requirements

### Requirement: Agent Name Extraction from Main Transcript

**Reason:** Agent name is now provided directly via `--agent` flag in frontmatter hooks, eliminating the need for transcript parsing.

**Migration:** Run `conclaude init` to inject hooks into agent frontmatter. The `--agent` flag provides agent name directly.

### Requirement: Agent ID from Payload

**Reason:** With frontmatter hooks, agent identity is explicit in the hook command rather than extracted from payload.

**Migration:** Agent-aware hooks now use `--agent <name>` flag instead of payload extraction.

## MODIFIED Requirements

### Requirement: Subagent Stop Command Configuration

The system SHALL provide a `subagentStop` configuration section that maps subagent name patterns to lists of commands to execute when matching subagents terminate.

#### Scenario: Commands receive agent name from flag
- **WHEN** a Stop hook fires for an agent with frontmatter hooks
- **AND** the hook command includes `--agent coder`
- **THEN** subagentStop commands SHALL use the provided agent name for pattern matching
- **AND** transcript parsing SHALL be skipped
- **AND** CONCLAUDE_AGENT_NAME environment variable SHALL equal "coder"

#### Scenario: Wildcard pattern configuration
- **WHEN** config includes `subagentStop.commands["*"]` with a list of commands
- **THEN** those commands SHALL execute for every subagent that stops
- **AND** each command SHALL include run, message, showStdout, showStderr, maxOutputLines fields
