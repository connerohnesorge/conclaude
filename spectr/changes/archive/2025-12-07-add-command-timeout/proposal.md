## Why

Commands in hook configurations currently have no timeout protection, which can lead to processes hanging indefinitely and blocking hook execution. Adding timeout functionality will provide better control over command execution and prevent system hangs.

## What Changes

- Add optional `timeout` field to `StopCommand` configuration structure
- Implement timeout handling in command execution logic
- Add proper error reporting when timeout occurs
- Maintain backward compatibility with existing configurations

## Impact

- Affected specs: Configuration, Command Execution
- Affected code: `src/config.rs` (StopCommand struct), `src/hooks.rs` (execute_stop_commands function)
- Breaking change: No (optional field with backward compatibility)