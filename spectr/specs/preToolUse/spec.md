# PreToolUse Specification

## Purpose

Define the PreToolUse hook validation rules that control file protection policies, including root-level file addition prevention, glob pattern-based file protection via uneditableFiles, tool usage validation rules, and preventAdditions enforcement.

## Requirements

### Requirement: Root-Level File Addition Prevention

The system SHALL prevent Claude from creating **new** files at the repository root when `preventRootAdditions` is enabled. However, the system SHALL allow modifications to existing root-level files.

**Previous behavior:** Blocked all file creation and modification at root level (overly restrictive).

**New behavior:** Only blocks creation of new files at root; allows editing and overwriting existing root files (balanced protection).

#### Scenario: Prevent root additions enabled
- **WHEN** `preToolUse.preventRootAdditions` is set to `true`
- **AND** the target file does NOT exist at repository root
- **THEN** Claude SHALL NOT be allowed to create the new file
- **AND** any attempt to create such files SHALL result in an error message explaining the restriction

#### Scenario: Allow modification of existing root files
- **WHEN** `preToolUse.preventRootAdditions` is set to `true`
- **AND** the target file already exists at repository root
- **THEN** Claude SHALL be allowed to modify/overwrite the existing file
- **AND** no preventRootAdditions error SHALL be generated

#### Scenario: Prevent root additions disabled
- **WHEN** `preToolUse.preventRootAdditions` is set to `false`
- **THEN** Claude SHALL be allowed to create or modify files at the repository root
- **AND** all file operations in subdirectories remain subject to other restrictions

#### Scenario: Default behavior
- **WHEN** `preToolUse.preventRootAdditions` is not specified in configuration
- **THEN** the system SHALL default to `preventRootAdditions: true`
- **AND** root-level file creation SHALL be prevented by default
- **AND** existing root files may still be modified

### Requirement: File Protection via Glob Patterns

The system SHALL prevent Claude from editing specified files using glob patterns in the `uneditableFiles` configuration, **with optional agent scoping**.

#### Scenario: Exact file match (updated)

- **WHEN** `preToolUse.uneditableFiles` contains `"package.json"` (simple format)
- **THEN** ALL agents SHALL NOT be allowed to edit `package.json` at any directory level
- **AND** any attempt to modify this file SHALL result in an error message

