## ADDED Requirements

### Requirement: Configurable Root Additions Block Message

The system SHALL allow users to configure a custom error message displayed when `preventRootAdditions` blocks file creation at the repository root. The custom message SHALL support template variable substitution for dynamic content.

#### Scenario: Custom message configured with variables
- **GIVEN** configuration contains `preventRootAdditionsMessage: "Files must go in src/. Cannot create {file_path} using {tool}."`
- **WHEN** Claude attempts to create a new file at repository root
- **THEN** the system SHALL block the operation
- **AND** the error message SHALL display: "Files must go in src/. Cannot create newfile.txt using Write."
- **AND** `{file_path}` SHALL be replaced with the attempted file path
- **AND** `{tool}` SHALL be replaced with the tool name (Write)

#### Scenario: Custom message configured without variables
- **GIVEN** configuration contains `preventRootAdditionsMessage: "Please place files in the src/ directory."`
- **WHEN** Claude attempts to create a new file at repository root
- **THEN** the error message SHALL display the exact configured message without modification

#### Scenario: No custom message configured (default behavior)
- **GIVEN** configuration does NOT contain `preventRootAdditionsMessage` OR it is set to `null`
- **WHEN** Claude attempts to create a new file at repository root
- **THEN** the system SHALL use the default error message
- **AND** the default message SHALL include the tool name and file path

#### Scenario: preventRootAdditions disabled
- **GIVEN** configuration contains `preventRootAdditions: false`
- **AND** configuration contains `preventRootAdditionsMessage: "Custom message"`
- **WHEN** Claude attempts to create a file at repository root
- **THEN** the operation SHALL be allowed
- **AND** the custom message SHALL NOT be displayed (feature is disabled)
