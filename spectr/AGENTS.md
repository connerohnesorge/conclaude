# Spectr Instructions

Instructions for AI coding assistants using Spectr for spec-driven development.

## TL;DR Quick Checklist

- Search existing work: Read `spectr/changes/` and `spectr/specs/` directories (use `rg` for full-text search)
- Decide scope: new capability vs modify existing capability
- Pick a unique `change-id`: kebab-case, verb-led (`add-`, `update-`, `remove-`, `refactor-`)
- Scaffold: `proposal.md`, `tasks.md`, `design.md` (only if needed), and delta specs per affected capability
- Write deltas: use `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`; include at least one `#### Scenario:` per requirement
- Validate: `spectr validate [change-id]` and fix issues
- Request approval: Do not start implementation until proposal is approved

## Two-Stage Workflow

### Stage 1: Creating Changes
Create proposal when you need to:
- Add features or functionality
- Make breaking changes (API, schema)
- Change architecture or patterns
- Optimize performance (changes behavior)
- Update security patterns

Loose matching guidance:
- Contains one of: `proposal`, `change`, `spec`
- With one of: `create`, `plan`, `make`, `start`, `help`

Skip proposal for:
- Bug fixes (restore intended behavior)
- Typos, formatting, comments
- Dependency updates (non-breaking)
- Configuration changes
- Tests for existing behavior

Workflow:
1. Review `spectr/project.md` and read `spectr/specs/` and `spectr/changes/` directories to understand current context.
2. Choose a unique verb-led `change-id` and scaffold `proposal.md`, `tasks.md`, optional `design.md`, and spec deltas under `spectr/changes/<id>/`.
3. Draft spec deltas using `## ADDED|MODIFIED|REMOVED Requirements` with at least one `#### Scenario:` per requirement.
4. Run `spectr validate <id>` and resolve any issues before sharing the proposal.

### Stage 2: Implementing Changes
Track these steps as TODOs and complete them one by one.
1. Read proposal.md - Understand what's being built
2. Read design.md (if exists) - Review technical decisions
3. Read tasks.md - Get implementation checklist
4. Implement tasks sequentially - Complete in order
5. Confirm completion - Ensure every item in `tasks.md` is finished before updating statuses
6. Update checklist - After all work is done, set every task to `- [x]` so the list reflects reality
7. Approval gate - Do not start implementation until the proposal is reviewed and approved

## Before Any Task

Context Checklist:
- [ ] Read relevant specs in `specs/[capability]/spec.md`
- [ ] Check pending changes in `changes/` for conflicts
- [ ] Read `spectr/project.md` for conventions
- [ ] Read `spectr/changes/` directory to see active changes
- [ ] Read `spectr/specs/` directory to see existing capabilities

Before Creating Specs:
- Always check if capability already exists
- Prefer modifying existing specs over creating duplicates
- Read `spectr/specs/<capability>/spec.md` directly to review current state
- If request is ambiguous, ask 1-2 clarifying questions before scaffolding

### Search Guidance
- Enumerate specs: Read `spectr/specs/` directory (or `spectr spec list --long` for formatted output)
- Enumerate changes: Read `spectr/changes/` directory (or `spectr list` for formatted output)
- Read details directly:
  - Spec: Read `spectr/specs/<capability>/spec.md`
  - Change: Read `spectr/changes/<change-id>/proposal.md`
- Full-text search (use ripgrep): `rg -n "Requirement:|Scenario:" spectr/specs`

## Quick Start

### CLI Commands

```bash
# Essential commands
spectr list                  # List active changes
spectr list --specs          # List specifications
spectr validate [item]       # Validate changes or specs

# Project management
spectr init [path]           # Initialize or update instruction files

# Interactive mode
spectr validate              # Bulk validation mode

# Debugging
spectr validate [change]
```

Reading Specs and Changes (for AI agents):
- Specs: Read `spectr/specs/<capability>/spec.md` directly
- Changes: Read `spectr/changes/<change-id>/proposal.md` directly
- Tasks: Read `spectr/changes/<change-id>/tasks.md` directly

### Command Flags

- `--json` - Machine-readable output
- `--type change|spec` - Disambiguate items
- `--no-interactive` - Disable prompts

## Directory Structure

```
spectr/
├── project.md              # Project conventions
├── specs/                  # Current truth - what IS built
│   └── [capability]/       # Single focused capability
│       ├── spec.md         # Requirements and scenarios
│       └── design.md       # Technical patterns
├── changes/                # Proposals - what SHOULD change
│   ├── [change-name]/
│   │   ├── proposal.md     # Why, what, impact
│   │   ├── tasks.md        # Implementation checklist
│   │   ├── design.md       # Technical decisions (optional; see criteria)
│   │   └── specs/          # Delta changes
│   │       └── [capability]/
│   │           └── spec.md # ADDED/MODIFIED/REMOVED
│   └── archive/            # Completed changes
```

