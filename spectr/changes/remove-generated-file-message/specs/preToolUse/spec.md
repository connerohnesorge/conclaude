## ADDED Requirements

### Requirement: Reject Removed generatedFileMessage Field

The system SHALL reject configuration containing the deprecated `generatedFileMessage` field, which has been removed from the preToolUse configuration schema.

#### Scenario: Field rejected in configuration
- **WHEN** a user provides `preToolUse.generatedFileMessage` in their configuration
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate that `generatedFileMessage` is no longer a valid field

#### Scenario: Default message used for generation marker blocks
- **WHEN** `preventGeneratedFileEdits: true` is configured
- **AND** Claude attempts to edit a file containing a generation marker
- **THEN** the system SHALL block the operation with the default message format
- **AND** the message SHALL include the file path and detected marker
- **AND** no customization of this message SHALL be available
