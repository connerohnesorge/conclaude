## ADDED Requirements

### Requirement: No Automatic Content-Based Protection

The system SHALL NOT automatically define settings that target specific text patterns within files for determining file editability. Users MAY configure their own explicit file protection rules via `uneditableFiles` glob patterns.

#### Scenario: No built-in text markers

- WHEN conclaude evaluates whether a file can be edited
- THEN the system SHALL NOT scan file contents for markers like "DO NOT EDIT", "@generated", or "AUTO-GENERATED"
- AND file editability SHALL be determined solely by user-configured rules

#### Scenario: User configures explicit protection

- WHEN a user wants to protect generated files
- THEN the user SHALL configure protection via `uneditableFiles` with explicit glob patterns (e.g., `["*.generated.ts", "generated/**"]`)
- AND the system SHALL NOT augment these rules with automatic content detection
