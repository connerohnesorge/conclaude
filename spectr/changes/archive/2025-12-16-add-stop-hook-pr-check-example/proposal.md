# Proposal: Add Stop Hook PR Check Example

## Problem

Users working in Git workflows often want to ensure that their work is tracked in a pull request before ending a Claude Code session. Currently, the documentation lacks a practical example showing how to use the Stop hook to verify that the current branch has an open pull request into the main/default branch.

Without this example, users may:
- Forget to create PRs before ending sessions
- Need to manually remember to check PR status
- Lack visibility into a common workflow guardrail pattern

## Proposed Solution

Add a documented example to the Hooks Overview guide (`docs/src/content/docs/guides/hooks.md`) that demonstrates using the Stop hook with GitHub CLI (`gh`) to verify the current branch has an open pull request.

The example will show:
- Using `gh pr list` to check for PRs from the current branch
- Providing a clear error message when no PR exists
- Integrating this check into a typical Stop hook workflow alongside other validation commands

This example serves as a workflow guardrail pattern that teams can adopt to enforce PR creation discipline.

## Why This Change

### User Value
- **Workflow Enforcement**: Prevents accidental session completion without PR creation
- **Team Standards**: Helps teams maintain consistent PR workflows
- **Discoverability**: Shows users how to integrate external tooling (GitHub CLI) into hooks
- **Real-World Pattern**: Demonstrates a practical, production-ready use case

### Documentation Completeness
- Current Stop hook examples focus on code quality (linting, testing, formatting)
- Missing examples of workflow and process enforcement
- Fills gap in demonstrating integration with external tools (gh, git)

## Impact

### Scope
- **Modified**: `docs/src/content/docs/guides/hooks.md` — Add new example section under Stop hook
- **Capability**: `documentation` — Extends existing guide with new example

### Breaking Changes
None. This is purely additive documentation.

### Dependencies
- Requires GitHub CLI (`gh`) to be installed and authenticated
- Example should note this as a prerequisite
- Should mention graceful handling when `gh` is not available

## Open Questions

None. The implementation is straightforward:
1. Add example to existing Stop hook section in hooks.md
2. Include commented YAML showing the configuration
3. Explain how the check works and when it fails
4. Note prerequisites (gh CLI installed and authenticated)

## Success Criteria

- [ ] Example added to hooks.md under Stop hook section
- [ ] Example includes full YAML configuration
- [ ] Example explains the workflow and failure cases
- [ ] Prerequisites clearly stated (gh CLI requirement)
- [ ] Internal links validated during build
- [ ] Documentation builds successfully without errors