#### Scenario: Detailed format with agent scoping

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "package.json", message: "...", agent: "coder" }`
- **THEN** only the "coder" agent SHALL be blocked from editing `package.json`
- **AND** other agents (main, tester) SHALL NOT be blocked by this rule

#### Scenario: Configuration validation with agent field

- **WHEN** `preToolUse.uneditableFiles` contains a detailed entry with `agent` field
- **AND** `agent` is a valid string (including glob patterns)
- **THEN** the configuration SHALL be accepted
- **AND** the agent pattern SHALL be validated against glob syntax

### Requirement: Configuration Validation
The system SHALL validate `preventRootAdditions` and `uneditableFiles` configuration values.

#### Scenario: Valid preventRootAdditions value
- **WHEN** `preToolUse.preventRootAdditions` contains a boolean value (`true` or `false`)
- **THEN** the configuration SHALL be accepted
- **AND** the setting SHALL be applied during pre-tool-use validation

#### Scenario: Invalid preventRootAdditions value
- **WHEN** `preToolUse.preventRootAdditions` contains a non-boolean value
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL clearly indicate the type mismatch

#### Scenario: Valid uneditableFiles array
- **WHEN** `preToolUse.uneditableFiles` contains an array of string glob patterns
- **THEN** the configuration SHALL be accepted
- **AND** each pattern SHALL be evaluated against file paths

#### Scenario: Invalid uneditableFiles value
- **WHEN** `preToolUse.uneditableFiles` is not an array (e.g., a string or object)
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate that an array is expected

### Requirement: Combined File Protection Policies
The system SHALL enforce both `preventRootAdditions` and `uneditableFiles` restrictions together as a unified file protection policy.

#### Scenario: Root addition prevention with glob patterns
- **WHEN** both `preventRootAdditions: true` and `uneditableFiles: ["Cargo.toml"]` are configured
- **THEN** Claude SHALL NOT create files in the root directory
- **AND** Claude SHALL NOT edit `Cargo.toml` regardless of directory level
- **AND** both restrictions are enforced independently

#### Scenario: Overlapping protections
- **WHEN** `preventRootAdditions: true` and `uneditableFiles: ["*"]` are configured
- **THEN** all files are protected from modification
- **AND** the system evaluates both rules and applies the most restrictive result

#### Scenario: Nested files with root prevention
- **WHEN** `preventRootAdditions: true` is set
- **THEN** files in subdirectories like `src/app.ts` remain editable (unless blocked by other rules)
- **AND** only root-level files are blocked by this specific restriction

### Requirement: Configuration Default Values and Backward Compatibility
The system SHALL provide appropriate defaults and signal deprecation for the removed `rules` section.

#### Scenario: New configuration format
- **WHEN** a user provides a configuration with `preToolUse` containing `preventRootAdditions` and `uneditableFiles`
- **THEN** the configuration SHALL be accepted as valid
- **AND** the values SHALL be used as specified

#### Scenario: Detection of old configuration format
- **WHEN** configuration contains the old `rules` section
- **THEN** the system SHALL fail configuration loading with an error
- **AND** the error message SHALL clearly indicate that the `rules` section is no longer supported
- **AND** the error message SHALL provide specific migration instructions for moving fields to `preToolUse`

#### Scenario: Migration example in documentation
- **WHEN** user documentation is generated
- **THEN** migration examples SHALL clearly show before/after configurations
- **AND** the rationale for consolidation SHALL be documented

### Requirement: Tool Usage Validation Rules

The system SHALL enforce per-tool restrictions defined in `toolUsageValidation` to control which tools can operate on which files, optionally scoped to specific agents.

#### Scenario: Block tool on file pattern

- **WHEN** `preToolUse.toolUsageValidation` contains a rule blocking "bash" on "*.md" files
- **THEN** Claude SHALL NOT be allowed to execute bash commands on markdown files
- **AND** any attempt SHALL result in an error message referencing the tool usage rule

#### Scenario: Allow tool on specific pattern

- **WHEN** `preToolUse.toolUsageValidation` contains a rule allowing "Write" only on "src/**/*.ts"
- **THEN** Claude SHALL NOT be allowed to use Write tool on files outside the `src/` TypeScript directory
- **AND** the permission boundary SHALL be enforced

#### Scenario: Command pattern matching

- **WHEN** a tool usage rule includes a `commandPattern` (e.g., regex)
- **THEN** the rule SHALL match against the specific command/parameters passed to the tool
- **AND** match mode (exact, regex, glob) SHALL determine matching behavior

#### Scenario: Multiple validation rules

- **WHEN** multiple `toolUsageValidation` rules are configured
- **THEN** all applicable rules SHALL be evaluated
- **AND** the first matched rule SHALL determine the action (block/allow)

#### Scenario: Rule precedence with match modes

- **WHEN** multiple rules could apply to the same tool and file pattern
- **THEN** rules SHALL be evaluated in order
- **AND** the first matching rule SHALL take precedence

#### Scenario: Validation error messages

- **WHEN** a tool usage rule blocks an operation
- **THEN** the error message SHALL include the tool name, file pattern, and custom message if provided
- **AND** the user SHALL understand why the operation was blocked

#### Scenario: Agent-scoped tool usage rule

- **GIVEN** configuration contains:
  ```yaml
  toolUsageValidation:
    - tool: "Bash"
      pattern: "*"
      action: "block"
      commandPattern: "git push*"
      agent: "coder"
      message: "Coder agent cannot push to git"
  ```
- **WHEN** the "coder" subagent attempts to run `git push origin main`
- **THEN** the operation SHALL be blocked with the custom message
- **AND** the orchestrator ("main") SHALL still be allowed to run git push commands

#### Scenario: Agent pattern uses glob matching

- **GIVEN** configuration contains a rule with `agent: "test*"`
- **WHEN** a subagent named "tester" or "test-runner" attempts an operation matching the rule
- **THEN** the rule SHALL apply to both agents
- **AND** agents not matching the pattern (e.g., "coder") SHALL not be affected

#### Scenario: Missing agent field defaults to all agents

- **GIVEN** configuration contains a rule without an `agent` field
- **WHEN** any agent (orchestrator or subagent) attempts an operation matching the rule
- **THEN** the rule SHALL apply to all agents
- **AND** behavior SHALL be equivalent to specifying `agent: "*"`

#### Scenario: Agent field with wildcard matches all

- **GIVEN** configuration contains a rule with `agent: "*"`
- **WHEN** any agent attempts an operation matching the rule
- **THEN** the rule SHALL apply regardless of agent type

#### Scenario: Rule skipped for non-matching agent

- **GIVEN** configuration contains a rule with `agent: "coder"`
- **WHEN** the "tester" subagent attempts an operation that would otherwise match the rule
- **THEN** the rule SHALL be skipped (not applied)
- **AND** subsequent rules SHALL continue to be evaluated

#### Scenario: Agent context detection from session

- **GIVEN** a subagent session is active with `subagent_type: "coder"`
- **WHEN** evaluating tool usage rules with `agent` fields
- **THEN** the system SHALL read the current agent from the session file
- **AND** use "main" if no session file exists (orchestrator context)

#### Scenario: Error message includes agent context

- **WHEN** an agent-scoped tool usage rule blocks an operation
- **THEN** the error message SHALL include the current agent name
- **AND** the error SHALL indicate which agent pattern triggered the block

### Requirement: File Addition Prevention via Glob Patterns
The system SHALL enforce the `preventAdditions` configuration by blocking `Write` tool operations that create NEW files at paths matching configured glob patterns. Existing files can be overwritten.

**Previous behavior (BROKEN):** `preventAdditions` field existed in schema but was never checked by the hook, causing silent failure.

**New behavior (FIXED):** `preventAdditions` patterns are enforced during PreToolUse hook execution for Write tool operations creating new files.

#### Scenario: Exact directory pattern blocks file creation
- **GIVEN** configuration contains `preventAdditions: ["dist"]`
- **WHEN** Claude attempts to use Write tool to create file `dist/output.js`
- **THEN** the operation SHALL be blocked before execution
- **AND** error message SHALL indicate the file matches pattern `"dist"` and show the attempted path

#### Scenario: Recursive directory pattern blocks nested files
- **GIVEN** configuration contains `preventAdditions: ["build/**"]`
- **WHEN** Claude attempts to use Write tool to create file `build/nested/deep/file.js`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL indicate the file matches pattern `"build/**"`

#### Scenario: File extension pattern blocks files
- **GIVEN** configuration contains `preventAdditions: ["*.log"]`
- **WHEN** Claude attempts to use Write tool to create file `debug.log`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL indicate the file matches pattern `"*.log"`

#### Scenario: Multiple patterns all enforced
- **GIVEN** configuration contains `preventAdditions: ["dist/**", "build/**", "*.log"]`
- **WHEN** Claude attempts to create any file matching any of the patterns
- **THEN** the operation SHALL be blocked with appropriate pattern indicated
- **AND** Claude attempts to create file not matching any pattern (e.g., `src/main.rs`)
- **THEN** the operation SHALL be allowed to proceed

#### Scenario: Non-matching paths are allowed
- **GIVEN** configuration contains `preventAdditions: ["dist/**"]`
- **WHEN** Claude attempts to use Write tool to create file `src/components/Button.tsx`
- **THEN** the operation SHALL be allowed (no pattern match)
- **AND** no error message SHALL be generated

#### Scenario: Empty preventAdditions array allows all operations
- **GIVEN** configuration contains `preventAdditions: []`
- **WHEN** Claude attempts to use Write tool to create any file
- **THEN** the operation SHALL be allowed (no patterns to check)
- **AND** no preventAdditions validation SHALL occur

#### Scenario: Existing files can be overwritten
- **GIVEN** configuration contains `preventAdditions: ["docs/**"]`
- **AND** file `docs/README.md` already exists
- **WHEN** Claude attempts to use Write tool to overwrite `docs/README.md`
- **THEN** the operation SHALL be allowed (file already exists)
- **AND** preventAdditions only blocks creation of NEW files, not overwrites

### Requirement: Write Tool Exclusivity for preventAdditions
The system SHALL only apply `preventAdditions` checks to the `Write` tool, not to `Edit` or `NotebookEdit` tools.

#### Scenario: Edit tool bypasses preventAdditions
- **GIVEN** configuration contains `preventAdditions: ["dist/**"]`
- **AND** file `dist/existing.js` already exists
- **WHEN** Claude attempts to use Edit tool to modify `dist/existing.js`
- **THEN** the operation SHALL NOT be blocked by preventAdditions
- **AND** the operation may be subject to `uneditableFiles` checks but not preventAdditions

#### Scenario: NotebookEdit tool bypasses preventAdditions
- **GIVEN** configuration contains `preventAdditions: ["notebooks/**"]`
- **AND** file `notebooks/analysis.ipynb` exists
- **WHEN** Claude attempts to use NotebookEdit tool to modify the notebook
- **THEN** the operation SHALL NOT be blocked by preventAdditions
- **AND** preventAdditions validation SHALL not run for this tool

### Requirement: preventAdditions Error Reporting
The system SHALL provide clear, actionable error messages when preventAdditions blocks a file creation operation.

#### Scenario: Error message includes all context
- **WHEN** preventAdditions blocks a Write operation
- **THEN** error message SHALL include:
  - The tool name (`Write`)
  - The matching glob pattern (e.g., `"dist/**"`)
  - The attempted file path (e.g., `dist/output.js`)
- **AND** error format SHALL be: `"Blocked {tool} operation: file matches preToolUse.preventAdditions pattern '{pattern}'. File: {path}"`

#### Scenario: Diagnostic logging for debugging
- **WHEN** preventAdditions blocks an operation
- **THEN** a diagnostic message SHALL be logged to stderr
- **AND** log message SHALL include: tool_name, file_path, and matching pattern

### Requirement: File Existence Check for Root Additions

The system SHALL check if a target file exists at the resolved path before determining whether to block a Write operation under preventRootAdditions.

#### Scenario: Existence check prevents false positives
- **GIVEN** configuration contains `preToolUse.preventRootAdditions: true`
- **WHEN** determining whether to block a Write operation
- **THEN** the system SHALL check if the file exists at the resolved path
- **AND** only block if file does NOT exist at root

#### Scenario: File existence allows write
- **GIVEN** configuration contains `preToolUse.preventRootAdditions: true`
- **AND** file `package.json` exists at root
- **WHEN** Claude attempts to use Write tool to overwrite/modify `package.json`
- **THEN** the operation SHALL be allowed
- **AND** no error message SHALL be generated for preventRootAdditions

#### Scenario: Non-existent file is blocked
- **GIVEN** configuration contains `preToolUse.preventRootAdditions: true`
- **AND** file `docker-compose.yml` does NOT exist at root
- **WHEN** Claude attempts Write to `docker-compose.yml`
- **THEN** the system SHALL detect file does not exist
- **AND** the operation SHALL be blocked (new file at root)

---

**Summary:** preventRootAdditions now correctly allows modifications to existing root-level files while maintaining protection against creating new files at the root. This preserves the semantic meaning of "preventRootAdditions" (prevent adding/creating files at root) while enabling practical workflows that require updating configuration files.

### Requirement: Git-Ignored File Protection Configuration
The system SHALL provide an optional `preventUpdateGitIgnored` boolean field in the `preToolUse` configuration to block Claude from modifying or creating files that match entries in `.gitignore`.

#### Scenario: preventUpdateGitIgnored enabled
- **WHEN** `preToolUse.preventUpdateGitIgnored` is set to `true`
- **THEN** the system SHALL check if any requested file operation targets a path that matches an entry in `.gitignore`
- **AND** if matched, the operation SHALL be blocked with a clear error message
- **AND** if not matched, the operation SHALL proceed normally

#### Scenario: preventUpdateGitIgnored disabled
- **WHEN** `preToolUse.preventUpdateGitIgnored` is set to `false`
- **THEN** git-ignore rules SHALL NOT be evaluated
- **AND** Claude SHALL be allowed to create or modify files freely (subject to other restrictions)
- **AND** existing behavior is preserved

#### Scenario: Default behavior
- **WHEN** `preToolUse.preventUpdateGitIgnored` is not specified in configuration
- **THEN** the system SHALL default to `preventUpdateGitIgnored: false`
- **AND** git-ignored files are not automatically protected

### Requirement: Git-Ignore Pattern Matching
The system SHALL correctly evaluate files against `.gitignore` patterns using git-standard semantics.

#### Scenario: Simple pattern match
- **WHEN** `.gitignore` contains `node_modules`
- **AND** Claude attempts to modify `node_modules/package.json`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked
- **AND** an error message SHALL indicate the file is git-ignored

#### Scenario: Glob pattern match
- **WHEN** `.gitignore` contains `*.log`
- **AND** Claude attempts to create `debug.log`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked

#### Scenario: Directory pattern match
- **WHEN** `.gitignore` contains `.env` (exact filename)
- **AND** Claude attempts to modify `.env` in the repository root
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked

#### Scenario: Nested .gitignore files
- **WHEN** repository contains `.gitignore` at root and `src/.gitignore` in a subdirectory
- **AND** `src/.gitignore` contains `local-config.json`
- **AND** Claude attempts to modify `src/local-config.json`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked

#### Scenario: Negation patterns
- **WHEN** `.gitignore` contains `*.log` followed by `!important.log`
- **AND** Claude attempts to modify `important.log`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL NOT be blocked (negation pattern allows it)

#### Scenario: Comments in .gitignore
- **WHEN** `.gitignore` contains `# Comment` on a line
- **AND** Claude attempts to create a file named `# Comment`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL proceed (comment lines are not patterns)

