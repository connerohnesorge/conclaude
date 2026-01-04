# Change: Add tree-sitter-based `ts` hook command type

## Why

Currently, enforcing structural code patterns in hooks requires either:
1. Regex patterns via `rg` which can't understand code structure
2. Custom shell scripts that are hard to maintain

A tree-sitter-based `ts` command type would provide:
- Semantic code analysis that understands AST structure
- Language-aware pattern matching (functions, classes, unsafe blocks, etc.)
- Native tree-sitter query syntax with predicates for filtering
- Count and text-based assertions on matched nodes
- No external dependencies (grammars bundled in binary)

## What Changes

### New `ts` Field Structure

The `ts` field is mutually exclusive with `run` and `rg`. It uses tree-sitter S-expression queries.

**Required Fields:**
- `query` - Tree-sitter S-expression query string
- `files` - Glob pattern for file matching

**Constraint Fields (mutually exclusive, default: `max: 0`):**
- `max` - Maximum allowed captures
- `min` - Minimum required captures
- `equal` - Exact capture count required

**Query Options:**
- `capture` - Which capture to count/assert on (default: first capture in query)
- `language` - Override auto-detected language (optional)

**File Walking Options (shared with `rg` command):**
- `maxDepth`, `hidden`, `followLinks`, `maxFilesize`
- `gitIgnore`, `ignore`, `parents`, `sameFileSystem`
- `threads`

**Common Fields (shared with `run` and `rg`):**
- `message` - Error message on constraint violation
- `action` - `block` (default) or `warn`
- `showStdout` - Show matched nodes (default: false)
- `showStderr` - Show parse errors (default: false)
- `maxOutputLines` - Limit output lines
- `timeout` - Execution timeout in seconds

### Bundled Languages (~25)

Primary: `rust`, `javascript`, `typescript`, `python`, `go`, `java`, `c`, `cpp`
Web: `html`, `css`, `json`, `yaml`, `toml`, `markdown`
Additional: `ruby`, `php`, `csharp`, `swift`, `kotlin`, `bash`, `sql`, `lua`, `zig`, `haskell`, `ocaml`

### Configuration Examples

```yaml
stop:
  commands:
    # Block unsafe blocks in Rust (any match = fail)
    - ts:
        query: "(unsafe_block) @unsafe"
        files: "**/*.rs"
      message: "Unsafe blocks are not allowed"
      showStdout: true
      maxOutputLines: 10

    # Require at least one test function
    - ts:
        query: |
          (function_item
            (attribute_item (attribute (identifier) @attr))
            name: (identifier) @name
            (#eq? @attr "test")) @test_fn
        files: "**/*.rs"
        min: 1
        capture: "@test_fn"
      message: "No test functions found"

    # Block console.log in production JS
    - ts:
        query: |
          (call_expression
            function: (member_expression
              object: (identifier) @obj
              property: (property_identifier) @prop)
            (#eq? @obj "console")
            (#eq? @prop "log")) @console_log
        files: "src/**/*.{js,ts}"
        max: 0
      message: "console.log calls not allowed in production"

    # Allow max 5 TODO comments
    - ts:
        query: |
          (comment) @comment
          (#match? @comment "TODO|FIXME")
        files: "**/*.rs"
        max: 5
      message: "Too many TODO comments (max 5)"
      action: warn

    # Ensure main function exists
    - ts:
        query: |
          (function_item
            name: (identifier) @name
            (#eq? @name "main")) @main_fn
        files: "src/main.rs"
        min: 1
        capture: "@main_fn"
      message: "No main function found"

    # Block unwrap() calls
    - ts:
        query: |
          (call_expression
            function: (field_expression
              field: (field_identifier) @method)
            (#any-of? @method "unwrap" "expect")) @unwrap_call
        files: "**/*.rs"
        max: 0
      message: "unwrap()/expect() calls not allowed - use proper error handling"

subagentStop:
  commands:
    coder:
      - ts:
          query: "(unsafe_block) @unsafe"
          files: "**/*.rs"
          max: 0
        message: "Coder introduced unsafe code"
```

### Output Format

When `showStdout: true`, matches are displayed as:
```
src/lib.rs:42:5 [unsafe_block]: unsafe { ptr::read(addr) }
src/lib.rs:87:9 [unsafe_block]: unsafe { mem::transmute(x) }
```

Format: `file:line:column [node_type]: captured_text`

## Impact

- Affected specs: `execution`, `configuration`
- Affected code: `src/config.rs`, `src/hooks.rs`, new `src/treesitter.rs`
- New dependencies: `tree-sitter`, `rs-tree-sitter-languages` (or individual grammar crates)
- Breaking changes: None (additive feature)
- Binary size impact: ~15-25MB increase from bundled grammars

## Technical Notes

### Language Detection

Languages are auto-detected from file extensions:
- `.rs` -> Rust
- `.js`, `.mjs`, `.cjs` -> JavaScript
- `.ts`, `.tsx` -> TypeScript
- `.py` -> Python
- `.go` -> Go
- `.java` -> Java
- `.c`, `.h` -> C
- `.cpp`, `.cc`, `.hpp` -> C++
- etc.

The `language` field can override auto-detection when needed.

### Query Predicates

Full tree-sitter predicate support:
- `#eq?` / `#not-eq?` - Exact text comparison
- `#match?` / `#not-match?` - Regex matching
- `#any-of?` - Match any of listed values
- `#any-eq?` / `#any-match?` - Quantified capture variants

### Capture Counting

By default, counts the first capture in the query. Use `capture: "@name"` to count a specific capture when query has multiple.

### Parse Error Handling

Files that fail to parse are skipped with a warning (logged to stderr if `showStderr: true`). This allows graceful handling of syntax errors or unsupported language features.

### Performance

- Each file is parsed once per query
- Tree-sitter parsing is incremental and fast (~2-3x slower than hand-written parsers)
- Queries are compiled once and reused across files
- Parallel file processing via the ignore crate's WalkParallel
