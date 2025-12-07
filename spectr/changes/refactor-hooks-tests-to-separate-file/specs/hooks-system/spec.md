## ADDED Requirements

### Requirement: Test Organization
Unit tests for the hooks module SHALL be organized in a dedicated test file (`src/hooks_tests.rs`) rather than inline within the implementation file.

#### Scenario: Test file structure
- **WHEN** examining the hooks test code
- **THEN** tests SHALL reside in `src/hooks_tests.rs`
- **AND** the test module SHALL be conditionally compiled with `#[cfg(test)]`
- **AND** all existing test coverage SHALL be preserved
