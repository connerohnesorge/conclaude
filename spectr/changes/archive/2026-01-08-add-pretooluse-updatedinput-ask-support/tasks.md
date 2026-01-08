# Implementation Tasks

## Phase 1: Type System Updates

### Task 1.1: Add updated_input field to HookResult
- **Description**: Add optional `updated_input` field to HookResult struct
- **Location**: `src/types.rs` (HookResult struct)
- **Implementation**:
  - Add field: `pub updated_input: Option<HashMap<String, serde_json::Value>>`
  - Include doc comment explaining the field's purpose
  - Update `success()`, `blocked()`, and `with_context()` constructors to set `updated_input: None`
- **Validation**: Code compiles without errors

### Task 1.2: Add decision field to HookResult
- **Description**: Add optional `decision` field for explicit permission decisions
- **Location**: `src/types.rs` (HookResult struct)
- **Implementation**:
  - Add field: `pub decision: Option<String>`
  - Include doc comment explaining valid values: "allow", "deny", "ask"
  - Update constructors to set `decision: None`
- **Validation**: Code compiles without errors

### Task 1.3: Add HookResult constructor for ask with updated_input
- **Description**: Create helper method for creating ask response with modified input
- **Location**: `src/types.rs` (HookResult impl)
- **Implementation**:
  - Add method: `pub fn ask_with_input(reason: impl Into<String>, updated_input: HashMap<String, serde_json::Value>) -> Self`
  - Sets decision to "ask", message to reason, and updated_input to provided value
- **Validation**: Method can be called and returns correct struct

## Phase 2: Hook Handler Updates

### Task 2.1: Update PreToolUse handler to support ask decision
- **Description**: Modify handle_pre_tool_use to recognize and handle ask decisions
- **Location**: `src/hooks.rs` (handle_pre_tool_use function)
- **Implementation**:
  - When returning HookResult, check if decision is "ask"
  - Serialize updated_input in response if present
  - Ensure backward compatibility with existing blocked/success flow
- **Validation**: PreToolUse hook correctly outputs ask decision with updated_input

### Task 2.2: Update handle_hook_result to support ask decision
- **Description**: Modify result handler to process ask decisions correctly
- **Location**: `src/hooks.rs` (handle_hook_result function)
- **Implementation**:
  - Add case for decision = "ask" (different from blocked)
  - Output JSON with decision, message, and updated_input fields
  - Exit with appropriate code (0 for ask, different from blocked exit code 2)
- **Validation**: Ask decisions produce correct JSON output

## Phase 3: Testing

### Task 3.1: Test HookResult serialization with updated_input
- **Description**: Verify HookResult serializes correctly with updated_input field
- **Location**: `src/types.rs` (in #[cfg(test)] module)
- **Implementation**:
  - Create HookResult with updated_input containing command modification
  - Serialize to JSON
  - Assert JSON contains "updated_input" with expected values
- **Validation**: Test passes

### Task 3.2: Test HookResult serialization with decision field
- **Description**: Verify HookResult serializes correctly with decision field
- **Location**: `src/types.rs` (in #[cfg(test)] module)
- **Implementation**:
  - Create HookResult with decision = "ask"
  - Serialize to JSON
  - Assert JSON contains "decision": "ask"
- **Validation**: Test passes

### Task 3.3: Test ask_with_input constructor
- **Description**: Verify the new constructor creates correct HookResult
- **Location**: `src/types.rs` (in #[cfg(test)] module)
- **Implementation**:
  - Call ask_with_input with reason and modified command
  - Assert decision is "ask"
  - Assert message contains reason
  - Assert updated_input contains modified values
- **Validation**: Test passes

### Task 3.4: Test backward compatibility
- **Description**: Verify existing hook results still work correctly
- **Location**: `src/types.rs` (in #[cfg(test)] module)
- **Implementation**:
  - Verify HookResult::success() still works
  - Verify HookResult::blocked() still works
  - Verify HookResult::with_context() still works
  - Assert new fields are None in these cases
- **Validation**: All existing tests pass

## Phase 4: Validation and Documentation

### Task 4.1: Run cargo test
- **Description**: Verify all tests pass including new updated_input tests
- **Command**: `cargo test`
- **Validation**: All tests pass

### Task 4.2: Run cargo clippy
- **Description**: Verify no new linting warnings introduced
- **Command**: `cargo clippy -- -D warnings`
- **Validation**: No warnings

### Task 4.3: Update inline documentation
- **Description**: Verify doc comments accurately describe new fields
- **Location**: `src/types.rs` (HookResult struct)
- **Implementation**:
  - Document updated_input field and its purpose
  - Document decision field and valid values
  - Document ask_with_input constructor usage
- **Validation**: Documentation is clear and accurate

### Task 4.4: Validate with spectr
- **Description**: Run spectr validation to ensure proposal is properly formed
- **Command**: `spectr validate add-pretooluse-updatedinput-ask-support`
- **Validation**: No validation errors

## Task Dependencies
- Task 1.2 can run in parallel with 1.1
- Task 1.3 requires completion of 1.1 and 1.2
- Phase 2 requires completion of Phase 1
- Phase 3 requires completion of Phase 2
- Phase 4 requires completion of Phase 3

## Success Criteria
- [ ] updated_input field added to HookResult
- [ ] decision field added to HookResult
- [ ] ask_with_input constructor added
- [ ] PreToolUse handler supports ask with updated_input
- [ ] All new tests pass
- [ ] cargo test passes
- [ ] cargo clippy passes with no warnings
- [ ] spectr validate passes
