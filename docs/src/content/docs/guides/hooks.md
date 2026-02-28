---
title: Hooks Overview
description: Understanding the conclaude hook system and how to configure each hook type.
---

conclaude integrates with Claude Code through lifecycle hooks—strategic intervention points that let you enforce rules, run validations, and maintain control over your codebase.

## How Hooks Work

When Claude Code runs in a directory with conclaude configured, it invokes conclaude at specific moments:

1. Claude Code fires a hook event (e.g., "PreToolUse")
2. conclaude receives the event as a JSON payload via stdin
3. conclaude evaluates rules and runs commands based on your configuration
4. conclaude returns a result (allow, block, or error)
5. Claude Code proceeds based on the result

## Hook Types

### PreToolUse

**When:** Before Claude uses any tool (Write, Bash, Read, etc.)

**Purpose:** Gate tool execution based on your rules.

**Common Use Cases:**
- Block file creation at project root
- Protect critical files from modification
- Validate tool inputs before execution

**Configuration:**

```yaml
preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "Cargo.lock"
    - ".env*"
    - "target/**"
```

**Example Scenario:**

Claude wants to create `debug.log` in your project root. With `preventRootAdditions: true`, conclaude blocks the operation. Claude adapts and creates `logs/debug.log` instead.

**Real-World Example: Protecting Test Integrity**

During a CSS migration, visual regression tests start failing because pixel measurements have shifted. Claude analyzes the failures and suggests adding tolerance (±10px) to the test assertions to make them pass. This would weaken your test suite—instead of fixing the root cause (CSS layout issues), it would mask them.

**Configuration:**

```yaml
preToolUse:
  uneditableFiles:
    - "visual-regression/**"
```

**Error Output:**

When Claude attempts to modify a protected test file, conclaude blocks the operation:

```text
Blocked Edit operation: file matches preToolUse.uneditableFiles pattern 'visual-regression/**' File: /home/user/project/visual-regression/tests/header-layout.spec.ts
```

**Explanation:**

The pattern `visual-regression/**` matches all files within the `visual-regression` directory and its subdirectories. This glob pattern protects:
- `visual-regression/tests/header-layout.spec.ts`
- `visual-regression/tests/sidebar.spec.ts`
- `visual-regression/snapshots/baseline.png`
- Any other file nested under `visual-regression/`

By preventing Claude from weakening the test assertions, the configuration forces it to address the actual problem: fixing the CSS that caused the layout shifts. This preserves the integrity of pixel-perfect visual regression tests.

**Purpose:**

File protection patterns connect mechanism to purpose. While `preventRootAdditions` keeps your project root clean, `uneditableFiles` enforces your engineering values. In this case, the value is "tests should verify behavior, not accommodate it." When tests fail, fix the code, not the test.

---

### PostToolUse

**When:** After a tool operation completes

**Purpose:** Observe and log tool results.

**Common Use Cases:**
- Audit logging of all changes
- Performance monitoring
- Post-processing validation

**Note:** PostToolUse is currently used for logging and observation. It does not block operations.

---

### Stop

**When:** Claude finishes a task and the session is about to end

**Purpose:** Final validation gate before session completion.

This is the most important hook for enforcing code quality.

**Configuration:**

```yaml
stop:
  commands:
    - run: cargo fmt --check
      message: "Check code formatting"
    - run: cargo clippy -- -D warnings
      message: "Run clippy lints"
    - run: cargo test
      message: "Run test suite"
```

**How It Works:**

1. Claude completes a task
2. conclaude runs each command in order
3. If any command fails, Claude sees the error output
4. Claude can fix the issues
5. Session only completes when all commands pass

**Infinite Mode:**

For long coding sessions, enable continuous validation:

```yaml
stop:
  commands:
    - run: cargo check
    - run: cargo test
  infinite: true
  infiniteMessage: "Monitoring active - validating after each change"
```

With infinite mode, validation runs after every task completion, not just at session end.

**Workflow Guardrails: PR Verification**

Beyond code quality checks, Stop hooks can enforce workflow policies. This example ensures all changes are committed and a pull request exists before the session ends:

```yaml
stop:
  commands:
    # Ensure no uncommitted changes
    - run: |
        if [ -n "$(git status --porcelain)" ]; then
          echo "Error: Uncommitted changes detected"
          git status --short
          exit 1
        fi
      message: "Check for uncommitted changes"

    # Ensure no untracked files
    - run: |
        UNTRACKED=$(git ls-files --others --exclude-standard)
        if [ -n "$UNTRACKED" ]; then
          echo "Error: Untracked files detected:"
          echo "$UNTRACKED"
          exit 1
        fi
      message: "Check for untracked files"

    # Verify PR exists for current branch
    - run: |
        BRANCH=$(git branch --show-current)
        if [ "$BRANCH" = "main" ] || [ "$BRANCH" = "master" ]; then
          exit 0  # Skip PR check on main/master
        fi
        PR_COUNT=$(gh pr list --head "$BRANCH" --json number --jq 'length')
        if [ "$PR_COUNT" -eq 0 ]; then
          echo "Error: No open PR found for branch '$BRANCH'"
          echo "Create a PR with: gh pr create"
          exit 1
        fi
      message: "Verify pull request exists"
```