### Requirement: File Operation Blocking Scope
The system SHALL block Read, Write, and Edit operations that target git-ignored paths. Glob operations are NOT blocked.

#### Scenario: Block Read operation
- **WHEN** `.gitignore` contains `.env`
- **AND** Claude uses `Read` tool to read `.env`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked before execution

#### Scenario: Block Write operation (file creation)
- **WHEN** `.gitignore` contains `*.tmp`
- **AND** Claude uses `Write` tool to create `session.tmp`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked before execution

#### Scenario: Block Edit operation (file modification)
- **WHEN** `.gitignore` contains `config.local`
- **AND** Claude uses `Edit` tool to modify existing `config.local` file
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked before execution

#### Scenario: Allow Glob operations
- **WHEN** `.gitignore` contains `node_modules/`
- **AND** Claude uses `Glob` tool with pattern `**/*.js`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be allowed (Glob is not blocked)
- **AND** Glob results may include git-ignored files

#### Scenario: Allow operations on non-ignored files
- **WHEN** `.gitignore` contains `*.log`
- **AND** Claude attempts to Read or Write `src/main.ts`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be allowed (file is not ignored)

### Requirement: Error Reporting for Blocked Operations
The system SHALL provide clear, actionable error messages when blocking git-ignored file operations.

