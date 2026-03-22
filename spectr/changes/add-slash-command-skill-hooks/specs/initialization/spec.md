# Slash Commands and Skills Hooks - Initialization Delta Spec

This delta spec extends the initialization capability to support commands and skills hook injection.

---

## MODIFIED Requirements

### Requirement: Agent Frontmatter Hook Injection

The existing agent hook injection SHALL remain unchanged while command/skill injection is added.

#### Scenario: Agents still work independently

- **GIVEN** agent files with existing hooks using `--agent` flag
- **WHEN** `conclaude init` runs
- **THEN** agent hooks SHALL NOT be affected by skill injection
- **AND** agent-specific rules SHALL use `CONCLAUDE_AGENT` env var
- **AND** skill-specific rules SHALL use `CONCLAUDE_SKILL` env var