**Prerequisites:**
- Git must be installed and the directory must be a Git repository
- GitHub CLI (`gh`) must be installed and authenticated for PR verification
- Run `gh auth login` to authenticate if needed

**How It Works:**

1. **Uncommitted changes check**: Uses `git status --porcelain` to detect any staged or unstaged modifications
2. **Untracked files check**: Uses `git ls-files --others --exclude-standard` to find files not yet added to Git
3. **PR verification**: Queries GitHub for open PRs from the current branch using `gh pr list`

**When Checks Fail:**

- If uncommitted changes exist, Claude sees the list of modified files and can commit them
- If untracked files exist, Claude sees the file list and can add them to Git
- If no PR exists, Claude sees instructions to create one with `gh pr create`
- The session only completes when all checks pass

**Combining with Code Quality:**

```yaml
stop:
  commands:
    # Code quality first
    - run: cargo fmt --check
      message: "Check formatting"
    - run: cargo test
      message: "Run tests"

    # Then workflow checks
    - run: test -z "$(git status --porcelain)"
      message: "No uncommitted changes"
    - run: |
        BRANCH=$(git branch --show-current)
        [ "$BRANCH" = "main" ] || gh pr list --head "$BRANCH" --json number --jq 'length' | grep -q '[1-9]'
      message: "PR exists for branch"
```

This pattern ensures code quality AND workflow discipline before session completion.

---

### SessionStart

**When:** A new Claude Code session begins

**Purpose:** Initialize session-specific resources.

**Use Cases:**
- Set up session logging
- Initialize monitoring
- Prepare workspace

---

### UserPromptSubmit

**When:** User submits input to Claude

**Purpose:** Process user prompts and inject context based on prompt content.

**Use Cases:**
- Context injection based on prompt patterns
- Automatically load project-specific guidelines
- Audit logging of user inputs

**Configuration:**

```yaml
userPromptSubmit:
  contextRules:
    - pattern: "sidebar|navigation"
      prompt: |
        Read @.claude/contexts/sidebar.md before making changes.

    - pattern: "auth|login|authentication"
      prompt: |
        Review authentication patterns in @.claude/contexts/auth.md
      caseInsensitive: true
```

**How It Works:**

Context injection allows you to automatically prepend relevant guidelines, documentation, or reminders to Claude's system prompt based on what you're asking about. When your prompt matches a configured pattern, conclaude injects the associated context before Claude processes your request.

**Example Scenario:**

You ask: "How do I update the sidebar component?"

With the configuration above:
1. Pattern `sidebar|navigation` matches "sidebar"
2. conclaude prepends the context from `.claude/contexts/sidebar.md`
3. Claude receives your prompt with additional context about sidebar guidelines
4. Claude responds with knowledge of your project-specific sidebar patterns

**Pattern Matching:**
- Supports regular expressions for flexible matching
- Multiple rules can match and inject context
- Case-sensitive by default (use `caseInsensitive: true` to override)
- File references using `@` syntax are expanded to file contents

**See Also:** [UserPromptSubmit Configuration Reference](/conclaude/reference/config/user-prompt-submit)

---

### SubagentStart

**When:** A Claude subagent (coder, tester, stuck, etc.) begins work

**Purpose:** Track subagent initialization.

**Note:** Agent-specific hooks are now defined in agent frontmatter (`.claude/agents/*.md`) rather than in `settings.json`. Run `conclaude init` to automatically inject hooks into agent files.

**Payload Fields:**
- `agent_id` — Identifier for the subagent (e.g., "coder", "tester")
- `subagent_type` — Category of subagent (e.g., "implementation", "testing")
- `agent_transcript_path` — Path to subagent's transcript file

**Configuration:**

Agent frontmatter hooks are automatically injected by `conclaude init`:

```yaml
---
name: my-agent
hooks:
  PreToolUse:
    - hooks:
        - type: command
          command: "conclaude Hooks PreToolUse --agent my-agent"
  Stop:
    - hooks:
        - type: command
          command: "conclaude Hooks Stop --agent my-agent"
---
```

For notifications in settings.json:

```yaml
notifications:
  enabled: true
  hooks:
    - "SubagentStart"
```

**Use Cases:**
- Log when specific agents start
- Initialize agent-specific resources
- Collect timing metrics

---

### SubagentStop

**When:** A Claude subagent completes its work

**Purpose:** Handle subagent completion.

**Note:** Agent-specific hooks are now defined in agent frontmatter (`.claude/agents/*.md`). The `--agent` flag allows hook handlers to identify which agent is executing, eliminating the need for transcript parsing.

**Payload Fields:**
- `agent_id` — Identifier for the completed subagent
- `agent_transcript_path` — Path to subagent's transcript file

