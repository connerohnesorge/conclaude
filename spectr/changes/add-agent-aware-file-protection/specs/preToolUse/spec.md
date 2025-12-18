# preToolUse Spec Delta

## ADDED Requirements

### Requirement: Agent-Aware File Protection

The system SHALL support optional `agent` field in `uneditableFiles` detailed format to restrict file protection rules to specific agents or agent patterns.

#### Scenario: Rule with agent wildcard applies to all agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: ".conclaude.yml", agent: "*" }`
- **AND** any agent (main session, coder, tester, etc.) attempts to edit `.conclaude.yml`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL indicate the file matches uneditableFiles pattern

#### Scenario: Rule with specific agent only blocks that agent

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "tasks.jsonc", agent: "coder" }`
- **AND** the "coder" subagent attempts to edit `tasks.jsonc`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL include agent context (e.g., "agent: coder")

#### Scenario: Rule with specific agent does not block other agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "tasks.jsonc", agent: "coder" }`
- **AND** the main session (orchestrator) attempts to edit `tasks.jsonc`
- **THEN** the operation SHALL NOT be blocked by this rule
- **AND** the file is editable unless blocked by another rule

#### Scenario: Rule with glob pattern matches multiple agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "src/**/*.ts", agent: "code*" }`
- **AND** the "coder" subagent attempts to edit `src/app.ts`
- **THEN** the operation SHALL be blocked (pattern "code*" matches "coder")
- **AND** if "coder-v2" subagent attempts the same, it SHALL also be blocked

#### Scenario: Main session identified as "main" agent

- **WHEN** preToolUse hook executes in the main Claude session (not a subagent)
- **AND** `preToolUse.uneditableFiles` contains `{ pattern: "config.yml", agent: "main" }`
- **AND** main session attempts to edit `config.yml`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL reference agent "main"

#### Scenario: Subagent identified from transcript

- **WHEN** preToolUse hook executes within a subagent context
- **THEN** the system SHALL parse the transcript to extract the subagent type from the Task tool invocation
- **AND** the extracted subagent type SHALL be used for agent pattern matching

### Requirement: Agent Field Default Behavior

The system SHALL default `agent` to `"*"` (all agents) when the field is omitted, ensuring backward compatibility with existing configurations.

#### Scenario: Rule without agent field applies to all agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: ".env" }` (no agent field)
- **AND** any agent attempts to edit `.env`
- **THEN** the operation SHALL be blocked
- **AND** behavior SHALL be identical to `{ pattern: ".env", agent: "*" }`

#### Scenario: Simple string format applies to all agents

- **WHEN** `preToolUse.uneditableFiles` contains `".env"` (simple string format)
- **AND** any agent attempts to edit `.env`
- **THEN** the operation SHALL be blocked
- **AND** simple format SHALL continue to apply to all agents

### Requirement: Agent Pattern Matching Semantics

The system SHALL use glob-style pattern matching for the `agent` field, consistent with file pattern matching.

#### Scenario: Wildcard matches all agents

- **WHEN** agent pattern is `"*"`
- **THEN** it SHALL match main session, coder, tester, and any other agent type

#### Scenario: Exact match requires full agent name

- **WHEN** agent pattern is `"coder"`
- **THEN** it SHALL match agent type "coder" exactly
- **AND** it SHALL NOT match "coder-v2" or "tester"

#### Scenario: Glob pattern with asterisk suffix

- **WHEN** agent pattern is `"code*"`
- **THEN** it SHALL match "coder", "coder-v2", "codefix", etc.
- **AND** it SHALL NOT match "tester" or "main"

#### Scenario: Case-sensitive matching

- **WHEN** agent pattern is `"Coder"` (capitalized)
- **AND** actual agent type is `"coder"` (lowercase)
- **THEN** the pattern SHALL NOT match (case-sensitive)

### Requirement: Agent Context in Error Messages

The system SHALL include agent context in error messages when an agent-specific uneditableFiles rule blocks an operation.

#### Scenario: Error message includes agent for specific rules

- **WHEN** a uneditableFiles rule with `agent: "coder"` blocks an operation
- **THEN** error message SHALL include the agent context
- **AND** format SHALL be: `"Blocked {tool} operation: file matches preToolUse.uneditableFiles pattern '{pattern}' (agent: {agent}). File: {path}"`

#### Scenario: Error message omits agent for universal rules

- **WHEN** a uneditableFiles rule with `agent: "*"` or no agent field blocks an operation
- **THEN** error message MAY omit agent context
- **AND** format MAY remain: `"Blocked {tool} operation: file matches preToolUse.uneditableFiles pattern '{pattern}'. File: {path}"`

### Requirement: Agent Detection Failure Handling

The system SHALL handle agent detection failures gracefully without blocking legitimate operations.

#### Scenario: Transcript parse failure defaults to main

- **WHEN** the transcript cannot be parsed to extract agent type
- **THEN** the system SHALL log a warning
- **AND** the system SHALL treat the session as "main" agent
- **AND** rules with `agent: "main"` or `agent: "*"` SHALL apply

#### Scenario: Missing transcript defaults to main

- **WHEN** the transcript path is empty or file does not exist
- **THEN** the system SHALL treat the session as "main" agent
- **AND** agent-specific rules SHALL be evaluated against "main"

## MODIFIED Requirements

### Requirement: File Protection via Glob Patterns

The system SHALL prevent Claude from editing specified files using glob patterns in the `uneditableFiles` configuration, **with optional agent scoping**.

#### Scenario: Exact file match (updated)

- **WHEN** `preToolUse.uneditableFiles` contains `"package.json"` (simple format)
- **THEN** ALL agents SHALL NOT be allowed to edit `package.json` at any directory level
- **AND** any attempt to modify this file SHALL result in an error message

#### Scenario: Detailed format with agent scoping

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "package.json", message: "...", agent: "coder" }`
- **THEN** only the "coder" agent SHALL be blocked from editing `package.json`
- **AND** other agents (main, tester) SHALL NOT be blocked by this rule

#### Scenario: Configuration validation with agent field

- **WHEN** `preToolUse.uneditableFiles` contains a detailed entry with `agent` field
- **AND** `agent` is a valid string (including glob patterns)
- **THEN** the configuration SHALL be accepted
- **AND** the agent pattern SHALL be validated against glob syntax