#### Scenario: Blocked operation error message
- **WHEN** Claude attempts to modify a git-ignored file with `preventUpdateGitIgnored: true`
- **THEN** an error message SHALL be returned indicating:
  - The file path that was blocked
  - The reason (git-ignored status)
  - The `.gitignore` pattern(s) that matched
  - A suggestion to update `.gitignore` or disable the setting if needed

#### Scenario: Error includes matching pattern
- **WHEN** `.gitignore` contains `dist/` and Claude tries to write `dist/app.js`
- **THEN** the error message SHALL include the matching pattern `dist/`

#### Scenario: Error indicates setting responsible
- **WHEN** a file operation is blocked due to git-ignore
- **THEN** the error message SHALL clearly state that `preventUpdateGitIgnored` setting is enforcing this restriction

### Requirement: Git-Ignored File Configuration Validation
The system SHALL validate the `preventUpdateGitIgnore` boolean field.

#### Scenario: Valid boolean value
- **WHEN** `preToolUse.preventUpdateGitIgnored` is set to `true` or `false`
- **THEN** the configuration SHALL be accepted
- **AND** the setting SHALL be applied during pre-tool-use validation

#### Scenario: Invalid non-boolean value
- **WHEN** `preToolUse.preventUpdateGitIgnored` contains a non-boolean value (e.g., `"yes"`, `1`, `null`)
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the type mismatch and expected boolean value