**Configuration:**

See SubagentStart above for agent frontmatter hook injection. For notifications:

```yaml
notifications:
  enabled: true
  hooks:
    - "SubagentStop"
```

**Use Cases:**
- Log subagent completion
- Cleanup operations
- Performance metrics

---

### Notification

**When:** System notifications are sent

**Purpose:** Handle and filter system alerts.

---

### PreCompact

**When:** Before transcript compaction

**Purpose:** Prepare transcripts before archival.

---

### PermissionRequest

**When:** A tool requests permission

**Purpose:** Handle permission flows.

---

### SessionEnd

**When:** A session terminates

**Purpose:** Cleanup and final logging.

---

## Configuration Examples

### Basic Development Setup

```yaml
stop:
  commands:
    - run: cargo check
      message: "Type check"

preToolUse:
  preventRootAdditions: false
  uneditableFiles:
    - "Cargo.toml"
```

### Production Quality Gates

```yaml
stop:
  commands:
    - run: cargo fmt --check
      message: "Formatting must pass"
    - run: cargo clippy -- -D warnings
      message: "No clippy warnings allowed"
    - run: cargo test
      message: "All tests must pass"
    - run: cargo build --release
      message: "Release build must succeed"

preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "Cargo.toml"
    - "Cargo.lock"
    - ".env*"
    - ".github/workflows/**"
```

### Node.js Project

```yaml
stop:
  commands:
    - run: npm run lint
      message: "ESLint check"
    - run: npm run typecheck
      message: "TypeScript check"
    - run: npm test
      message: "Jest tests"

preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "package-lock.json"
    - ".env*"
    - "node_modules/**"
```

### Continuous Monitoring Session

```yaml
stop:
  commands:
    - run: cargo check --quiet
    - run: cargo test --quiet
  infinite: true
  infiniteMessage: "Validating after each change..."

preToolUse:
  preventRootAdditions: false
```

### Context Injection for Project Guidelines

Automatically inject relevant context when working on specific features:

```yaml
userPromptSubmit:
  contextRules:
    # React component guidelines
    - pattern: "component|react|jsx|tsx"
      prompt: |
        Component guidelines: @.claude/contexts/components.md
        - Use TypeScript
        - Add PropTypes
        - Include unit tests
      caseInsensitive: true

    # API development standards
    - pattern: "api|endpoint|route"
      prompt: |
        API standards: @.claude/contexts/api.md
        - Use proper HTTP status codes
        - Implement error handling
        - Add request validation

    # Security reminders
    - pattern: "auth|login|password|security"
      prompt: |
        SECURITY CHECKLIST:
        - Never commit secrets
        - Use environment variables
        - Review: @.claude/contexts/security.md
      caseInsensitive: true

# Optional: Get notified when context is injected
notifications:
  enabled: true
  hooks:
    - "UserPromptSubmit"
  showSuccess: true
```

**How This Helps:**

When you ask "How do I add a new React component?", conclaude automatically reminds Claude about your component guidelines before it responds. You don't need to repeat instructions manually.

## Notifications

Get system notifications when hooks execute:

```yaml
notifications:
  enabled: true
  hooks:
    - "Stop"
    - "PreToolUse"
```

Use `["*"]` for all hooks:

```yaml
notifications:
  enabled: true
  hooks: ["*"]
```

## Environment Variables

Commands executed by hooks have access to context variables:

| Variable | Description |
|----------|-------------|
| `CONCLAUDE_SESSION_ID` | Unique session identifier |
| `CONCLAUDE_TRANSCRIPT_PATH` | Path to session transcript |
| `CONCLAUDE_CWD` | Current working directory |
| `CONCLAUDE_HOOK_EVENT` | Name of executing hook |
| `CONCLAUDE_CONFIG_DIR` | Directory containing config |

For agent-aware hooks (when using `--agent` flag):

| Variable | Description |
|----------|-------------|
| `CONCLAUDE_AGENT_NAME` | Agent name passed via `--agent` flag (e.g., "coder", "tester", "stuck") |

For subagent hooks (payload-based):

| Variable | Description |
|----------|-------------|
| `CONCLAUDE_AGENT_ID` | Agent identifier from payload (e.g., "adb0a8b") |
| `CONCLAUDE_SUBAGENT_TYPE` | Subagent type (e.g., "implementation") (SubagentStart only) |
| `CONCLAUDE_AGENT_TRANSCRIPT_PATH` | Path to subagent transcript |

## Next Steps

- **[CLI Reference](/conclaude/reference/cli)** — All commands and hook handlers
- **[Configuration Reference](/conclaude/reference/config/configuration)** — Complete configuration options
- **[Stop Hook](/conclaude/reference/config/stop)** — Detailed stop hook configuration
- **[PreToolUse Hook](/conclaude/reference/config/pre-tool-use)** — File protection rules
- **[UserPromptSubmit Hook](/conclaude/reference/config/user-prompt-submit)** — Context injection configuration
