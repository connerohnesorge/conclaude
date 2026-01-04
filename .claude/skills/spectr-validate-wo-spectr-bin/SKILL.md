---
name: spectr-validate-wo-spectr-bin
description: |
  Validate Spectr specifications and change proposals without requiring the spectr binary
  USE WHEN you're in a sandboxed or restricted execution context and spectr is not available in your path.
  DO NOT USE WHEN you need a lightweight alternative for task acceptance, but have the spectr binary available.
  DO NOT USE when you have the spectr binary available.
compatibility:
  requirements:
    - bash 4.0+
    - grep (GNU or BSD compatible)
    - sed (GNU or BSD compatible)
    - find (standard Unix utility)
  optional:
    - jq (JSON processor for --json output)
  platforms:
    - Linux
    - macOS
    - Unix-like systems with bash
---

# Spectr Validate (Without Binary)

This skill provides the ability to validate Spectr specifications and change proposals without requiring the `spectr` binary. This is particularly useful in sandboxed environments, CI pipelines, or fresh repository checkouts where the spectr binary may not be available.

## Usage

The skill provides a `scripts/validate.sh` script that implements the core validation rules matching the behavior of `spectr validate`.

### Basic Usage

```bash
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh [--spec <id> | --change <id> | --all] [--json]
```

### Validation Modes

#### Validate a Single Specification

```bash
# Validate the "validation" spec
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --spec validation
```

This validates `spectr/specs/validation/spec.md` for:
- Presence of `## Requirements` section
- Requirements contain SHALL or MUST keywords
- Requirements have at least one `#### Scenario:` block
- Proper scenario formatting (correct header levels, no bullets/bold)

#### Validate a Single Change

```bash
# Validate the "add-new-feature" change
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --change add-new-feature
```

This validates `spectr/changes/add-new-feature/`:
- All delta spec files (`specs/*/spec.md`)
- Presence of at least one delta section (ADDED, MODIFIED, REMOVED, RENAMED)
- Delta sections are not empty
- ADDED/MODIFIED requirements have scenarios and SHALL/MUST
- REMOVED requirements use proper format
- Tasks file (`tasks.md`) contains valid task items (if present)

#### Validate All Specifications and Changes

```bash
# Validate entire repository
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --all
```

This discovers and validates:
- All specifications in `spectr/specs/`
- All changes in `spectr/changes/` (excluding archive)

### Output Formats

#### Human-Readable Output (Default)

```bash
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --spec validation
```

Example output:
```
changes/add-new-feature/specs/validation/spec.md
  [ERROR] line 42: Requirement missing scenario block
  [ERROR] line 58: Malformed scenario header (3 hashtags)

Summary: 0 passed, 1 failed (2 errors), 1 total
```

Colors are automatically enabled when output is to a TTY (terminal) and disabled when piped or redirected.

#### JSON Output

```bash
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --all --json
```

Example output:
```json
{
  "version": 1,
  "items": [
    {
      "name": "validation",
      "type": "spec",
      "valid": false,
      "issues": [
        {
          "level": "ERROR",
          "path": "specs/validation/spec.md",
          "line": 42,
          "message": "Requirement missing scenario block"
        }
      ]
    }
  ],
  "summary": {
    "total": 1,
    "passed": 0,
    "failed": 1,
    "errors": 1,
    "warnings": 0
  }
}
```

**Note**: JSON output requires `jq` to be installed. If `jq` is not available, the script will warn and fall back to human-readable output.

### Additional Options

#### Help

```bash
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --help
# or
bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh -h
```

Displays usage information, flags, and examples.

#### Custom Spectr Directory

```bash
# Validate specs in a different location
SPECTR_DIR=custom/path/to/spectr bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --all
```

The `SPECTR_DIR` environment variable allows you to specify an alternative location for the spectr directory (default: `spectr`).

## Validation Rules

The script implements the following validation rules, matching `spectr validate` behavior:

### Specification File Validation

1. **Requirements Section**
   - **ERROR**: Spec file missing `## Requirements` section
   - Every specification must have exactly one Requirements section

2. **Requirement Headers**
   - Requirements must use `### Requirement: <name>` format
   - Requirements must contain SHALL or MUST keywords (strict mode)

3. **Scenario Blocks**
   - **ERROR**: Requirement missing `#### Scenario:` block (strict mode)
   - Each requirement must have at least one scenario

4. **Malformed Scenarios**
   - **ERROR**: Scenario with wrong header level (3, 5, or 6 hashtags instead of 4)
   - **ERROR**: Scenario using bold (`**Scenario:**`) instead of header
   - **ERROR**: Scenario using bullet point (`- **Scenario:**`)

### Change Delta Validation

1. **Delta Sections**
   - **ERROR**: No delta sections found (ADDED, MODIFIED, REMOVED, RENAMED)
   - **ERROR**: Delta section exists but is empty (no requirements)

