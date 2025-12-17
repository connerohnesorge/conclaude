# documentation Specification Delta

## ADDED Requirements

### Requirement: Stop Hook PR Verification Example

The Hooks Overview guide SHALL include an example demonstrating how to use the Stop hook to verify that the current branch has an open pull request.

#### Scenario: User finds PR verification example

- **WHEN** a user reads the Stop hook section in the Hooks Overview guide
- **THEN** they SHALL find a subsection titled "Workflow Guardrails: PR Verification"
- **AND** the subsection SHALL explain the purpose of using Stop hooks for PR verification

#### Scenario: Example includes working YAML configuration

- **WHEN** a user reads the PR verification example
- **THEN** they SHALL find a YAML code block showing Stop hook configuration
- **AND** the configuration SHALL use `gh pr list --head $(git branch --show-current) --json number` or equivalent
- **AND** the configuration SHALL include a `message` field explaining what failed
- **AND** the YAML SHALL be syntactically valid

#### Scenario: Prerequisites are documented

- **WHEN** a user reads the PR verification example
- **THEN** they SHALL find a note about required prerequisites
- **AND** the prerequisites SHALL mention GitHub CLI (`gh`) must be installed
- **AND** the prerequisites SHALL mention `gh` must be authenticated
- **AND** the note SHALL explain what happens if `gh` is not available

#### Scenario: Example explains failure behavior

- **WHEN** a user reads the PR verification example
- **THEN** they SHALL find an explanation of when the check fails
- **AND** the explanation SHALL describe the error message displayed to Claude
- **AND** the explanation SHALL clarify that Claude can fix issues by creating a PR

#### Scenario: Example shows integration with other checks

- **WHEN** a user reads the PR verification example
- **THEN** they SHALL see how to combine PR verification with other Stop commands
- **AND** the example SHALL demonstrate ordering (e.g., run linting/tests first, then PR check)

#### Scenario: Command syntax is correct

- **WHEN** the PR verification example is tested manually
- **THEN** the `gh pr list` command SHALL correctly identify PRs for the current branch
- **AND** the command SHALL exit with non-zero status when no PR exists
- **AND** the command SHALL work on both GitHub and GitHub Enterprise (if authenticated)