#### Scenario: Missing field defaults to false
- **WHEN** `preToolUse.preventUpdateGitIgnored` is not specified
- **THEN** the system SHALL default to `false`
- **AND** no validation error SHALL occur

### Requirement: Git-Ignored Combined Protection Policies
The system SHALL enforce `preventUpdateGitIgnored` alongside existing file protection mechanisms.

#### Scenario: preventUpdateGitIgnored with preventRootAdditions
- **WHEN** both `preventRootAdditions: true` and `preventUpdateGitIgnored: true` are configured
- **AND** `.gitignore` contains `.env`
- **AND** Claude attempts to create `.env` (root-level git-ignored file)
- **THEN** the operation SHALL be blocked
- **AND** the error message SHALL indicate which restriction applied (or both)

#### Scenario: preventUpdateGitIgnored with uneditableFiles
- **WHEN** both `preventUpdateGitIgnored: true` and `uneditableFiles: ["*.lock"]` are configured
- **AND** `.gitignore` contains `node_modules/`
- **AND** Claude attempts to modify `node_modules/file.js`
- **THEN** the operation SHALL be blocked by git-ignore check

#### Scenario: preventUpdateGitIgnored with uneditableFiles overlap
- **WHEN** both `preventUpdateGitIgnored: true` and `uneditableFiles: [".env"]` are configured
- **AND** `.gitignore` also contains `.env`
- **AND** Claude attempts to modify `.env`
- **THEN** the operation SHALL be blocked
- **AND** the system evaluates both rules and applies the most restrictive result

