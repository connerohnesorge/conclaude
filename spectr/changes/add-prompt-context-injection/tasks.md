# Tasks: Add Prompt Context Injection

## 1. Configuration Layer

- [ ] 1.1 Add `ContextInjectionRule` struct in `src/config.rs` with fields: `pattern`, `prompt`, `enabled`, `case_insensitive`
- [ ] 1.2 Add `UserPromptSubmitConfig` struct with `context_rules: Vec<ContextInjectionRule>`
- [ ] 1.3 Add `user_prompt_submit` field to `ConclaudeConfig` struct
- [ ] 1.4 Implement `FieldList` derive for new config structs
- [ ] 1.5 Add config validation for regex patterns (fail fast on invalid regex)
- [ ] 1.6 Update `suggest_similar_fields()` to include new section

## 2. Schema Updates

- [ ] 2.1 Regenerate `conclaude-schema.json` with new definitions
- [ ] 2.2 Add comprehensive documentation comments to config structs
- [ ] 2.3 Update `src/default-config.yaml` with commented examples

## 3. Hook Handler Implementation

- [ ] 3.1 Extend `HookResult` struct with optional `system_prompt` field in `src/types.rs`
- [ ] 3.2 Update `handle_user_prompt_submit()` in `src/hooks.rs` to:
  - Load and cache compiled regex patterns
  - Match prompt against all enabled rules
  - Collect and concatenate matching contexts
  - Return augmented `HookResult` with `system_prompt`
- [ ] 3.3 Add helper function `expand_file_references()` to resolve `@file` syntax
- [ ] 3.4 Add helper function `compile_context_rules()` for regex compilation

## 4. Testing

- [ ] 4.1 Add unit tests for `ContextInjectionRule` parsing
- [ ] 4.2 Add unit tests for regex pattern matching
- [ ] 4.3 Add unit tests for file reference expansion
- [ ] 4.4 Add unit tests for multiple pattern matches
- [ ] 4.5 Add integration test with sample config
- [ ] 4.6 Add test for invalid regex handling
- [ ] 4.7 Add test for missing file reference handling

## 5. Documentation

- [ ] 5.1 Update `docs/src/content/docs/reference/config/` with new section documentation
- [ ] 5.2 Add example configurations to docs
- [ ] 5.3 Update `docs/src/content/docs/guides/hooks.md` with context injection guide

## Dependencies

- Tasks 1.x must complete before 2.x (config before schema)
- Tasks 1.x and 3.1 must complete before 3.2-3.4
- All implementation tasks must complete before 4.x (tests)
- All tasks must complete before 5.x (documentation)

## Parallelizable Work

- 1.1-1.4 can be done in parallel
- 3.3 and 3.4 can be done in parallel
- 4.1-4.6 can be done in parallel after 3.x completes
