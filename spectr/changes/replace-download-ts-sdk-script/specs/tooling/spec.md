## ADDED Requirements

### Requirement: TypeScript SDK Download Binary
The system SHALL provide a Rust binary (`download-ts-sdk`) that downloads and extracts the Anthropic TypeScript SDK (`@anthropic-ai/claude-agent-sdk`) from the npm registry to `.claude/contexts/claude-agent-sdk-ts` without requiring npm, node, or any JavaScript runtime to be installed.

#### Scenario: Successful SDK download on fresh system
- **WHEN** the `download-ts-sdk` binary is executed
- **AND** the target directory does not exist
- **THEN** it fetches package metadata from `https://registry.npmjs.org/@anthropic-ai/claude-agent-sdk`
- **AND** downloads the tarball for the latest version
- **AND** creates the target directory `.claude/contexts/claude-agent-sdk-ts`
- **AND** extracts the tarball contents (stripping the `package/` prefix)
- **AND** displays success messages including the installed version

#### Scenario: Existing installation replacement
- **WHEN** the target directory `.claude/contexts/claude-agent-sdk-ts` already contains a previous SDK installation (has `package.json`)
- **THEN** the binary prints `[INFO] Removing old installation...`
- **AND** removes the existing target directory
- **AND** creates a fresh target directory
- **AND** extracts the new version

#### Scenario: npm not installed on system
- **WHEN** the `download-ts-sdk` binary is executed on a system without npm or node installed
- **THEN** the download completes successfully using only the npm registry HTTP API
- **AND** no npm or node commands are invoked

### Requirement: Colored Terminal Output
The download binary SHALL display colored output using ANSI escape codes matching the original shell script format.

#### Scenario: Informational messages
- **WHEN** the binary displays informational messages
- **THEN** they are prefixed with `[INFO]` in green (`\x1b[0;32m`)
- **AND** followed by the reset code (`\x1b[0m`)

#### Scenario: Warning messages
- **WHEN** the binary displays warning messages
- **THEN** they are prefixed with `[WARN]` in yellow (`\x1b[1;33m`)
- **AND** followed by the reset code (`\x1b[0m`)

#### Scenario: Error messages
- **WHEN** the binary displays error messages
- **THEN** they are prefixed with `[ERROR]` in red (`\x1b[0;31m`)
- **AND** written to stderr
- **AND** followed by the reset code (`\x1b[0m`)

### Requirement: Error Handling
The download binary SHALL handle errors gracefully and provide descriptive error messages.

#### Scenario: Network failure during metadata fetch
- **WHEN** the npm registry is unreachable during metadata fetch
- **THEN** the binary displays `[ERROR] Failed to fetch package metadata`
- **AND** provides guidance about network connectivity
- **AND** exits with non-zero status

#### Scenario: Network failure during tarball download
- **WHEN** the tarball download fails
- **THEN** the binary displays `[ERROR] Failed to download package tarball`
- **AND** provides guidance about network connectivity
- **AND** exits with non-zero status

#### Scenario: Extraction failure
- **WHEN** the tarball extraction fails
- **THEN** the binary displays `[ERROR] Failed to extract package tarball`
- **AND** exits with non-zero status

#### Scenario: Directory creation failure
- **WHEN** the target directory cannot be created (permission denied)
- **THEN** the binary displays `[ERROR] Failed to create directory`
- **AND** advises checking write permissions
- **AND** exits with non-zero status

### Requirement: Success Output Format
The download binary SHALL display a specific sequence of messages on successful completion matching the original shell script.

#### Scenario: Complete success output
- **WHEN** the download and extraction complete successfully
- **THEN** the binary outputs the following sequence:
  1. Blank line
  2. `[INFO] TypeScript SDK Download Script`
  3. Blank line
  4. `[INFO] Downloading @anthropic-ai/claude-agent-sdk...`
  5. `[INFO] Extracting TypeScript SDK source files...`
  6. `[INFO] Removing old installation...` (only if previous installation exists)
  7. `[INFO] Installing to .claude/contexts/claude-agent-sdk-ts...`
  8. `[INFO] TypeScript SDK successfully installed!`
  9. Blank line
  10. `[INFO] ✓ TypeScript SDK source files are available at: .claude/contexts/claude-agent-sdk-ts`
  11. `[INFO] ✓ Source code location: .claude/contexts/claude-agent-sdk-ts/src` (only if src/ directory exists)
  12. `[INFO] ✓ Version: {extracted_version}`
  13. Blank line