#### Scenario: Multiple protection rules enforced
- **WHEN** `preventRootAdditions: true`, `uneditableFiles: ["Cargo.toml"]`, and `preventUpdateGitIgnored: true` are all configured
- **AND** `.gitignore` contains `dist/`
- **THEN** all three rules are evaluated independently for each file operation
- **AND** if any rule blocks the operation, it SHALL be denied

### Requirement: Git-Ignore Semantics Compliance
The system SHALL respect standard git-ignore semantics and behavior.

#### Scenario: Leading slash anchors to root
- **WHEN** `.gitignore` contains `/build` (leading slash)
- **AND** Claude attempts to modify `build/output.js` in the repository root
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked
- **AND** a `build/` directory in subdirectories is not blocked by this rule

#### Scenario: Trailing slash matches directories only
- **WHEN** `.gitignore` contains `dist/` (trailing slash)
- **AND** Claude attempts to create a file named `dist` (as a file, not directory)
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation may proceed (pattern matches directories only)

#### Scenario: Double asterisk matches nested levels
- **WHEN** `.gitignore` contains `src/**/*.test.ts`
- **AND** Claude attempts to modify `src/components/Button.test.ts`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL be blocked

#### Scenario: Exclamation negation overrides
- **WHEN** `.gitignore` contains:
  - `node_modules/`
  - `!node_modules/important-package/`
- **AND** Claude attempts to modify `node_modules/important-package/file.js`
- **AND** `preventUpdateGitIgnored: true`
- **THEN** the operation SHALL NOT be blocked (negation overrides the general rule)

### Requirement: Performance and Caching
The system SHALL cache git-ignore evaluation results to minimize performance impact.

#### Scenario: Git-ignore cache within session
- **WHEN** Claude performs multiple file operations within a session
- **THEN** git-ignore rules SHALL be loaded and parsed once per session (or when `.gitignore` changes)
- **AND** subsequent checks SHALL use cached rules for efficiency
- **AND** cache invalidation SHALL occur when `.gitignore` is modified

#### Scenario: No performance regression
- **WHEN** `preventUpdateGitIgnored` is set to `false`
- **THEN** no git-ignore checking code SHALL execute
- **AND** there SHALL be no performance impact on tool execution

### Requirement: Agent-Aware File Protection

The system SHALL support optional `agent` field in `uneditableFiles` detailed format to restrict file protection rules to specific agents or agent patterns.

#### Scenario: Rule with agent wildcard applies to all agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: ".conclaude.yml", agent: "*" }`
- **AND** any agent (main session, coder, tester, etc.) attempts to edit `.conclaude.yml`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL indicate the file matches uneditableFiles pattern

#### Scenario: Rule with specific agent only blocks that agent

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "tasks.jsonc", agent: "coder" }`
- **AND** the "coder" subagent attempts to edit `tasks.jsonc`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL include agent context (e.g., "agent: coder")

#### Scenario: Rule with specific agent does not block other agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "tasks.jsonc", agent: "coder" }`
- **AND** the main session (orchestrator) attempts to edit `tasks.jsonc`
- **THEN** the operation SHALL NOT be blocked by this rule
- **AND** the file is editable unless blocked by another rule

#### Scenario: Rule with glob pattern matches multiple agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: "src/**/*.ts", agent: "code*" }`
- **AND** the "coder" subagent attempts to edit `src/app.ts`
- **THEN** the operation SHALL be blocked (pattern "code*" matches "coder")
- **AND** if "coder-v2" subagent attempts the same, it SHALL also be blocked