## Creating Change Proposals

### Decision Tree

```
New request?
├─ Bug fix restoring spec behavior? → Fix directly
├─ Typo/format/comment? → Fix directly
├─ New feature/capability? → Create proposal
├─ Breaking change? → Create proposal
├─ Architecture change? → Create proposal
└─ Unclear? → Create proposal (safer)
```

### Proposal Structure

1. Create directory: `changes/[change-id]/` (kebab-case, verb-led, unique)

2. Write proposal.md:
```markdown
# Change: [Brief description of change]

## Why
[1-2 sentences on problem/opportunity]

## What Changes
- [Bullet list of changes]
- [Mark breaking changes with BREAKING]

## Impact
- Affected specs: [list capabilities]
- Affected code: [key files/systems]
```

3. Create spec deltas: `specs/[capability]/spec.md`
```markdown
## ADDED Requirements
### Requirement: New Feature
The system SHALL provide...

#### Scenario: Success case
- WHEN user performs action
- THEN expected result

## MODIFIED Requirements
### Requirement: Existing Feature
[Complete modified requirement]

## REMOVED Requirements
### Requirement: Old Feature
Reason: [Why removing]
Migration: [How to handle]
```
If multiple capabilities are affected, create multiple delta files under `changes/[change-id]/specs/<capability>/spec.md`—one per capability.

4. Create tasks.md:
```markdown
## 1. Implementation
- [ ] 1.1 Create database schema
- [ ] 1.2 Implement API endpoint
- [ ] 1.3 Add frontend component
- [ ] 1.4 Write tests
```

5. Create design.md when needed:
Create `design.md` if any of the following apply; otherwise omit it:
- Cross-cutting change (multiple services/modules) or a new architectural pattern
- New external dependency or significant data model changes
- Security, performance, or migration complexity
- Ambiguity that benefits from technical decisions before coding

Minimal `design.md` skeleton:
```markdown
## Context
[Background, constraints, stakeholders]

## Goals / Non-Goals
- Goals: [...]
- Non-Goals: [...]

## Decisions
- Decision: [What and why]
- Alternatives considered: [Options + rationale]

## Risks / Trade-offs
- [Risk] → Mitigation

## Migration Plan
[Steps, rollback]

## Open Questions
- [...]
```

## Spec File Format

### Critical: Scenario Formatting

CORRECT (use #### headers):
```markdown
#### Scenario: User login success
- WHEN valid credentials provided
- THEN return JWT token
```

WRONG (don't use bullets or bold):
```markdown
- Scenario: User login  [NO]
Scenario: User login     [NO]
### Scenario: User login      [NO]
```

Every requirement MUST have at least one scenario.

### Requirement Wording
- Use SHALL/MUST for normative requirements (avoid should/may unless intentionally non-normative)

### Delta Operations

- `## ADDED Requirements` - New capabilities
- `## MODIFIED Requirements` - Changed behavior
- `## REMOVED Requirements` - Deprecated features
- `## RENAMED Requirements` - Name changes

Headers matched with `trim(header)` - whitespace ignored.

#### When to use ADDED vs MODIFIED
- ADDED: Introduces a new capability or sub-capability that can stand alone as a requirement. Prefer ADDED when the change is orthogonal (e.g., adding "Slash Command Configuration") rather than altering the semantics of an existing requirement.
- MODIFIED: Changes the behavior, scope, or acceptance criteria of an existing requirement. Always paste the full, updated requirement content (header + all scenarios). The archiver will replace the entire requirement with what you provide here; partial deltas will drop previous details.
- RENAMED: Use when only the name changes. If you also change behavior, use RENAMED (name) plus MODIFIED (content) referencing the new name.

Common pitfall: Using MODIFIED to add a new concern without including the previous text. This causes loss of detail at archive time. If you aren't explicitly changing the existing requirement, add a new requirement under ADDED instead.

Authoring a MODIFIED requirement correctly:
1) Locate the existing requirement in `spectr/specs/<capability>/spec.md`.
2) Copy the entire requirement block (from `### Requirement: ...` through its scenarios).
3) Paste it under `## MODIFIED Requirements` and edit to reflect the new behavior.
4) Ensure the header text matches exactly (whitespace-insensitive) and keep at least one `#### Scenario:`.

Example for RENAMED:
```markdown
## RENAMED Requirements
- FROM: `### Requirement: Login`
- TO: `### Requirement: User Authentication`
```

## Troubleshooting

### Common Errors

"Change must have at least one delta"
- Check `changes/[name]/specs/` exists with .md files
- Verify files have operation prefixes (## ADDED Requirements)

"Requirement must have at least one scenario"
- Check scenarios use `#### Scenario:` format (4 hashtags)
- Don't use bullet points or bold for scenario headers

