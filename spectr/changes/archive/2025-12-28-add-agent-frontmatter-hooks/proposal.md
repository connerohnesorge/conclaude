# Change: Add Agent Frontmatter Hooks

## Why

The current SubagentStart/SubagentStop hook implementation is brittle:
- Complex transcript parsing to extract agent names (150+ lines)
- Session file persistence for agent context tracking (race conditions, temp file accumulation)
- Pattern matching logic that fails silently on errors

Claude Code now supports hooks directly in agent frontmatter (per Boris Cherny's tweet), enabling native per-agent hook execution without transcript parsing.

## What Changes

- **ADDED**: `--agent <name>` flag to all hook CLI commands
- **ADDED**: Agent frontmatter injection in `conclaude init`
- **REMOVED**: SubagentStart/SubagentStop hooks from settings.json generation
- **REMOVED**: Transcript parsing code for agent name extraction
- **REMOVED**: Session file persistence for agent context
- **BREAKING**: SubagentStart/SubagentStop hooks no longer needed in settings.json

## Impact

- Affected specs: initialization, cli-structure, subagent-hooks
- Affected code: src/main.rs, src/hooks.rs, src/init.rs
- Migration: Run `conclaude init` to update agent files with frontmatter hooks
