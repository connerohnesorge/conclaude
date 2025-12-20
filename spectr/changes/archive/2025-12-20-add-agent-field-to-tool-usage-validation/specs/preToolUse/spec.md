## MODIFIED Requirements

### Requirement: Tool Usage Validation Rules

The system SHALL enforce per-tool restrictions defined in `toolUsageValidation` to control which tools can operate on which files, optionally scoped to specific agents.

#### Scenario: Block tool on file pattern

- **WHEN** `preToolUse.toolUsageValidation` contains a rule blocking "bash" on "*.md" files
- **THEN** Claude SHALL NOT be allowed to execute bash commands on markdown files
- **AND** any attempt SHALL result in an error message referencing the tool usage rule

#### Scenario: Allow tool on specific pattern

- **WHEN** `preToolUse.toolUsageValidation` contains a rule allowing "Write" only on "src/**/*.ts"
- **THEN** Claude SHALL NOT be allowed to use Write tool on files outside the `src/` TypeScript directory
- **AND** the permission boundary SHALL be enforced

#### Scenario: Command pattern matching

- **WHEN** a tool usage rule includes a `commandPattern` (e.g., regex)
- **THEN** the rule SHALL match against the specific command/parameters passed to the tool
- **AND** match mode (exact, regex, glob) SHALL determine matching behavior

#### Scenario: Multiple validation rules

- **WHEN** multiple `toolUsageValidation` rules are configured
- **THEN** all applicable rules SHALL be evaluated
- **AND** the first matched rule SHALL determine the action (block/allow)

#### Scenario: Rule precedence with match modes

- **WHEN** multiple rules could apply to the same tool and file pattern
- **THEN** rules SHALL be evaluated in order
- **AND** the first matching rule SHALL take precedence

#### Scenario: Validation error messages

- **WHEN** a tool usage rule blocks an operation
- **THEN** the error message SHALL include the tool name, file pattern, and custom message if provided
- **AND** the user SHALL understand why the operation was blocked

#### Scenario: Agent-scoped tool usage rule

- **GIVEN** configuration contains:
  ```yaml
  toolUsageValidation:
    - tool: "Bash"
      pattern: "*"
      action: "block"
      commandPattern: "git push*"
      agent: "coder"
      message: "Coder agent cannot push to git"
  ```
- **WHEN** the "coder" subagent attempts to run `git push origin main`
- **THEN** the operation SHALL be blocked with the custom message
- **AND** the orchestrator ("main") SHALL still be allowed to run git push commands

#### Scenario: Agent pattern uses glob matching

- **GIVEN** configuration contains a rule with `agent: "test*"`
- **WHEN** a subagent named "tester" or "test-runner" attempts an operation matching the rule
- **THEN** the rule SHALL apply to both agents
- **AND** agents not matching the pattern (e.g., "coder") SHALL not be affected

#### Scenario: Missing agent field defaults to all agents

- **GIVEN** configuration contains a rule without an `agent` field
- **WHEN** any agent (orchestrator or subagent) attempts an operation matching the rule
- **THEN** the rule SHALL apply to all agents
- **AND** behavior SHALL be equivalent to specifying `agent: "*"`

#### Scenario: Agent field with wildcard matches all

- **GIVEN** configuration contains a rule with `agent: "*"`
- **WHEN** any agent attempts an operation matching the rule
- **THEN** the rule SHALL apply regardless of agent type

#### Scenario: Rule skipped for non-matching agent

- **GIVEN** configuration contains a rule with `agent: "coder"`
- **WHEN** the "tester" subagent attempts an operation that would otherwise match the rule
- **THEN** the rule SHALL be skipped (not applied)
- **AND** subsequent rules SHALL continue to be evaluated

#### Scenario: Agent context detection from session

- **GIVEN** a subagent session is active with `subagent_type: "coder"`
- **WHEN** evaluating tool usage rules with `agent` fields
- **THEN** the system SHALL read the current agent from the session file
- **AND** use "main" if no session file exists (orchestrator context)

#### Scenario: Error message includes agent context

- **WHEN** an agent-scoped tool usage rule blocks an operation
- **THEN** the error message SHALL include the current agent name
- **AND** the error SHALL indicate which agent pattern triggered the block