Silent scenario parsing failures
- Exact format required: `#### Scenario: Name`
- Debug by reading the delta spec file directly: `spectr/changes/<change-id>/specs/<capability>/spec.md`

### Validation Tips

```bash
# Validate a change (validation is always strict by default)
spectr validate [change]
```

For validation debugging:
- Read delta specs directly: `spectr/changes/<change-id>/specs/<capability>/spec.md`
- Check spec content by reading: `spectr/specs/<capability>/spec.md`

## Happy Path Script

```bash
# 1) Explore current state
spectr spec list --long
spectr list
# Optional full-text search:
# rg -n "Requirement:|Scenario:" spectr/specs
# rg -n "^#|Requirement:" spectr/changes

# 2) Choose change id and scaffold
CHANGE=add-two-factor-auth
mkdir -p spectr/changes/$CHANGE/{specs/auth}
printf "## Why\\n...\\n\\n## What Changes\\n- ...\\n\\n## Impact\\n- ...\\n" > spectr/changes/$CHANGE/proposal.md
printf "## 1. Implementation\\n- [ ] 1.1 ...\\n" > spectr/changes/$CHANGE/tasks.md

# 3) Add deltas (example)
cat > spectr/changes/$CHANGE/specs/auth/spec.md << 'EOF'
## ADDED Requirements
### Requirement: Two-Factor Authentication
Users MUST provide a second factor during login.

#### Scenario: OTP required
- WHEN valid credentials are provided
- THEN an OTP challenge is required
EOF

# 4) Validate
spectr validate $CHANGE
```

## Multi-Capability Example

```
spectr/changes/add-2fa-notify/
├── proposal.md
├── tasks.md
└── specs/
    ├── auth/
    │   └── spec.md   # ADDED: Two-Factor Authentication
    └── notifications/
        └── spec.md   # ADDED: OTP email notification
```

auth/spec.md
```markdown
## ADDED Requirements
### Requirement: Two-Factor Authentication
...
```

notifications/spec.md
```markdown
## ADDED Requirements
### Requirement: OTP Email Notification
...
```

## Best Practices

### Simplicity First
- Default to <100 lines of new code
- Single-file implementations until proven insufficient
- Avoid frameworks without clear justification
- Choose boring, proven patterns

### Complexity Triggers
Only add complexity with:
- Performance data showing current solution too slow
- Concrete scale requirements (>1000 users, >100MB data)
- Multiple proven use cases requiring abstraction

### Clear References
- Use `file.ts:42` format for code locations
- Reference specs as `specs/auth/spec.md`
- Link related changes and PRs

### Capability Naming
- Use verb-noun: `user-auth`, `payment-capture`
- Single purpose per capability
- 10-minute understandability rule
- Split if description needs "AND"

### Change ID Naming
- Use kebab-case, short and descriptive: `add-two-factor-auth`
- Prefer verb-led prefixes: `add-`, `update-`, `remove-`, `refactor-`
- Ensure uniqueness; if taken, append `-2`, `-3`, etc.

## Tool Selection Guide

| Task | Tool | Why |
|------|------|-----|
| Find files by pattern | Glob | Fast pattern matching |
| Search code content | Grep | Optimized regex search |
| Read specific files | Read | Direct file access |
| Explore unknown scope | Task | Multi-step investigation |

## Error Recovery

### Change Conflicts
1. Read `spectr/changes/` directory to see active changes
2. Check for overlapping specs
3. Coordinate with change owners
4. Consider combining proposals

### Validation Failures
1. Run `spectr validate` (validation is always strict)
2. Check JSON output for details
3. Verify spec file format
4. Ensure scenarios properly formatted

### Missing Context
1. Read project.md first
2. Check related specs
3. Review recent archives
4. Ask for clarification

## Quick Reference

### Stage Indicators
- `changes/` - Proposed, not yet built
- `specs/` - Built and deployed
- `archive/` - Completed changes

### File Purposes
- `proposal.md` - Why and what
- `tasks.md` - Implementation steps
- `spec.md` - Requirements and behavior
- `design.md` - Technical decisions (optional)

### CLI Essentials
```bash
spectr list              # What's in progress?
spectr validate          # Is it correct?
spectr archive <change-id> [--yes|-y]  # Mark complete (add --yes for automation)
```

Reading Details (for AI agents):
- Read `spectr/specs/<capability>/spec.md` for spec details
- Read `spectr/changes/<change-id>/proposal.md` for change details

Remember: Specs are truth. Changes are proposals. Keep them in sync.
