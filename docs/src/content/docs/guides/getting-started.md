---
title: Getting Started
description: Install conclaude and configure your first project with guardrails for Claude Code sessions.
---

This guide walks you through installing conclaude, creating your first configuration, and running your first guarded Claude Code session.

## Prerequisites

- A terminal (bash, zsh, PowerShell)
- A project directory where you want to use Claude Code
- Claude Code CLI installed

## Installation

The fastest way to install conclaude is via the shell installer:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.sh | sh
```

For other installation methods, see the [Installation guide](/conclaude/guides/installation).

Verify the installation:

```bash
conclaude --version
```

## Initialize Your Project

Navigate to your project directory and run:

```bash
conclaude init
```

This creates two files:

1. **`.conclaude.yaml`** — Your project's guardrail configuration
2. **`.claude/settings.json`** — Claude Code hook configuration (updated if it exists)

## Your First Configuration

Open `.conclaude.yaml` in your editor. The default configuration includes:

```yaml
# .conclaude.yaml
stop:
  commands:
    - run: echo "Conclaude stop hook executed"
      message: "Stop hook completed"

preToolUse:
  preventRootAdditions: false
  uneditableFiles: []
```

Let's make it useful. Here's a configuration for a Rust project:

```yaml
# .conclaude.yaml
stop:
  commands:
    - run: cargo fmt --check
      message: "Check code formatting"
    - run: cargo clippy -- -D warnings
      message: "Run clippy lints"
    - run: cargo test
      message: "Run test suite"

preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "Cargo.lock"
    - ".env*"
```

For a Node.js project:

```yaml
# .conclaude.yaml
stop:
  commands:
    - run: npm run lint
      message: "Run linting"
    - run: npm test
      message: "Run tests"

preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "package-lock.json"
    - ".env*"
```

## Validate Your Configuration

Before starting a Claude Code session, validate your configuration:

```bash
conclaude validate
```

You should see:

```
🔍 Validating conclaude configuration...
✅ Configuration is valid!
   Config file: /path/to/project/.conclaude.yaml
```

If there are errors, the output will explain what needs to be fixed.

## How It Works

When you start a Claude Code session in your project, conclaude hooks into the lifecycle:

1. **PreToolUse** — Before Claude uses any tool (Write, Bash, etc.), conclaude checks:
   - Is the target file protected?
   - Is Claude trying to add a file to the project root when blocked?

2. **Stop** — When Claude finishes a task, conclaude runs your validation commands:
   - If any command fails, Claude sees the error and can fix it
   - The session only completes when all checks pass

## Example Session

Start a Claude Code session normally:

```bash
claude
```

Ask Claude to make changes:

```
> Add a new utility function to parse configuration files
```

When Claude finishes, conclaude automatically:

1. Runs `cargo fmt --check` (or your configured linter)
2. Runs `cargo clippy` (or your configured static analysis)
3. Runs `cargo test` (or your test suite)

If any check fails, Claude sees the output and can fix the issues before the session ends.

## Next Steps

- **[Installation](/conclaude/guides/installation)** — All installation methods including npm, Nix, and building from source
- **[Hooks Overview](/conclaude/guides/hooks)** — Deep dive into the hook system and all available hooks
- **[CLI Reference](/conclaude/reference/cli)** — Complete command reference
- **[Configuration Reference](/conclaude/reference/config/configuration)** — All configuration options explained
