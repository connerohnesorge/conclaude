# Tasks: Add Stop Hook PR Check Example

## Implementation Tasks

### 1. Add PR Check Example to Hooks Guide
- [ ] 1.1 Open `docs/src/content/docs/guides/hooks.md` and locate Stop hook section
- [ ] 1.2 Add new subsection titled "**Workflow Guardrails: PR Verification**"
- [ ] 1.3 Write example YAML configuration using `gh pr list --head $(git branch --show-current) --json number`
- [ ] 1.4 Include clear explanation of what the check does and when it fails
- [ ] 1.5 Document prerequisites (GitHub CLI installed and authenticated)
- [ ] 1.6 Show example error message when no PR exists
- [ ] 1.7 Add note about combining with other Stop commands (linting, testing)

### 2. Validation
- [ ] 2.1 Run `npm run build` in docs/ directory to ensure build succeeds
- [ ] 2.2 Verify internal links are valid (starlight-links-validator)
- [ ] 2.3 Review rendered output in browser to ensure formatting is correct
- [ ] 2.4 Test the example command manually to verify it works as expected

### 3. Spectr Validation
- [ ] 3.1 Run `spectr validate add-stop-hook-pr-check-example --strict`
- [ ] 3.2 Resolve any validation issues
- [ ] 3.3 Confirm all requirements have corresponding scenarios

## Definition of Done

- Example is added to hooks.md in the Stop hook section
- Documentation builds without errors
- Internal link validation passes
- Example YAML is syntactically correct
- Prerequisites are clearly stated
- Spectr validation passes with `--strict` flag
