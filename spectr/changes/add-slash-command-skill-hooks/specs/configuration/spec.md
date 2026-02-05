# Slash Commands and Skills Hooks - Delta Specs

This change adds support for injecting conclaude hooks into `.claude/commands/*.md` and `.claude/skills/*.md` files, similar to the existing agent hook injection. It also adds skill context support via `--skill` flag and `CONCLAUDE_SKILL` environment variable.

## Affected Capabilities

- `initialization` - Extend init command to handle commands and skills
- `cli-structure` - Add `--skill` flag to hook commands
- `configuration` - Add `skill` field to rule types for skill-specific rules
- `hooks-system` - Add skill context to hook processing

---

## ADDED Requirements

### Requirement: Command File Hook Injection

The `conclaude init` command SHALL discover Slash Command files and inject conclaude hooks into their frontmatter.

#### Scenario: Command files discovered and updated

- **GIVEN** a project with `.claude/commands/` directory containing markdown files
- **WHEN** a user runs `conclaude init`
- **THEN** each command file SHALL be discovered and processed
- **AND** the hooks section SHALL be injected into frontmatter
- **AND** each hook SHALL call `conclaude Hooks <type> --skill <name>`

#### Scenario: Skill name extracted from frontmatter

- **GIVEN** a command file with a `name` field in frontmatter
- **WHEN** hooks are injected
- **THEN** the hooks SHALL use that name value with `--skill` flag
- **AND** the command format SHALL be `conclaude Hooks <type> --skill <name>`

#### Scenario: Skill name derived from filename

- **GIVEN** a command file without a `name` field in frontmatter
- **WHEN** hooks are injected
- **THEN** the name SHALL be derived from the filename (without .md extension)
- **AND** a warning SHALL be logged suggesting adding a name field

#### Scenario: Existing hooks preserved in commands

- **GIVEN** a command file already has a `hooks` section
- **WHEN** conclaude injects hooks
- **THEN** conclaude-generated hooks SHALL be merged
- **AND** user-defined hooks SHALL NOT be overwritten

---

### Requirement: Skill File Hook Injection

The `conclaude init` command SHALL discover Skill files and inject conclaude hooks into their frontmatter.

#### Scenario: Skill files discovered and updated

- **GIVEN** a project with `.claude/skills/` directory containing markdown files
- **WHEN** a user runs `conclaude init`
- **THEN** each skill file SHALL be discovered and processed
- **AND** the hooks section SHALL be injected into frontmatter
- **AND** each hook SHALL call `conclaude Hooks <type> --skill <name>`

#### Scenario: Skill name extracted from frontmatter

- **GIVEN** a skill file with a `name` field in frontmatter
- **WHEN** hooks are injected
- **THEN** the hooks SHALL use that name value with `--skill` flag

#### Scenario: Skill name derived from filename

- **GIVEN** a skill file without a `name` field in frontmatter
- **WHEN** hooks are injected
- **THEN** the name SHALL be derived from the filename (without .md extension)

---

### Requirement: CLI Skill Flag Support

All hook commands SHALL support a `--skill` flag for skill context.

#### Scenario: PreToolUse with --skill flag

- **GIVEN** the PreToolUse hook command
- **WHEN** invoked with `--skill my-skill`
- **THEN** the `CONCLAUDE_SKILL` environment variable SHALL be set to "my-skill"

#### Scenario: Stop hook with --skill flag

- **GIVEN** the Stop hook command
- **WHEN** invoked with `--skill tester`
- **THEN** the skill context SHALL be available in hook processing

#### Scenario: All hooks support --skill

- **GIVEN** all hook commands (PreToolUse, PostToolUse, Stop, SessionStart, SessionEnd, Notification, PreCompact, PermissionRequest, UserPromptSubmit, SubagentStart, SubagentStop)
- **WHEN** checking CLI arguments
- **THEN** each SHALL accept an optional `--skill <name>` argument

---

### Requirement: Skill Environment Variable

The system SHALL provide skill context to hook handlers via environment variable.

#### Scenario: CONCLAUDE_SKILL set during execution

- **GIVEN** a hook command invoked with `--skill my-skill`
- **WHEN** the hook handler executes
- **THEN** `CONCLAUDE_SKILL` environment variable SHALL be set to "my-skill"

#### Scenario: CONCLAUDE_SKILL absent when not specified

- **GIVEN** a hook command invoked without `--skill` flag
- **WHEN** the hook handler executes
- **THEN** `CONCLAUDE_SKILL` environment variable SHALL NOT be set

---

### Requirement: Skill Field in Configuration Rules

Rule types SHALL support a `skill` field for skill-specific rule scoping.

#### Scenario: Uneditable file rule with skill

- **GIVEN** a `.conclaude.yaml` configuration:
  ```yaml
  preToolUse:
    uneditableFiles:
      - pattern: "tests/**"
        skill: "test*"
        message: "Test files managed by testing skills"
  ```
- **WHEN** a tool attempts to edit a test file
- **AND** `CONCLAUDE_SKILL` matches "test*" pattern
- **THEN** the rule SHALL apply and block the edit

#### Scenario: Tool usage rule with skill

- **GIVEN** a `.conclaude.yaml` configuration:
  ```yaml
  preToolUse:
    toolUsageValidation:
      - tool: "Write"
        pattern: "docs/**"
        skill: "doc*"
        action: "block"
  ```
- **WHEN** a documentation skill attempts to write outside docs/
- **THEN** the rule SHALL apply and block the operation

#### Scenario: Stop command with skill

- **GIVEN** a `.conclaude.yaml` configuration:
  ```yaml
  stop:
    commands:
      - run: "cargo test --lib"
        skill: "tester"
        message: "Library tests must pass"
  ```
- **WHEN** the tester skill completes work
- **THEN** only skill-specific commands SHALL execute

---

### Requirement: Skill Pattern Matching

The system SHALL support glob pattern matching for skill names.

#### Scenario: Exact skill match

- **GIVEN** a rule with `skill: "tester"`
- **WHEN** `CONCLAUDE_SKILL` is "tester"
- **THEN** the rule SHALL match

#### Scenario: Prefix glob pattern

- **GIVEN** a rule with `skill: "test*"`
- **WHEN** `CONCLAUDE_SKILL` is "tester" or "test-runner"
- **THEN** the rule SHALL match both

#### Scenario: Wildcard pattern

- **GIVEN** a rule with `skill: "*"`
- **WHEN** any skill is active
- **THEN** the rule SHALL match

#### Scenario: No match

- **GIVEN** a rule with `skill: "doc*"`
- **WHEN** `CONCLAUDE_SKILL` is "tester"
- **THEN** the rule SHALL NOT match

---

## MODIFIED Requirements

### Requirement: Auto-Generated Field Lists for Error Suggestions

The auto-generated field lists SHALL include the new `skill` field.

#### Scenario: skill field in suggestions

- **WHEN** a user misconfigures a skill field (e.g., `skills` instead of `skill`)
- **THEN** error suggestions SHALL include "skill" as a valid field
- **AND** the suggestion SHALL apply to relevant rule types

