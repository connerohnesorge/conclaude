# Configuration Delta: Slash Command and Skill Hooks

## ADDED Requirements

### Requirement: Slash Command Hook Configuration

The system SHALL support configuring hooks that trigger when slash commands are detected in user prompts.

#### Scenario: Basic slash command configuration

- **GIVEN** a `.conclaude.yaml` configuration with:
  ```yaml
  userPromptSubmit:
    slashCommands:
      commands:
        "/commit":
          - run: ".claude/scripts/pre-commit.sh"
  ```
- **WHEN** a user submits a prompt containing `/commit`
- **THEN** the configured command SHALL execute
- **AND** the `CONCLAUDE_SLASH_COMMAND` environment variable SHALL be set to "commit"

#### Scenario: Slash command with arguments

- **GIVEN** a slash command configuration for `/deploy`
- **WHEN** a user submits `/deploy production --force`
- **THEN** `CONCLAUDE_SLASH_COMMAND` SHALL be "deploy"
- **AND** `CONCLAUDE_SLASH_COMMAND_ARGS` SHALL be "production --force"

#### Scenario: Glob pattern matching for slash commands

- **GIVEN** a configuration with pattern `/test*`
- **WHEN** a user submits `/test-unit` or `/testing`
- **THEN** the configured commands SHALL execute for both
- **AND** commands configured for `/commit` SHALL NOT execute

#### Scenario: Wildcard slash command configuration

- **GIVEN** a configuration with pattern `"*"`
- **WHEN** any slash command is detected
- **THEN** the wildcard commands SHALL execute
- **AND** more specific patterns SHALL take precedence when both match

### Requirement: Skill Start Hook Configuration

The system SHALL support configuring hooks that trigger when skills (subagents) are started.

#### Scenario: Basic skill start configuration

- **GIVEN** a `.conclaude.yaml` configuration with:
  ```yaml
  skillStart:
    commands:
      "coder":
        - run: ".claude/scripts/coder-init.sh"
  ```
- **WHEN** a SubagentStart event occurs with `agent_type` = "coder"
- **THEN** the configured command SHALL execute
- **AND** the `CONCLAUDE_SKILL_NAME` environment variable SHALL be set to "coder"

#### Scenario: Skill start with agent ID

- **GIVEN** a skill start configuration
- **WHEN** a SubagentStart event occurs with `agent_id` = "abc123"
- **THEN** `CONCLAUDE_AGENT_ID` environment variable SHALL be set to "abc123"

#### Scenario: Glob pattern matching for skills

- **GIVEN** a configuration with pattern `test*`
- **WHEN** a SubagentStart event occurs with `agent_type` = "tester" or "test-runner"
- **THEN** the configured commands SHALL execute for both

#### Scenario: Wildcard skill configuration

- **GIVEN** a configuration with pattern `"*"`
- **WHEN** any skill starts
- **THEN** the wildcard commands SHALL execute

### Requirement: Slash Command Detection from Prompts

The system SHALL detect slash commands from user prompt text using pattern matching.

#### Scenario: Slash command at prompt start

- **GIVEN** a prompt text `/commit fix typo in README`
- **WHEN** the prompt is analyzed for slash commands
- **THEN** a slash command SHALL be detected with name "commit"
- **AND** arguments SHALL be "fix typo in README"

#### Scenario: Slash command after newline

- **GIVEN** a prompt text with multiple lines where line 2 is `/deploy prod`
- **WHEN** the prompt is analyzed
- **THEN** the slash command SHALL be detected on that line

#### Scenario: No slash command in normal text

- **GIVEN** a prompt text "Please fix the /path/to/file.txt issue"
- **WHEN** the prompt is analyzed for slash commands
- **THEN** no slash command SHALL be detected
- **AND** the path SHALL NOT be misinterpreted as a command

#### Scenario: Multiple slash commands (first wins)

- **GIVEN** a prompt with `/commit` on line 1 and `/deploy` on line 2
- **WHEN** the prompt is analyzed
- **THEN** only the first slash command ("commit") SHALL be processed

### Requirement: Slash Command Hook Blocking

The system SHALL support blocking slash command execution based on hook command exit codes.

#### Scenario: Blocking a slash command

- **GIVEN** a slash command hook configured for `/deploy`
- **WHEN** the hook command exits with code 2
- **THEN** the slash command SHALL be blocked
- **AND** the configured message SHALL be returned to Claude

#### Scenario: Non-blocking hook failure

- **GIVEN** a slash command hook configured for `/commit`
- **WHEN** the hook command exits with a non-zero, non-2 code
- **THEN** the slash command SHALL NOT be blocked
- **AND** the error SHALL be logged to stderr

### Requirement: Environment Variables for Slash Commands

The system SHALL provide environment variables with slash command metadata to hook commands.

#### Scenario: Standard environment variables

- **WHEN** a slash command hook executes
- **THEN** the following environment variables SHALL be available:
  - `CONCLAUDE_SLASH_COMMAND` - Command name without leading slash
  - `CONCLAUDE_SLASH_COMMAND_ARGS` - Arguments after the command
  - `CONCLAUDE_USER_PROMPT` - Full prompt text
  - `CONCLAUDE_SESSION_ID` - Current session ID
  - `CONCLAUDE_CWD` - Current working directory
  - `CONCLAUDE_HOOK_EVENT` - "UserPromptSubmit"

### Requirement: Environment Variables for Skill Start

The system SHALL provide environment variables with skill metadata to hook commands.

#### Scenario: Skill start environment variables

- **WHEN** a skill start hook executes
- **THEN** the following environment variables SHALL be available:
  - `CONCLAUDE_SKILL_NAME` - The skill/agent type name
  - `CONCLAUDE_AGENT_ID` - Unique agent identifier
  - `CONCLAUDE_SESSION_ID` - Current session ID
  - `CONCLAUDE_TRANSCRIPT_PATH` - Path to session transcript
  - `CONCLAUDE_HOOK_EVENT` - "SubagentStart"

### Requirement: Command Execution Options

Slash command and skill start hooks SHALL support the same command options as other hooks.

#### Scenario: Command with all options

- **GIVEN** a command configuration with:
  ```yaml
  - run: ".claude/scripts/check.sh"
    message: "Check failed"
    showCommand: true
    showStdout: true
    showStderr: false
    maxOutputLines: 100
    timeout: 30
  ```
- **WHEN** the command executes
- **THEN** all options SHALL be respected
- **AND** behavior SHALL match existing hook command execution

#### Scenario: Default command options

- **GIVEN** a command configuration with only `run` specified
- **THEN** `showCommand` SHALL default to true
- **AND** `showStdout` and `showStderr` SHALL default to false
- **AND** `maxOutputLines` SHALL have no limit by default
- **AND** `timeout` SHALL use the default hook timeout

### Requirement: Pattern Precedence

When multiple patterns match a slash command or skill, the system SHALL use consistent precedence rules.

#### Scenario: Exact match takes precedence

- **GIVEN** configurations for patterns `/commit` and `/comm*`
- **WHEN** `/commit` is invoked
- **THEN** only the exact match commands SHALL execute

#### Scenario: More specific pattern wins

- **GIVEN** configurations for patterns `/test*` and `*`
- **WHEN** `/testing` is invoked
- **THEN** only `/test*` commands SHALL execute
- **AND** wildcard `*` SHALL NOT execute

#### Scenario: First matching pattern in order

- **GIVEN** multiple patterns that match equally (e.g., two glob patterns)
- **WHEN** a matching command is invoked
- **THEN** patterns SHALL be matched in configuration order
- **AND** the first matching pattern's commands SHALL execute