#### Scenario: Main session identified as "main" agent

- **WHEN** preToolUse hook executes in the main Claude session (not a subagent)
- **AND** `preToolUse.uneditableFiles` contains `{ pattern: "config.yml", agent: "main" }`
- **AND** main session attempts to edit `config.yml`
- **THEN** the operation SHALL be blocked
- **AND** error message SHALL reference agent "main"

#### Scenario: Subagent identified from transcript

- **WHEN** preToolUse hook executes within a subagent context
- **THEN** the system SHALL parse the transcript to extract the subagent type from the Task tool invocation
- **AND** the extracted subagent type SHALL be used for agent pattern matching

### Requirement: Agent Field Default Behavior

The system SHALL default `agent` to `"*"` (all agents) when the field is omitted, ensuring backward compatibility with existing configurations.

#### Scenario: Rule without agent field applies to all agents

- **WHEN** `preToolUse.uneditableFiles` contains `{ pattern: ".env" }` (no agent field)
- **AND** any agent attempts to edit `.env`
- **THEN** the operation SHALL be blocked
- **AND** behavior SHALL be identical to `{ pattern: ".env", agent: "*" }`

#### Scenario: Simple string format applies to all agents

- **WHEN** `preToolUse.uneditableFiles` contains `".env"` (simple string format)
- **AND** any agent attempts to edit `.env`
- **THEN** the operation SHALL be blocked
- **AND** simple format SHALL continue to apply to all agents

### Requirement: Agent Pattern Matching Semantics

The system SHALL use glob-style pattern matching for the `agent` field, consistent with file pattern matching.

#### Scenario: Wildcard matches all agents

- **WHEN** agent pattern is `"*"`
- **THEN** it SHALL match main session, coder, tester, and any other agent type

#### Scenario: Exact match requires full agent name

- **WHEN** agent pattern is `"coder"`
- **THEN** it SHALL match agent type "coder" exactly
- **AND** it SHALL NOT match "coder-v2" or "tester"

#### Scenario: Glob pattern with asterisk suffix

- **WHEN** agent pattern is `"code*"`
- **THEN** it SHALL match "coder", "coder-v2", "codefix", etc.
- **AND** it SHALL NOT match "tester" or "main"

#### Scenario: Case-sensitive matching

- **WHEN** agent pattern is `"Coder"` (capitalized)
- **AND** actual agent type is `"coder"` (lowercase)
- **THEN** the pattern SHALL NOT match (case-sensitive)

### Requirement: Agent Context in Error Messages

The system SHALL include agent context in error messages when an agent-specific uneditableFiles rule blocks an operation.

#### Scenario: Error message includes agent for specific rules

- **WHEN** a uneditableFiles rule with `agent: "coder"` blocks an operation
- **THEN** error message SHALL include the agent context
- **AND** format SHALL be: `"Blocked {tool} operation: file matches preToolUse.uneditableFiles pattern '{pattern}' (agent: {agent}). File: {path}"`

#### Scenario: Error message omits agent for universal rules

- **WHEN** a uneditableFiles rule with `agent: "*"` or no agent field blocks an operation
- **THEN** error message MAY omit agent context
- **AND** format MAY remain: `"Blocked {tool} operation: file matches preToolUse.uneditableFiles pattern '{pattern}'. File: {path}"`

### Requirement: Agent Detection Failure Handling

The system SHALL handle agent detection failures gracefully without blocking legitimate operations.

#### Scenario: Transcript parse failure defaults to main

- **WHEN** the transcript cannot be parsed to extract agent type
- **THEN** the system SHALL log a warning
- **AND** the system SHALL treat the session as "main" agent
- **AND** rules with `agent: "main"` or `agent: "*"` SHALL apply

#### Scenario: Missing transcript defaults to main

- **WHEN** the transcript path is empty or file does not exist
- **THEN** the system SHALL treat the session as "main" agent
- **AND** agent-specific rules SHALL be evaluated against "main"

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

### Requirement: Updated Input with Ask Permission Decision

The system SHALL support returning `updatedInput` alongside an `ask` permission decision in PreToolUse hook responses, enabling hooks to act as middleware that modifies tool inputs while still requesting user consent.