2. **ADDED Requirements**
   - **ERROR**: Missing scenario block
   - **ERROR**: Missing SHALL or MUST keywords

3. **MODIFIED Requirements**
   - **ERROR**: Missing scenario block
   - **ERROR**: Missing SHALL or MUST keywords

4. **REMOVED Requirements**
   - Format validation only (no scenario/SHALL validation)

5. **RENAMED Requirements**
   - Different format, minimal validation

### Tasks File Validation

1. **Task Items**
   - **ERROR**: `tasks.md` exists but contains zero task items
   - Task items must use format: `- [ ] Task description` or `- [x] Task description`
   - Task IDs are optional: `- [ ] 1.1 Task description`
   - Uppercase X is supported: `- [X] Task description`

## Exit Codes

The script uses standard exit codes for CI integration:

- **0**: All validations passed (no errors)
- **1**: One or more validations failed (errors detected)
- **2**: Usage error (invalid arguments or flags)

### Example CI Usage

```bash
# Run validation in CI pipeline
if bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --all; then
  echo "Validation passed!"
else
  echo "Validation failed!"
  exit 1
fi
```

## Requirements

### Required Dependencies

- **bash 4.0+**: The script uses associative arrays and other bash 4+ features
- **grep**: For pattern matching (GNU or BSD compatible)
- **sed**: For text processing (GNU or BSD compatible)
- **find**: For discovering specs and changes

You can verify bash version:
```bash
bash --version
```

### Optional Dependencies

- **jq**: Required for `--json` output format

You can verify jq is available:
```bash
which jq
```

If jq is not available and `--json` is requested, the script will emit a warning and fall back to human-readable output.

## Limitations

This skill provides validation capabilities equivalent to `spectr validate` but has some limitations:

### Sequential Processing

- The script processes files sequentially, not in parallel
- Performance is acceptable for typical usage (30-50 spec files)
- For very large repositories, the full `spectr` binary may be faster

### No Pre-Merge Validation

- The script validates the current state on disk
- It does not validate pre-merge scenarios (staged changes, specific commits)
- Use `spectr validate --pre-merge` for advanced pre-commit validation

### No Cross-Capability Duplicate Detection

- The script does not check for duplicate requirement IDs across capabilities
- This is a limitation of the bash implementation
- Use the full `spectr` binary if you need this validation

### Regex Pattern Synchronization

- Validation patterns must stay synchronized with `internal/markdown/` matchers
- Behavior differences may exist if patterns diverge
- Report any discrepancies between script and binary behavior

## When to Use

Use this skill when:
- The spectr binary is not available in your environment
- You're in a sandboxed or restricted execution context (Claude Code, Cursor)
- You need a lightweight alternative for validation
- You're running in CI/CD pipelines without spectr installed
- You need validation during AI-assisted specification authoring

For production workflows with advanced requirements (pre-merge validation, parallel processing, cross-capability checks), consider installing the full spectr binary.

## Troubleshooting

### "bash: version 4.0+ required"

The script uses associative arrays which require bash 4.0+. Update your bash version or use the full spectr binary.

### "jq not found, falling back to human output"

This warning appears when `--json` is used but `jq` is not installed. Install jq:

```bash
# macOS
brew install jq

# Ubuntu/Debian
apt-get install jq

# Alpine
apk add jq
```

### "No specs found in spectr/specs/"

Ensure you're running the script from the repository root or set `SPECTR_DIR`:

```bash
SPECTR_DIR=/path/to/spectr bash .claude/skills/spectr-validate-wo-spectr-bin/scripts/validate.sh --all
```

### "Spec directory not found: <id>"

The specified spec ID does not exist under `spectr/specs/`. Check available specs:

```bash
ls spectr/specs/
```

### "Change directory not found: <id>"

The specified change ID does not exist under `spectr/changes/`. Check available changes:

```bash
ls spectr/changes/
```

### Validation Results Differ from Binary

If you notice differences between script validation and `spectr validate` output:

1. Check regex pattern synchronization
2. Verify bash version (4.0+)
3. Report discrepancies as issues

The script aims to match binary behavior exactly, but edge cases may exist.

## Performance Notes

### Expected Performance

- **Small repos** (< 10 specs): < 1 second
- **Medium repos** (10-30 specs): 1-3 seconds
- **Large repos** (30-50 specs): 3-5 seconds

Performance is primarily determined by:
- Number of spec files
- Size of spec files (line count)
- Disk I/O speed

### Performance vs Binary

The Go binary (`spectr validate`) uses parallel workers and is significantly faster for large repositories. Use the binary when:
- Validating very large repositories (100+ specs)
- Running validation frequently in tight loops
- Performance is critical

Use the skill when:
- Binary installation is not possible
- Portability is more important than speed
- Repository size is small to medium
