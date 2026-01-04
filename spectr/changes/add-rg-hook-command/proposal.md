# Change: Add ripgrep-based `rg` hook command type

## Why

Currently, enforcing pattern-based file content checks in hooks requires verbose shell commands:
```yaml
- run: "rg 'class=.*(?:bg-|text-)' **/*.html && exit 1 || exit 0"
  message: "Inline CSS found"
```

This is error-prone (inverted exit codes), hard to read, and doesn't integrate with conclaude's output limiting or constraint features. A declarative `rg` command type provides:
- Cleaner, more readable configuration
- Built-in match count constraints (max, min, equal)
- Full ripgrep library integration (no external binary required)
- Comprehensive regex and file traversal options
- Configurable block vs warn behavior

## What Changes

### New `rg` Field Structure

The `rg` field is mutually exclusive with `run`. It contains:

**Required Fields:**
- `pattern` - Regex pattern to search for
- `files` - Glob pattern for file matching (must match at least one file)

**Constraint Fields (mutually exclusive, default: `max: 0`):**
- `max` - Maximum allowed matches
- `min` - Minimum required matches
- `equal` - Exact match count required

**Regex Options (Extended Set):**
- `ignoreCase` - Case insensitive matching (default: false)
- `smartCase` - Auto-detect case sensitivity from pattern (default: false)
- `word` - Word boundary matching like `\b` (default: false)
- `fixedStrings` - Treat pattern as literal, not regex (default: false)
- `multiLine` - `^`/`$` match line boundaries (default: false)
- `wholeLine` - Pattern must match entire line (default: false)
- `dotMatchesNewLine` - `.` matches newlines (default: false)
- `unicode` - Unicode character classes (default: true)

**File Walking Options (Full Traversal):**
- `maxDepth` - Maximum directory depth (default: unlimited)
- `hidden` - Include hidden files (default: false)
- `followLinks` - Follow symbolic links (default: false)
- `maxFilesize` - Skip files larger than N bytes (default: unlimited)
- `gitIgnore` - Respect .gitignore files (default: true)
- `ignore` - Respect .ignore files (default: true)
- `parents` - Read parent directory ignore files (default: true)
- `sameFileSystem` - Don't cross filesystem boundaries (default: false)
- `threads` - Parallel search threads (default: auto)
- `types` - File type filters like `["rust", "js"]` (default: none)

**Search Options:**
- `context` - Lines of context before and after matches (default: 0)
- `countMode` - How to count: `lines` or `occurrences` (default: `lines`)
- `invertMatch` - Show non-matching lines (default: false)

**Common Fields (shared with `run`):**
- `message` - Error message on constraint violation
- `action` - `block` (default) or `warn`
- `showStdout` - Show match output (default: false)
- `showStderr` - Show errors (default: false)
- `maxOutputLines` - Limit output lines (default: unlimited)
- `timeout` - Execution timeout in seconds

### Configuration Examples

```yaml
stop:
  commands:
    # Basic: Block inline CSS in HTML (any match = fail)
    - rg:
        pattern: "class=.*(?:bg-|text-|flex|grid)"
        files: "**/*.html"
      message: "Inline CSS classes found - use CSS modules"
      showStdout: true
      maxOutputLines: 20

    # With explicit constraint - allow up to 5 TODOs
    - rg:
        pattern: "TODO|FIXME"
        files: "**/*.rs"
        max: 5
        ignoreCase: true
      message: "Too many TODO/FIXME comments (max 5)"
      action: warn

    # Require at least one test
    - rg:
        pattern: "#\\[test\\]"
        files: "**/*.rs"
        min: 1
        types: ["rust"]
      message: "No test functions found"

    # Exact count validation
    - rg:
        pattern: "fn main\\("
        files: "src/**/*.rs"
        equal: 1
        wholeLine: false
      message: "Expected exactly one main function"

    # Word boundary matching with context
    - rg:
        pattern: "unwrap"
        files: "**/*.rs"
        word: true
        max: 0
        context: 2
      message: "Found unwrap() - use proper error handling"
      showStdout: true

    # Count occurrences instead of lines
    - rg:
        pattern: "console\\.log"
        files: "**/*.{js,ts}"
        max: 10
        countMode: occurrences
      message: "Too many console.log calls"

    # Full traversal options
    - rg:
        pattern: "SECRET|PASSWORD|API_KEY"
        files: "**/*"
        ignoreCase: true
        hidden: true
        gitIgnore: false
        maxDepth: 10
        max: 0
      message: "Potential secrets in codebase"

subagentStop:
  commands:
    coder:
      - rg:
          pattern: "unwrap\\(\\)|expect\\("
          files: "**/*.rs"
          max: 0
        message: "Coder introduced unwrap/expect - use Result handling"
```

## Impact

- Affected specs: `execution`, `configuration`
- Affected code: `src/config.rs`, `src/hooks.rs`
- New dependencies: `grep-searcher`, `grep-regex`, `ignore` crates
- Breaking changes: None (additive feature)
- Binary size impact: ~2-3MB increase from ripgrep libraries

## Technical Notes

### Library Integration

Uses ripgrep's core library crates directly:
- `grep-regex` - Regex compilation with all options
- `grep-searcher` - File searching with binary detection
- `ignore` - File traversal with gitignore support

This approach:
- Requires no external `rg` binary installation
- Provides full control over all options
- Enables proper error handling and output capture
- Allows match counting at the library level

### Match Counting Semantics

- `countMode: lines` - Count unique lines containing matches (default)
- `countMode: occurrences` - Count every match instance

Example: Line `"TODO: fix TODO: test"` with pattern `TODO`:
- Lines mode: 1 match
- Occurrences mode: 2 matches

### Binary File Handling

By default, search quits when NUL byte detected (standard ripgrep behavior).
This prevents garbage output from binary files.

### Empty Glob Handling

If `files` glob matches zero files, the command fails with a validation error.
This catches typos like `**/*.rss` instead of `**/*.rs`.
