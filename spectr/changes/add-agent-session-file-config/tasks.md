## 1. Configuration

- [ ] 1.1 Add `preserveAgentSessionFiles` boolean field to `SubagentStopConfig` struct in `src/config.rs`
- [ ] 1.2 Add serde default to `false` for backward compatibility
- [ ] 1.3 Add field documentation and JSON schema annotation

## 2. Implementation

- [ ] 2.1 Update `cleanup_agent_session_file()` in `src/hooks.rs` to return `Result<(), io::Error>` instead of silently ignoring errors
- [ ] 2.2 Update `handle_subagent_stop()` to:
  - Read `preserveAgentSessionFiles` from config
  - Skip cleanup if `preserveAgentSessionFiles: true`
  - Log warning on cleanup failure (not file-not-found)
  - Continue hook execution regardless of cleanup outcome
- [ ] 2.3 Handle file-not-found as success case (not an error)

## 3. Testing

- [ ] 3.1 Add unit test: cleanup with `preserveAgentSessionFiles: true` does not delete file
- [ ] 3.2 Add unit test: cleanup with `preserveAgentSessionFiles: false` deletes file
- [ ] 3.3 Add unit test: cleanup error is logged but does not fail hook
- [ ] 3.4 Add unit test: file-not-found during cleanup is not logged as error

## 4. Documentation

- [ ] 4.1 Update default-config.yaml with `preserveAgentSessionFiles` example
- [ ] 4.2 Regenerate JSON schema to include new field
