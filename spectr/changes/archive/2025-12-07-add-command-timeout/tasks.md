## 1. Configuration Structure Updates
- [x] 1.1 Add optional `timeout` field to `StopCommand` struct in `src/config.rs`
- [x] 1.2 Update JSON schema generation to include timeout field
- [x] 1.3 Add validation logic for timeout field values during config loading

## 2. Command Execution Implementation
- [x] 2.1 Modify `execute_stop_commands` function in `src/hooks.rs` to handle timeouts
- [x] 2.2 Implement timeout logic using `tokio::time::timeout` for command execution
- [x] 2.3 Add process termination handling when timeout is exceeded
- [x] 2.4 Update `StopCommandConfig` struct to include timeout information
- [x] 2.5 Modify command collection logic to pass timeout values through

## 3. Error Handling and Messaging
- [x] 3.1 Create timeout-specific error messages
- [x] 3.2 Update logging to include timeout events
- [x] 3.3 Ensure proper hook result blocking when timeout occurs
- [x] 3.4 Add context to timeout error messages (duration, command)

## 4. Testing and Validation
- [x] 4.1 Write unit tests for timeout configuration parsing
- [x] 4.2 Write integration tests for command execution with timeouts
- [x] 4.3 Test backward compatibility with existing configurations
- [x] 4.4 Test timeout scenarios with long-running commands
- [x] 4.5 Test validation of invalid timeout values

## 5. Documentation
- [x] 5.1 Update default configuration template to show timeout field
- [x] 5.2 Add examples to documentation showing timeout usage
- [x] 5.3 Update schema documentation to reflect timeout field

## 6. Validation
- [x] 6.1 Run existing tests to ensure no regressions
- [x] 6.2 Run new timeout-specific tests
- [x] 6.3 Validate configuration loading with timeout values
- [x] 6.4 Test with various timeout scenarios (short, long, none)