#### Scenario: Ask decision with updated input
- **WHEN** a PreToolUse hook returns `decision: "ask"` with `updated_input` containing modified parameters
- **THEN** the system SHALL serialize both the decision and updated input in the response
- **AND** the user SHALL be prompted to approve the modified operation
- **AND** if approved, the tool SHALL execute with the modified input parameters

#### Scenario: Ask decision with reason and updated input
- **GIVEN** a PreToolUse hook that sanitizes bash commands
- **WHEN** the hook returns:
  ```json
  {
    "decision": "ask",
    "message": "Command modified for safety",
    "updated_input": {
      "command": "sanitized-command"
    }
  }
  ```
- **THEN** the user SHALL see the reason message
- **AND** if the user approves, the sanitized command SHALL be executed
- **AND** if the user denies, the operation SHALL be blocked

#### Scenario: Ask decision without updated input (backward compatible)
- **WHEN** a PreToolUse hook returns `decision: "ask"` without `updated_input`
- **THEN** the system SHALL prompt for user approval with the original tool input
- **AND** behavior SHALL be identical to the legacy permission flow

#### Scenario: Updated input partial replacement
- **GIVEN** a tool input with fields: `{ "command": "rm -rf /", "timeout": 30 }`
- **WHEN** a hook returns `updated_input: { "command": "echo hello" }`
- **THEN** only the `command` field SHALL be replaced
- **AND** the `timeout` field SHALL remain unchanged at 30
- **AND** the final input SHALL be `{ "command": "echo hello", "timeout": 30 }`

### Requirement: Permission Decision Field

The system SHALL support an explicit `decision` field in PreToolUse hook responses with values: "allow", "deny", or "ask".

#### Scenario: Decision field allow
- **WHEN** a PreToolUse hook returns `decision: "allow"`
- **THEN** the tool execution SHALL proceed without user prompt
- **AND** any `updated_input` SHALL be applied to the tool parameters
- **AND** behavior SHALL be equivalent to returning `blocked: false`

#### Scenario: Decision field deny
- **WHEN** a PreToolUse hook returns `decision: "deny"`
- **THEN** the tool execution SHALL be blocked
- **AND** the `message` field SHALL be shown to Claude as the reason
- **AND** behavior SHALL be equivalent to returning `blocked: true`

#### Scenario: Decision field ask
- **WHEN** a PreToolUse hook returns `decision: "ask"`
- **THEN** the user SHALL be prompted to approve or deny the operation
- **AND** the `message` field SHALL be shown to the user (not Claude)
- **AND** any `updated_input` SHALL be applied if the user approves

#### Scenario: Decision field takes precedence over blocked
- **WHEN** a PreToolUse hook returns both `decision: "allow"` and `blocked: true`
- **THEN** the `decision` field SHALL take precedence
- **AND** the tool SHALL be allowed to execute
- **AND** a warning MAY be logged about conflicting fields

#### Scenario: Missing decision field uses blocked field
- **WHEN** a PreToolUse hook returns without a `decision` field
- **THEN** the system SHALL use the `blocked` field to determine behavior
- **AND** `blocked: true` SHALL block the operation
- **AND** `blocked: false` SHALL allow the operation
- **AND** backward compatibility SHALL be maintained

### Requirement: Hook Result Serialization

The system SHALL correctly serialize HookResult with updated_input and decision fields to JSON for Claude Code consumption.

#### Scenario: Serialization includes all fields
- **GIVEN** a HookResult with decision = "ask", message = "Review this", and updated_input = { "command": "safe-cmd" }
- **WHEN** the result is serialized to JSON
- **THEN** the JSON SHALL include:
  ```json
  {
    "decision": "ask",
    "message": "Review this",
    "blocked": false,
    "system_prompt": null,
    "updated_input": {
      "command": "safe-cmd"
    }
  }
  ```

#### Scenario: Null fields omitted or null
- **GIVEN** a HookResult with only `blocked: false` set
- **WHEN** the result is serialized to JSON
- **THEN** optional fields (decision, updated_input) SHALL be null or omitted
- **AND** the JSON SHALL be valid for Claude Code consumption

#### Scenario: Updated input preserves value types
- **GIVEN** updated_input containing various types: string, number, boolean, array, object
- **WHEN** the result is serialized and deserialized
- **THEN** all value types SHALL be preserved correctly
- **AND** no type coercion SHALL occur
