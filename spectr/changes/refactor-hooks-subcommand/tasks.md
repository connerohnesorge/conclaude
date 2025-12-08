## 1. CLI Structure Refactoring

- [ ] 1.1 Create `HooksCommands` enum with all 11 hook variants (PreToolUse, PostToolUse, PermissionRequest, Notification, UserPromptSubmit, SessionStart, SessionEnd, Stop, SubagentStart, SubagentStop, PreCompact)
- [ ] 1.2 Add `Hooks` variant to top-level `Commands` enum with nested subcommand
- [ ] 1.3 Remove hook variants from top-level `Commands` enum
- [ ] 1.4 Update `main()` match arm to delegate `Commands::Hooks` to `HooksCommands` handling

## 2. Init Command Update

- [ ] 2.1 Update `handle_init()` to generate hook commands with `hooks` prefix (e.g., `conclaude hooks PreToolUse`)
- [ ] 2.2 Update help text in CLI to reflect new command structure

## 3. Testing

- [ ] 3.1 Verify all hooks work via new command structure (`conclaude hooks <HookName>`)
- [ ] 3.2 Run `conclaude Init` and verify generated `.claude/settings.json` uses new hook command format
- [ ] 3.3 Run existing tests to confirm no regressions

## 4. Documentation

- [ ] 4.1 Update README/documentation with new command structure
