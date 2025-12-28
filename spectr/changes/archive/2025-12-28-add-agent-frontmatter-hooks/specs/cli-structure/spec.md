## ADDED Requirements

### Requirement: Agent Flag for Hook Commands

All hook CLI commands SHALL accept an optional `--agent <name>` flag to specify agent context.

#### Scenario: Hook command with agent flag
- **WHEN** user executes `conclaude Hooks PreToolUse --agent coder`
- **THEN** the hook handler SHALL receive "coder" as the agent context
- **AND** CONCLAUDE_AGENT_NAME environment variable SHALL be set to "coder"
- **AND** transcript parsing for agent name SHALL be skipped

#### Scenario: Hook command without agent flag
- **WHEN** user executes `conclaude Hooks PreToolUse` without --agent flag
- **THEN** the hook handler SHALL operate in main session context
- **AND** CONCLAUDE_AGENT_NAME environment variable SHALL NOT be set
- **AND** backward compatibility SHALL be maintained

#### Scenario: All hook types support agent flag
- **WHEN** any hook command is executed (PreToolUse, PostToolUse, Stop, etc.)
- **THEN** the `--agent <name>` flag SHALL be accepted
- **AND** the agent name SHALL be passed to the corresponding handler
