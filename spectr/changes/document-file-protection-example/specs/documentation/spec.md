# documentation Specification Delta

## ADDED Requirements

### Requirement: File Protection Real-World Example in README

The README.md SHALL include a concrete example demonstrating how `uneditableFiles` pattern blocking enforces project values beyond simple file protection.

#### Scenario: User finds file protection scenario in README

- **WHEN** a user reads the "Real-World Scenarios" section in README.md
- **THEN** they SHALL find a scenario titled "The Test Integrity Enforcer"
- **AND** the scenario SHALL appear after "Scenario 2" and before "Scenario 3"
- **AND** the scenario SHALL follow the existing format (The Problem, The conclaude Solution, What Happens, The Result)

#### Scenario: Scenario explains the problem context

- **WHEN** a user reads the file protection scenario
- **THEN** the "The Problem" section SHALL explain visual regression test integrity
- **AND** it SHALL describe Claude's attempted action (adding tolerance to failing tests)
- **AND** it SHALL explain why tolerance defeats the purpose of pixel-perfect visual testing

#### Scenario: Configuration example is provided

- **WHEN** a user reads the solution section
- **THEN** they SHALL find a YAML configuration code block
- **AND** the configuration SHALL include `uneditableFiles: ["visual-regression/**"]`
- **AND** the pattern SHALL be explained in context

#### Scenario: Actual error output is shown

- **WHEN** demonstrating what happens when protection triggers
- **THEN** a bash code block SHALL show the actual conclaude error message
- **AND** the error message SHALL include:
  - Hook identification: PreToolUse hook error
  - Reason: PreToolUse blocked by preToolUse.uneditableFiles pattern
  - Details: tool_name, file_path, and matching pattern
  - Blocked operation message with file path
- **AND** the error output SHALL match actual conclaude PreToolUse hook format

#### Scenario: Outcome emphasizes value enforcement

- **WHEN** a user reads the result section
- **THEN** it SHALL explain that test integrity was preserved
- **AND** it SHALL state that Claude must fix the underlying CSS issue
- **AND** it SHALL connect file protection to enforcing project values (not just preventing edits)

### Requirement: File Protection Example in Hooks Guide

The Hooks Overview guide SHALL include an expanded real-world example in the PreToolUse section demonstrating file protection for test integrity.

#### Scenario: PreToolUse section has real-world example

- **WHEN** a user reads docs/src/content/docs/guides/hooks.md
- **THEN** the PreToolUse section SHALL include a subsection titled "Real-World Example: Protecting Test Integrity"
- **AND** the subsection SHALL appear after the basic "Example Scenario"
- **AND** content SHALL use proper markdown formatting

#### Scenario: Example shows complete error message

- **WHEN** a user reads the real-world example
- **THEN** they SHALL see the error message in a bash code block
- **AND** the error SHALL show the complete format:
  - Update(visual-regression/compare.ts) header
  - Error: PreToolUse:Edit hook error line
  - [conclaude Hooks PreToolUse] identification
  - Blocked operation with pattern and file path
- **AND** the error SHALL demonstrate pattern matching with `visual-regression/**`

#### Scenario: Example connects mechanism to purpose

- **WHEN** a user reads the explanation
- **THEN** it SHALL explain what Claude attempted to do
- **AND** it SHALL explain why that approach is problematic
- **AND** it SHALL explain how the pattern `visual-regression/**` matches nested files
- **AND** it SHALL connect the technical mechanism to the value being enforced

#### Scenario: Example is educational and actionable

- **WHEN** a user finishes reading the example
- **THEN** they SHALL understand when file protection enforces values (not just prevents mistakes)
- **AND** they SHALL see how to apply similar patterns to their projects
- **AND** they SHALL understand glob pattern syntax for directory recursion (`**`)

### Requirement: Documentation Consistency

The new examples SHALL maintain consistency with existing documentation style, format, and terminology.

#### Scenario: Formatting matches existing patterns

- **WHEN** code blocks are added
- **THEN** YAML configuration SHALL use ```yaml fencing
- **AND** bash/error output SHALL use ```bash fencing
- **AND** inline code references SHALL use single backticks
- **AND** section headers SHALL match the existing pattern

#### Scenario: Terminology is consistent

- **WHEN** examples reference file protection features
- **THEN** terms like `uneditableFiles`, `PreToolUse`, and `pattern` SHALL match existing docs
- **AND** glob pattern syntax explanations SHALL align with configuration reference
- **AND** no conflicting statements about feature behavior SHALL be introduced

#### Scenario: Error message format is accurate

- **WHEN** showing error output
- **THEN** the message structure SHALL match actual conclaude PreToolUse output
- **AND** field names SHALL be accurate (tool_name, file_path, pattern)
- **AND** the pattern `visual-regression/**` SHALL be valid glob syntax
- **AND** pattern behavior SHALL be correctly described
